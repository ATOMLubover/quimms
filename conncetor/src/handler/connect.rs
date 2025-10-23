use axum::Extension;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use crossfire::mpsc;
use futures::{SinkExt, StreamExt};
use tracing::{debug, trace};

use crate::service;
use crate::state::AppState;

pub async fn build_websock_conn(
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

async fn handle_websock_conn(mut socket: WebSocket, app_state: AppState, user_id: String) {
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
    let (inner_tx, mut inner_rx) = mpsc::bounded_async(64);

    app_state
        .online_users
        .entry(user_id.clone())
        .or_insert(inner_tx);

    let user_id_clone = user_id.clone();

    let websock_send_task = tokio::spawn(async move {
        while let Ok(inner_msg) = inner_rx.recv().await {
            // If any error occurs, we assume the client has disconnected and break the loop.
            match service::handle_inner_message(inner_msg).await {
                Ok(msg) => {
                    if websock_tx.send(msg).await.is_err() {
                        debug!(
                            "WebSocket send error for user_id: {}, disconnecting",
                            &user_id_clone
                        );
                        break;
                    }
                }
                Err(err) => {
                    debug!(
                        "Error handling inner message: {} for user_id {}",
                        err, &user_id_clone
                    );
                }
            }
        }

        debug!(
            "WebSocket send task exiting for user_id: {}",
            &user_id_clone
        );
    });

    let user_id_clone = user_id.clone();

    let websock_recv_task = tokio::spawn(async move {
        while let Some(result) = websock_rx.next().await {
            match result {
                Ok(msg) => {
                    trace!("Received message from client {}: {:?}", &user_id_clone, msg);
                    // Here you can handle messages received from the client if needed.
                }
                Err(err) => {
                    debug!(
                        "WebSocket receive error for user_id {}: {}",
                        &user_id_clone, err
                    );
                    break;
                }
            }
        }

        debug!(
            "WebSocket recv task exiting for user_id: {}",
            &user_id_clone
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
            Ok(())
        }
    }
}
