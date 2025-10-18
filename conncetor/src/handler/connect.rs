use axum::Extension;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use crossfire::mpsc;
use futures::StreamExt;
use tracing::{debug, trace};

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
        handle_websock_conn(socket, app_state, user_id);
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

    let (websock_tx, mut websock_rx) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_async::<String>();

    debug!(
        "WebSocket connection handler exiting for user_id: {}",
        user_id
    );
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
