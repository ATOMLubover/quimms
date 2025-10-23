use axum::extract::ws::Message;

use crate::message::InnerMessage;

pub async fn handle_inner_message(inner_message: InnerMessage) -> anyhow::Result<Message> {
    unimplemented!()
}

pub async fn handle_websock_message(websock_message: Message) -> anyhow::Result<()> {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    unimplemented!()
}
