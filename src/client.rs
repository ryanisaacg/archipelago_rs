use futures_util::{SinkExt, StreamExt};
use tungstenite::protocol::Message;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream, connect_async};
use tokio::net::TcpStream;
use thiserror::Error;

use crate::network_version;
use crate::server_message::{ServerMessage, Connected, DataPackageObject, RoomInfo};
use crate::client_message::{ClientMessage, Connect as ClientConnect, GetDataPackage, Say};

#[derive(Error, Debug)]
pub enum ArchipelagoError {
    #[error("network-level error")]
    NetworkError(#[from] NetworkError),
    #[error("illegal response")]
    IllegalResponse { received: ServerMessage, expected: &'static str },
    #[error("connection closed by server")]
    ConnectionClosed,
}

pub struct ArchipelagoClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    room_info: RoomInfo,
    message_buffer: Vec<ServerMessage>,
    data_package: Option<DataPackageObject>,
}

impl ArchipelagoClient {
    pub async fn new(url: &str) -> Result<ArchipelagoClient, ArchipelagoError> {
        let (mut ws, _) = connect_async(url).await.map_err(|e| <tungstenite::Error as Into<NetworkError>>::into(e))?;
        let response = recv_messages(&mut ws).await.ok_or(ArchipelagoError::ConnectionClosed)??;
        let mut iter = response.into_iter();
        let room_info = match iter.next() {
            Some(ServerMessage::RoomInfo(room)) => room,
            Some(received) => return Err(ArchipelagoError::IllegalResponse { received , expected: "Expected RoomInfo" }),
            None => return Err(ArchipelagoError::ConnectionClosed),
        };

        Ok(ArchipelagoClient {
            ws,
            room_info,
            message_buffer: iter.collect(),
            data_package: None,
        })
    }

    pub async fn with_data_package(url: &str, games: Option<Vec<String>>) -> Result<ArchipelagoClient, ArchipelagoError> {
        let mut client = Self::new(url).await?;
        client.send(ClientMessage::GetDataPackage(GetDataPackage { games })).await?;
        let response = client.recv().await?;
        match response {
            Some(ServerMessage::DataPackage(pkg)) => client.data_package = Some(pkg.data),
            Some(received) => return Err(ArchipelagoError::IllegalResponse { received, expected: "DataPackage" }),
            None => return Err(ArchipelagoError::ConnectionClosed),
        }

        Ok(client)
    }

    pub fn room_info(&self) -> &RoomInfo {
        &self.room_info
    }

    pub fn data_package(&self) -> Option<&DataPackageObject> {
        self.data_package.as_ref()
    }

    pub async fn send(&mut self, message: ClientMessage) -> Result<(), ArchipelagoError> {
        Ok(send_message(&mut self.ws, message).await?)
    }

    pub async fn recv(&mut self) -> Result<Option<ServerMessage>, ArchipelagoError> {
        if let Some(message) = self.message_buffer.pop() {
            return Ok(Some(message));
        }
        let messages = recv_messages(&mut self.ws).await;
        if let Some(result) = messages {
            let mut messages = result?;
            messages.reverse();
            let first = messages.pop();
            self.message_buffer = messages;
            Ok(first)
        } else {
            Ok(None)
        }
    }

    pub async fn connect(
        &mut self,
        game: &str,
        name: &str,
        uuid: &str,
        password: Option<&str>,
        items_handling: Option<i32>,
        tags: Vec<String>,
    ) -> Result<Connected, ArchipelagoError> {
        self.send(ClientMessage::Connect(ClientConnect {
            game: game.to_string(),
            name: name.to_string(),
            uuid: uuid.to_string(),
            password: password.map(|p| p.to_string()),
            version: network_version(),
            items_handling,
            tags
        })).await?;
        let response = self.recv().await?.ok_or(ArchipelagoError::ConnectionClosed)?;

        match response {
            ServerMessage::Connected(connected) => Ok(connected),
            received => Err(ArchipelagoError::IllegalResponse { received, expected: "Connected" }),
        }
    }

    pub async fn say(&mut self, message: &str) -> Result<(), ArchipelagoError> {
        Ok(self.send(ClientMessage::Say(Say { text: message.to_string() })).await?)
    }

    // TODO: fetch data package
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("data failed to serialize")]
    FailedSerialize(#[from] serde_json::Error),
    #[error("network error")]
    NetworkError(#[from] tungstenite::Error),
    #[error("unexpected non-text result from websocket")]
    NonTextWebsocketResult(Message),
}

pub async fn send_message(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, message: ClientMessage) -> Result<(), NetworkError> {
    send_messages(ws, &[message]).await
}

pub async fn send_messages(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, messages: &[ClientMessage]) -> Result<(), NetworkError> {
    let request = serde_json::to_string(messages)?;
    ws.send(Message::Text(request)).await?;

    Ok(())
}

pub async fn recv_messages(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Option<Result<Vec<ServerMessage>, NetworkError>> {
    match ws.next().await? {
        Ok(Message::Text(response)) => Some(serde_json::from_str::<Vec<ServerMessage>>(&response)
            .map_err(|e| e.into())),
        Ok(msg) => Some(Err(NetworkError::NonTextWebsocketResult(msg))),
        Err(e) => Some(Err(e.into())),
    }
}

