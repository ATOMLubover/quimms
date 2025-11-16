use std::{net::SocketAddrV4, sync::Arc};

use axum::{Router, http::StatusCode, response::IntoResponse, routing};
use tokio::net::TcpListener;
use tracing::debug;

use tokio::sync::Notify;

use crate::{service, state::AppState};

async fn bind_addr(host: &str, port: u16) -> anyhow::Result<TcpListener> {
    let addr: SocketAddrV4 = format!("{}:{}", host, port)
        .parse()
        .map_err(|err| anyhow::anyhow!("Error when parsing listening address: {}", err))?;

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|err| anyhow::anyhow!("Error when binding to address {}: {}", addr, err))?;

    debug!("Server now listening on {}", addr);

    Ok(listener)
}

async fn new_router(state: &AppState) -> anyhow::Result<Router> {
    let router: Router = Router::new()
        .route("/ws/{user_id}", routing::get(websock::on_websock_connect))
        .route("/check", routing::get(health_check))
        .with_state(state.clone());

    Ok(router)
}

pub async fn run_server(state: &AppState, shutdown: Arc<Notify>) -> anyhow::Result<()> {
    let listener = bind_addr(state.config().http_host(), state.config().http_port()).await?;

    let router = new_router(state).await?;

    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(async move {
            debug!("HTTP server awaiting shutdown signal");
            shutdown.notified().await;
            debug!("HTTP server received shutdown signal");
        })
        .await
        .map_err(|err| anyhow::anyhow!("Error running server: {}", err))?;

    Ok(())
}

