use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::Message;

use crate::protocol::*;

#[derive(Error, Debug)]
pub enum ArchipelagoError {
    #[error("illegal response")]
    IllegalResponse {
        received: ServerMessage,
        expected: &'static str,
    },
    #[error("connection closed by server")]
    ConnectionClosed,
    #[error("data failed to serialize")]
    FailedSerialize(#[from] serde_json::Error),
    #[error("unexpected non-text result from websocket")]
    NonTextWebsocketResult(Message),
    #[error("network error")]
    NetworkError(#[from] tungstenite::Error),
}

/**
 * A convenience layer to manage your connection to and communication with Archipelago
 */
pub struct ArchipelagoClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    room_info: RoomInfo,
    message_buffer: Vec<ServerMessage>,
    data_package: Option<DataPackageObject>,
}

impl ArchipelagoClient {
    /**
     * Create an instance of the client and connect to the server on the given URL
     */
    pub async fn new(url: &str) -> Result<ArchipelagoClient, ArchipelagoError> {
        let (mut ws, _) = connect_async(url).await?;
        let response = recv_messages(&mut ws)
            .await
            .ok_or(ArchipelagoError::ConnectionClosed)??;
        let mut iter = response.into_iter();
        let room_info = match iter.next() {
            Some(ServerMessage::RoomInfo(room)) => room,
            Some(received) => {
                return Err(ArchipelagoError::IllegalResponse {
                    received,
                    expected: "Expected RoomInfo",
                })
            }
            None => return Err(ArchipelagoError::ConnectionClosed),
        };

        Ok(ArchipelagoClient {
            ws,
            room_info,
            message_buffer: iter.collect(),
            data_package: None,
        })
    }

