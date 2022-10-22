use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{NetworkVersion, Permission, NetworkSlot, NetworkPlayer, NetworkItem};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum ServerMessage {
    RoomInfo(RoomInfo),
    ConnectionRefused(ConnectionRefused),
    Connected(Connected),
    ReceivedItems(ReceivedItems),
    LocationInfo(LocationInfo),
    RoomUpdate(RoomUpdate),
    Print(Print),
    PrintJSON(PrintJSON),
    DataPackage(DataPackage),
    Bounced(Bounced),
    InvalidPacket(InvalidPacket),
    Retrieved(Retrieved),
    SetReply(SetReply),
}

/**
 * Sent to clients when they connect to an Archipelago server.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfo {
    /**
     * Object denoting the version of Archipelago which the server is running.
     */
    pub version: NetworkVersion,
    /**
     pub * Denotes special features or capabilities that the sender is capable of. Example: WebHost
     */
    pub tags: Vec<String>,
    /**
     * Denoted whether a password is required to join this room.
     */
    pub password: bool,
    /**
     pub * Mapping of permission name to Permission, keys are: "forfeit", "collect" and "remaining".
     */
    pub permissions: HashMap<String, Permission>,
    /**
     * The amount of points it costs to receive a hint from the server.
     */
    pub hint_cost: i32,
    /**
     * The amount of hint points you receive per item/location check completed.
     */
    pub location_check_points: i32,
    /**
     * List of games present in this multiworld.
     */
    pub games: Vec<String>,
    pub datapackage_version: i32,
    pub datapackage_versions: HashMap<String, i32>,
    pub seed_name: String,
    pub time: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionRefused {
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connected {
    pub team: i32,
    pub slot: i32,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: Vec<i32>,
    pub checked_locations: Vec<i32>,
    pub slot_data: Value,
    pub slot_info: HashMap<String, NetworkSlot>, // TODO: docs claim this is an int key. they are lying?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedItems {
    pub index: i32,
    pub items: Vec<NetworkItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationInfo {
    pub locations: Vec<NetworkItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomUpdate {
    pub hint_points: i32,
    pub players: Vec<NetworkPlayer>,
    pub checked_locations: Vec<i32>,
    pub missing_locations: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Print {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintJSON {
    pub data: Vec<JSONMessagePart>,
    pub r#type: Option<String>,
    pub receiving: Option<i32>,
    pub item: Option<NetworkItem>,
    pub found: Option<bool>,
    pub countdown: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSONMessagePart {
    pub r#type: Option<String>,
    pub text: Option<String>,
    pub color: Option<String>,
    pub flags: Option<i32>,
    pub player: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataPackage {
    pub data: DataPackageObject
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataPackageObject {
    pub games: HashMap<String, GameData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub item_name_to_id: HashMap<String, i32>,
    pub location_name_to_id: HashMap<String, i32>,
    pub version: i32
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Bounced {
    pub games: Vec<String>,
    pub slots: Vec<i32>,
    pub tags: Vec<String>,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvalidPacket {
    pub r#type: String,
    pub original_cmd: Option<String>,
    pub text: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Retrieved {
    keys: Value,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SetReply {
    key: String,
    value: Value,
    original_value: Value,
}
