use crate::protocol::*;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, Stream, StreamExt,
};
use serde::Deserialize;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::{CloseFrame, Message};
use tungstenite::Utf8Bytes;

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
        // Attempt WSS, downgrade to WS if the TLS handshake fails
        let mut wss_url = String::new();
        wss_url.push_str("wss://");
        wss_url.push_str(url);
        let (mut ws, _) = match connect_async(&wss_url).await {
            Ok(result) => result,
            Err(tungstenite::error::Error::Tls(_)) => {
                let mut ws_url = String::new();
                ws_url.push_str("ws://");
                ws_url.push_str(url);
                connect_async(&ws_url).await?
            }
            Err(error) => return Err(ArchipelagoError::NetworkError(error)),
        };

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
        mut games: Option<Vec<String>>,
    ) -> Result<ArchipelagoClient, ArchipelagoError> {
        let mut client = Self::new(url).await?;
        if games.is_none() {
            // If None, request the games that are part of the connected room.
            let mut list: Vec<String> = vec![];
            client
                .room_info
                .datapackage_checksums
                .keys()
                .for_each(|name| {
                    list.push(name.clone());
                });
            games = Some(list);
        }
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
        self.ws
            .send(Message::Text(Utf8Bytes::from(request)))
            .await?;

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
            request_slot_data: true,
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

    /// Disconnect from the room
    pub async fn disconnect(&mut self, close_frame: Option<CloseFrame>) -> Result<(), tungstenite::Error> {
        self.ws.close(close_frame).await?;
        Ok(())
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
    pub async fn location_checks(&mut self, locations: Vec<i64>) -> Result<(), ArchipelagoError> {
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
        locations: Vec<i64>,
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

    /**
     * Split the client into two parts, one to handle sending and one to handle receiving.
     *
     * This removes access to a few convenience methods (like `get` or `set`) because it's
     * there's now extra coordination required to match a read and write, but it brings
     * the benefits of allowing simultaneous reading and writing.
     */
    pub fn split(self) -> (ArchipelagoClientSender, ArchipelagoClientReceiver) {
        let Self {
            ws,
            room_info,
            message_buffer,
            data_package,
        } = self;
        let (send, recv) = ws.split();
        (
            ArchipelagoClientSender { ws: send },
            ArchipelagoClientReceiver {
                ws: recv,
                room_info,
                message_buffer,
                data_package,
            },
        )
    }
}

/**
 * Once split, this struct handles the sending-side of your connection
 *
 * For helper method docs, see ArchipelagoClient. Helper methods that require
 * both sending and receiving are intentionally unavailable; for those messages,
 * use `send`.
 */
pub struct ArchipelagoClientSender {
    ws: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl ArchipelagoClientSender {
    pub async fn send(&mut self, message: ClientMessage) -> Result<(), ArchipelagoError> {
        let request = serde_json::to_string(&[message])?;
        self.ws
            .send(Message::Text(Utf8Bytes::from(request)))
            .await?;

        Ok(())
    }

    pub async fn say(&mut self, message: &str) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::Say(Say {
                text: message.to_string(),
            }))
            .await?)
    }

    pub async fn location_checks(&mut self, locations: Vec<i64>) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::LocationChecks(LocationChecks { locations }))
            .await?)
    }

    pub async fn status_update(&mut self, status: ClientStatus) -> Result<(), ArchipelagoError> {
        Ok(self
            .send(ClientMessage::StatusUpdate(StatusUpdate { status }))
            .await?)
    }

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
}

/**
 * Once split, this struct handles the receiving-side of your connection
 *
 * For helper method docs, see ArchipelagoClient. Helper methods that require
 * both sending and receiving are intentionally unavailable; for those messages,
 * use `recv`.
 */
pub struct ArchipelagoClientReceiver {
    ws: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    room_info: RoomInfo,
    message_buffer: Vec<ServerMessage>,
    data_package: Option<DataPackageObject>,
}

impl ArchipelagoClientReceiver {
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

    pub fn room_info(&self) -> &RoomInfo {
        &self.room_info
    }

    pub fn data_package(&self) -> Option<&DataPackageObject> {
        self.data_package.as_ref()
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MaybeArray {
    One(ServerMessage),
    Many(Vec<ServerMessage>),
}

impl Into<Vec<ServerMessage>> for MaybeArray {
    fn into(self) -> Vec<ServerMessage> {
        match self {
            MaybeArray::One(v) => vec![v],
            MaybeArray::Many(vs) => vs,
        }
    }
}


async fn recv_messages(
    mut ws: impl Stream<Item = Result<Message, tungstenite::error::Error>> + Unpin,
) -> Option<Result<Vec<ServerMessage>, ArchipelagoError>> {
    match ws.next().await? {
        Ok(Message::Text(response)) => {
            let result: Result<MaybeArray, _> = serde_json::from_str(&response);
            Some(result
                .map(|messages| messages.into())
                .map_err(|e| {
                    log::error!("Errored message: {}", response);
                    e.into()
                }))
        },
        Ok(Message::Close(_)) => Some(Err(ArchipelagoError::ConnectionClosed)),
        Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => None,
        Ok(msg) => Some(Err(ArchipelagoError::NonTextWebsocketResult(msg))),
        Err(e) => Some(Err(e.into())),
    }
}
