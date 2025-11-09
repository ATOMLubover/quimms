use std::net::SocketAddrV4;

use axum::{Router, routing};
use tokio::net::TcpListener;
use tracing::debug;

use crate::state::AppState;

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

async fn signal_term() {
    debug!("SIGNAL TERM receiver installed");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL-C signal handler");

    debug!("SIGNAL TERM received, shutting down gracefully...");
}

async fn new_router(state: &AppState) -> anyhow::Result<Router> {
    let router: Router = Router::new()
        .route("/check", routing::get(|| async { "OK" }))
        .with_state(state.clone());

    Ok(router)
}

pub async fn run_server(state: &AppState) -> anyhow::Result<()> {
    let listener = bind_addr(state.config().http_host(), state.config().http_port()).await?;

    let router = new_router(state).await?;

    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(signal_term())
        .await
        .map_err(|err| anyhow::anyhow!("Error running server: {}", err))?;

    Ok(())
}

mod websock {

    use axum::{
        Extension,
        body::Bytes,
        extract::{
            State,
            ws::{Message, WebSocket, WebSocketUpgrade},
        },
        response::IntoResponse,
    };
    use crossfire::mpsc;
    use futures::{SinkExt as _, StreamExt as _};
    use serde::{Deserialize, Serialize};
    use tracing::{debug, trace};

    use crate::{
        model::dto::{
            CreateChannelReq, CreateChannelRsp, CreateMessageReq, CreateMessageRsp, GetUserInfoReq,
            GetUserInfoRsp, JoinChannelReq, JoinChannelRsp, ListChannelDetailsReq,
            ListChannelDetailsRsp, ListMessagesReq, ListMessagesRsp, LoginUserReq, LoginUserRsp,
            RegisterUserReq, RegisterUserRsp,
        },
        service,
        state::AppState,
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type", content = "data")]
    #[serde(rename_all = "snake_case")]
    pub enum ReqMessage {
        RegisterUser(RegisterUserReq),
        LoginUser(LoginUserReq),
        GetUserInfo(GetUserInfoReq),
        CreateChannel(CreateChannelReq),
        ListChannelDetails(ListChannelDetailsReq),
        JoinChannel(JoinChannelReq),
        CreateMessage(CreateMessageReq),
        ListMessages(ListMessagesReq),
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type", content = "data")]
    #[serde(rename_all = "snake_case")]
    pub enum RspMessage {
        RegisterUser(RegisterUserRsp),
        LoginUser(LoginUserRsp),
        GetUserInfo(GetUserInfoRsp),
        CreateChannel(CreateChannelRsp),
        ListChannelDetails(ListChannelDetailsRsp),
        JoinChannel(JoinChannelRsp),
        CreateMessage(CreateMessageRsp),
        ListMessages(ListMessagesRsp),
    }

    pub async fn on_websock_connect(
        upgrade: WebSocketUpgrade,
        State(app_state): State<AppState>,
        Extension(user_id): Extension<String>,
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

        let (mut websock_tx, mut websock_rx) = socket.split();

        // NOTICE: Using bounded channel to prevent memory overflow in case of slow clients.
        // But it may drop messages if the client comsumes too slowly.
        let (serv_tx, serv_rx) = mpsc::bounded_async(64);

        app_state
            .online_users()
            .entry(user_id.clone())
            .or_insert(serv_tx);

        let user_id_cloned = user_id.clone();

        let websock_send_task = tokio::spawn(async move {
            while let Ok(serv_message) = serv_rx.recv().await {
                match service::handle_serv_message(serv_message).await {
                    Ok(websock_message) => {
                        if websock_tx.send(websock_message).await.is_err() {
                            // If any error occurs, we assume the client has disconnected and break the loop.
                            debug!(
                                "WebSocket send error for user_id: {}, disconnecting",
                                &user_id_cloned
                            );

                            break;
                        }
                    }
                    Err(err) => {
                        debug!(
                            "Error handling inner message: {} for user_id {}",
                            err, &user_id_cloned
                        );
                    }
                }
            }

            debug!(
                "WebSocket send task exiting for user_id: {}",
                &user_id_cloned
            );
        });

        let user_id_cloned = user_id.clone();

        let websock_recv_task = tokio::spawn(async move {
            while let Some(result) = websock_rx.next().await {
                match result {
                    Ok(msg) => {
                        trace!(
                            "Received message from client {}: {:?}",
                            &user_id_cloned, msg
                        );
                        // Here you can handle messages received from the client if needed.
                    }
                    Err(err) => {
                        debug!(
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
}