// TODO: move it to an independent health module.
async fn health_check() -> impl IntoResponse {
    match service::health_check().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

mod websock {
    use std::ops::ControlFlow;

    use axum::{
        body::Bytes,
        extract::{
            Path, State,
            ws::{Message, WebSocket, WebSocketUpgrade},
        },
        response::IntoResponse,
    };
    use crossfire::mpsc;
    use futures::{SinkExt as _, StreamExt as _};
    use tracing::{debug, error, trace};

    use crate::{
        cache::CacheClient,
        service::{handle_serv_message, handle_websock_message},
        state::AppState,
    };

    pub async fn on_websock_connect(
        upgrade: WebSocketUpgrade,
        State(app_state): State<AppState>,
        Path(user_id): Path<String>,
    ) -> impl IntoResponse {
        debug!("Building websocket connection for user_id: {}", user_id);

        let app_state = app_state.clone();
        let user_id = user_id.clone();

        upgrade.on_upgrade(move |socket| async move {
            handle_websock_conn(socket, app_state, user_id).await;
        })
    }

    async fn handle_websock_conn(
        mut socket: axum::extract::ws::WebSocket,
        app_state: AppState,
        user_id: String,
    ) {
        debug!("WebSocket connection established for user_id: {}", user_id);

        if let Err(err) = initial_ping(&mut socket).await {
            debug!(
                "Initial ping failed for user_id: {}, error: {}",
                user_id, err
            );
            return;
        }

        // Register user as online in cache.
        // If we fail to register, we refuse to establish the connection.
        match register_user_online(
            app_state.cache(),
            &user_id,
            app_state.config().service_name(),
            app_state.config().service_id(),
        )
        .await
        {
            Ok(_) => {
                debug!("User {} registered as online successfully", user_id);
            }
            Err(err) => {
                error!(
                    "Error registering user {} as online: {}, closing connection",
                    user_id, err
                );

                return;
            }
        };

        let (mut websock_snd, mut websock_rcv) = socket.split();

        // NOTICE: Using bounded channel to prevent memory overflow in case of slow clients.
        // But it may drop messages if the client comsumes too slowly.
        let (serv_tx, serv_rx) = mpsc::bounded_async(64);

        app_state
            .online_users()
            .entry(user_id.clone())
            .or_insert(serv_tx.clone());

        let user_id_cloned = user_id.clone();

        let websock_send_task = tokio::spawn(async move {
            while let Ok(serv_message) = serv_rx.recv().await {
                if let Some(websock_message) = handle_serv_message(serv_message).await {
                    if websock_snd.send(websock_message).await.is_err() {
                        // If any error occurs, we assume the client has disconnected and break the loop.
                        error!(
                            "WebSocket send error for user_id: {}, disconnecting",
                            &user_id_cloned
                        );

                        break;
                    }
                }
            }

            debug!(
                "WebSocket send task exiting for user_id: {}",
                &user_id_cloned
            );
        });

        let user_id_cloned = user_id.clone();
        let user_serv_snd = serv_tx.clone();

        let app_state_clone = app_state.clone();

        let websock_recv_task = tokio::spawn(async move {
            while let Some(websock_result) = websock_rcv.next().await {
                match websock_result {
                    Ok(websock_message) => {
                        trace!(
                            "Received message from client {}: {:?}",
                            &user_id_cloned, &websock_message
                        );

                        match handle_websock_message(
                            &user_id_cloned,
                            &user_serv_snd,
                            &app_state_clone.user_registry(),
                            &app_state_clone.channel_registry(),
                            &app_state_clone.message_registry(),
                            websock_message,
                        )
                        .await
                        {
                            ControlFlow::Continue(()) => {
                                continue;
                            }
                            ControlFlow::Break(res) => {
                                if let Err(err) = res {
                                    error!(
                                        "Error handling WebSocket message for user_id {}: {}",
                                        &user_id_cloned, err
                                    );
                                }

                                break;
                            }
                        }
                    }
                    Err(err) => {
                        error!(
                            "WebSocket receive error for user_id {}: {}",
                            &user_id_cloned, err
                        );

                        break;
                    }
                }
            }

            debug!(
                "WebSocket recv task exiting for user_id: {}",
                &user_id_cloned
            );
        });

        // Stop both ends when either send or receive task completes.
        tokio::select! {
            _ = websock_send_task => {
                debug!("WebSocket send task completed firstly for user_id: {}", user_id);
            }
            _ = websock_recv_task => {
                debug!("WebSocket receive task completed firstly for user_id: {}", user_id);
            }
        }

        // Clean up the online user entry.
        app_state.online_users().remove(&user_id);

        match deregister_user_online(app_state.cache(), &user_id).await {
            Ok(_) => {
                debug!(
                    "User {} deregistered successfully upon WebSocket disconnection",
                    user_id
                );
            }
            Err(err) => {
                error!(
                    "Error deregistering user {} upon WebSocket disconnection: {}",
                    user_id, err
                );
            }
        }

        debug!("WebSocket connection exiting for user_id: {}", user_id);
    }

    async fn initial_ping(socket: &mut WebSocket) -> anyhow::Result<()> {
        if socket
            .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
            .await
            .is_err()
        {
            anyhow::bail!("Failed to send initial ping to client, disconnecting");
        }

        match socket.recv().await {
            Some(Err(err)) => {
                anyhow::bail!("Error receiving initial message from client: {}", err);
            }
            None => {
                anyhow::bail!("Client disconnected before sending initial message");
            }
            Some(Ok(msg)) => {
                trace!("Received initial message from client: {:?}", msg);
            }
        };

        Ok(())
    }

    async fn register_user_online(
        cache: &CacheClient,
        user_id: &str,
        service_name: &str,
        service_id: &str,
    ) -> anyhow::Result<()> {
        let server_token = format!("{}:{}", service_name, service_id);

        const HASH_KEY: &str = "user:connector";

        let result = cache
            .hash_set(HASH_KEY, user_id, &server_token)
            .await
            .map_err(|err| {
                anyhow::anyhow!("Error registering user {} with cache: {}", user_id, err)
            })?;

        match result {
            true => {
                debug!(
                    "User {} registered in cache with service token {}",
                    user_id, server_token
                );

                Ok(())
            }
            false => {
                debug!("User {} already registered in cache, skipping", user_id);

                anyhow::bail!("User {} already registered in cache", user_id);
            }
        }
    }

    async fn deregister_user_online(cache: &CacheClient, user_id: &str) -> anyhow::Result<()> {
        const HASH_KEY: &str = "user:connector";

        let result = cache.hash_delete(HASH_KEY, user_id).await.map_err(|err| {
            anyhow::anyhow!("Error deregistering user {} from cache: {}", user_id, err)
        })?;

        match result {
            true => {
                debug!("User {} deregistered from cache", user_id);

                Ok(())
            }
            false => {
                debug!("User {} not found in cache during deregistration", user_id);

                anyhow::bail!("User {} not found in cache during deregistration", user_id);
            }
        }
    }
}
