use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod client_message;
pub mod server_message;

pub fn network_version() -> NetworkVersion {
    NetworkVersion {
        major: 0,
        minor: 3,
        build: 5,
        class: "Version".to_string(),
    }
}

pub mod client;

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum Permission {
    Disabled = 0,
    Enabled = 1,
    Goal = 2,
    Auto = 6,
    AutoEnabled = 7,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkVersion {
    pub major: i32,
    pub minor: i32,
    pub build: i32,
    pub class: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPlayer {
    pub team: i32,
    pub slot: i32,
    pub alias: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkItem {
    pub item: i32,
    pub location: i32,
    pub player: i32,
    pub flags: i32,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum SlotType  {
    Spectator = 0,
    Player = 1,
    Group = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSlot {
    pub name: String,
    pub game: String,
    pub r#type: SlotType,
    pub group_members: Vec<i32>,
}