    /**
     * Create an instance of the client and connect to the server, fetching the given games' Data
     * Package
     */
    pub async fn with_data_package(
        url: &str,
        games: Option<Vec<String>>,
    ) -> Result<ArchipelagoClient, ArchipelagoError> {
        let mut client = Self::new(url).await?;
        client
            .send(ClientMessage::GetDataPackage(GetDataPackage { games }))
            .await?;
        let response = client.recv().await?;
        match response {
            Some(ServerMessage::DataPackage(pkg)) => client.data_package = Some(pkg.data),
            Some(received) => {
                return Err(ArchipelagoError::IllegalResponse {
                    received,
                    expected: "DataPackage",
                })
            }
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
        let request = serde_json::to_string(&[message])?;
        self.ws.send(Message::Text(request)).await?;

        Ok(())
    }

    /**
     * Read a message from the server
     *
     * Will buffer results locally, and return results from buffer or wait on network
     * if buffer is empty
     */
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

    /**
     * Send a connect request to the Archipelago server
     *
     * Will attempt to read a Connected packet in response, and will return an error if
     * another packet is found
     */
    pub async fn connect(
        &mut self,
        game: &str,
        name: &str,
        password: Option<&str>,
        items_handling: Option<i32>,
        tags: Vec<String>,
    ) -> Result<Connected, ArchipelagoError> {
        self.send(ClientMessage::Connect(Connect {
            game: game.to_string(),
            name: name.to_string(),
            uuid: "".to_string(),
            password: password.map(|p| p.to_string()),
            version: network_version(),
            items_handling,
            tags,
        }))
        .await?;
        let response = self
            .recv()
            .await?
            .ok_or(ArchipelagoError::ConnectionClosed)?;

        match response {
            ServerMessage::Connected(connected) => Ok(connected),
            received => Err(ArchipelagoError::IllegalResponse {
                received,
                expected: "Connected",
            }),
        }
    }

    /**
     * Basic chat command which sends text to the server to be distributed to other clients.
     */
    pub async fn say(&mut self, message: &str) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::Say(Say {
                text: message.to_string(),
            }))
            .await?)
    }

    /**
     * Sent to server to request a ReceivedItems packet to synchronize items.
     *
     * Will buffer any non-ReceivedItems packets returned
     */
    pub async fn sync(&mut self) -> Result<ReceivedItems, ArchipelagoError> {
        self.send(ClientMessage::Sync).await?;
        while let Some(response) = self.recv().await? {
            match response {
                ServerMessage::ReceivedItems(items) => return Ok(items),
                resp => self.message_buffer.push(resp),
            }
        }

        Err(ArchipelagoError::ConnectionClosed)
    }

    /**
     * Sent to server to inform it of locations that the client has checked.
     *
     * Used to inform the server of new checks that are made, as well as to sync state.
     */
    pub async fn location_checks(&mut self, locations: Vec<i32>) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::LocationChecks(LocationChecks { locations }))
            .await?)
    }

    /**
     * Sent to the server to inform it of locations the client has seen, but not checked.
     *
     * Useful in cases in which the item may appear in the game world, such as 'ledge items' in A Link to the Past. Non-LocationInfo packets will be buffered
     */
    pub async fn location_scouts(
        &mut self,
        locations: Vec<i32>,
        create_as_hint: i32,
    ) -> Result<LocationInfo, ArchipelagoError> {
        self.send(ClientMessage::LocationScouts(LocationScouts {
            locations,
            create_as_hint,
        }))
        .await?;
        while let Some(response) = self.recv().await? {
            match response {
                ServerMessage::LocationInfo(items) => return Ok(items),
                resp => self.message_buffer.push(resp),
            }
        }

        Err(ArchipelagoError::ConnectionClosed)
    }

    /**
     * Sent to the server to update on the sender's status.
     *
     * Examples include readiness or goal completion. (Example: defeated Ganon in A Link to the Past)
     */
    pub async fn status_update(&mut self, status: ClientStatus) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::StatusUpdate(StatusUpdate { status }))
            .await?)
    }

    /**
     * Send this message to the server, tell it which clients should receive the message and the server will forward the message to all those targets to which any one requirement applies.
     */
    pub async fn bounce(
        &mut self,
        games: Option<Vec<String>>,
        slots: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        data: serde_json::Value,
    ) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::Bounce(Bounce {
                games,
                slots,
                tags,
                data,
            }))
            .await?)
    }

    /**
     * Used to request a single or multiple values from the server's data storage, see the Set package for how to write values to the data storage.
     *
     * A Get package will be answered with a Retrieved package. Non-Retrieved responses are
     * buffered
     */
    pub async fn get(&mut self, keys: Vec<String>) -> Result<Retrieved, ArchipelagoError> {
        self.send(ClientMessage::Get(Get { keys })).await?;
        while let Some(response) = self.recv().await? {
            match response {
                ServerMessage::Retrieved(items) => return Ok(items),
                resp => self.message_buffer.push(resp),
            }
        }

        Err(ArchipelagoError::ConnectionClosed)
    }

    /**
     * Used to write data to the server's data storage, that data can then be shared across worlds or just saved for later.
     *
     * Values for keys in the data storage can be retrieved with a Get package, or monitored with a SetNotify package. Non-SetReply responses are buffered
     */
    pub async fn set(
        &mut self,
        key: String,
        default: serde_json::Value,
        want_reply: bool,
        operations: Vec<DataStorageOperation>,
    ) -> Result<SetReply, ArchipelagoError> {
        self.send(ClientMessage::Set(Set {
            key,
            default,
            want_reply,
            operations,
        }))
        .await?;
        while let Some(response) = self.recv().await? {
            match response {
                ServerMessage::SetReply(items) => return Ok(items),
                resp => self.message_buffer.push(resp),
            }
        }

        Err(ArchipelagoError::ConnectionClosed)
    }
}

async fn recv_messages(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Option<Result<Vec<ServerMessage>, ArchipelagoError>> {
    match ws.next().await? {
        Ok(Message::Text(response)) => {
            Some(serde_json::from_str::<Vec<ServerMessage>>(&response).map_err(|e| e.into()))
        }
        Ok(Message::Close(_)) => Some(Err(ArchipelagoError::ConnectionClosed)),
        Ok(msg) => Some(Err(ArchipelagoError::NonTextWebsocketResult(msg))),
        Err(e) => Some(Err(e.into())),
    }
}
