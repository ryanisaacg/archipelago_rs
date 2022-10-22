use tungstenite::protocol::Message;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream, connect_async};
use crate::server_message::{ServerMessage, RoomInfo};
use crate::client_message::ClientMessage;
use futures_util::{SinkExt, StreamExt};

use thiserror::Error;
// TODO: error handling

#[derive(Error, Debug)]
pub enum ArchipelagoError {
    #[error("data failed to serialize")]
    FailedSerialize(#[from] serde_json::Error),
    #[error("network error")]
    NetworkError(#[from] tungstenite::Error),
    #[error("unexpected non-text result from websocket")]
    NonTextWebsocketResult(Message),
}

type Result<T> = std::result::Result<T, ArchipelagoError>;

pub async fn connect_websocket(url: &str) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Option<RoomInfo>)> {
    let (mut ws_stream, _response) = connect_async(url).await?;
    let response = recv_messages(&mut ws_stream).await;
    let response = match response {
        Some(Ok(response)) => match response.into_iter().next() {
            Some(ServerMessage::RoomInfo(room_info)) => Some(room_info),
            _ => None,
        },
        Some(Err(err)) => return Err(err),
        None => None
    };

    return Ok((ws_stream, response));
}

pub async fn send_message(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, message: ClientMessage) -> Result<()> {
    send_messages(ws, &[message]).await
}

pub async fn send_messages(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, messages: &[ClientMessage]) -> Result<()> {
    let request = serde_json::to_string(messages)?;
    ws.send(Message::Text(request)).await?;

    Ok(())
}

pub async fn recv_messages(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Option<Result<Vec<ServerMessage>>> {
    match ws.next().await? {
        Ok(Message::Text(response)) => Some(serde_json::from_str::<Vec<ServerMessage>>(&response)
            .map_err(|e| e.into())),
        Ok(msg) => Some(Err(ArchipelagoError::NonTextWebsocketResult(msg))),
        Err(e) => Some(Err(e.into())),
    }
}

