use axum::extract::ws;

use crate::message::ServiceMessage;

/// Handle a service message and convert it to a WebSocket message.
/// It may comes from gRPC server functions.
pub async fn handle_serv_message(serv_message: ServiceMessage) -> anyhow::Result<Message> {
    unimplemented!()
}

pub async fn handle_websock_message(websock_message: ws::Message) -> anyhow::Result<()> {
    

    unimplemented!()
}
