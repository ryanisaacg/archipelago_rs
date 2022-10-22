use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use serde_json::Value;

use crate::NetworkVersion;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum ClientMessage {
    Connect(Connect),
    Sync(Sync),
    LocationChecks(LocationChecks),
    LocationScouts(LocationScouts),
    StatusUpdate(StatusUpdate),
    Say(Say),
    GetDataPackage(GetDataPackage),
    Bounce(Bounce),
    Get(Get),
    Set(Set),
    SetNotify(SetNotify),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connect {
    pub password: Option<String>,
    pub name: String,
    pub version: NetworkVersion,
    pub items_handling: Option<i32>,
    pub tags: Vec<String>,
    pub uuid: String,
    pub game: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectUpdate {
    pub items_handling: i32,
    pub tags: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sync;

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationChecks {
    pub locations: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationScouts {
    pub locations: Vec<i32>,
    pub create_as_hint: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub status: ClientStatus,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum ClientStatus {
    ClientUnknown = 0,
    ClientReady = 10,
    ClientPlaying = 20,
    ClientGoal = 30,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Say {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataPackage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub games: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounce {
    pub games: Option<Vec<String>>,
    pub slots: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Get {
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Set {
    pub key: String,
    pub default: Value,
    pub want_reply: bool,
    pub operations: Vec<DataStorageOperation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataStorageOperation {
    pub replace: String, // TODO: enum-ify?
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetNotify {
    pub keys: Vec<String>,
}

