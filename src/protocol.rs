use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum ClientMessage {
    Connect(Connect),
    Sync,
    LocationChecks(LocationChecks),
    LocationScouts(LocationScouts),
    UpdateHint(UpdateHint),
    StatusUpdate(StatusUpdate),
    Say(Say),
    GetDataPackage(GetDataPackage),
    Bounce(Bounce),
    Get(Get),
    Set(Set),
    SetNotify(SetNotify),
}

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
    pub item: i64,
    pub location: i64,
    pub player: i32,
    pub flags: i32,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum SlotType {
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

pub fn network_version() -> NetworkVersion { // I should probably bump this
    NetworkVersion {
        major: 0,
        minor: 6,
        build: 3,
        class: "Version".to_string(),
    }
}

// REQUESTS

#[derive(Debug, Serialize, Deserialize)]
pub struct Connect {
    pub password: Option<String>,
    pub game: String,
    pub name: String,
    pub uuid: String,
    pub version: NetworkVersion,
    pub items_handling: Option<i32>,
    pub tags: Vec<String>,
    #[serde(rename = "slot_data")]
    pub request_slot_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectUpdate {
    pub items_handling: i32,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationChecks {
    pub locations: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationScouts {
    pub locations: Vec<i64>,
    pub create_as_hint: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateHint {
    pub player: i32,
    pub location: i64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<HintStatus>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum HintStatus {
    HintFound = 0,
    HintUnspecified = 1,
    HintNoPriority = 10,
    HintAvoid = 20,
    HintPriority = 30,
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
    #[serde(default)]
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
#[serde(tag = "operation", content = "value", rename_all = "snake_case")]
pub enum DataStorageOperation {
    Replace(Value),
    Default,
    Add(Value),
    Mul(Value),
    Pow(Value),
    Mod(Value),
    Floor,
    Ceil,
    Max(Value),
    Min(Value),
    And(Value),
    Or(Value),
    Xor(Value),
    LeftShift(Value),
    RightShift(Value),
    Remove(Value),
    Pop(Value),
    Update(Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetNotify {
    pub keys: Vec<String>,
}

// RESPONSES

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfo {
    pub version: NetworkVersion,
    pub generator_version: NetworkVersion,
    pub tags: Vec<String>,
    #[serde(rename = "password")]
    pub password_required: bool,
    pub permissions: HashMap<String, Permission>,
    pub hint_cost: i32,
    pub location_check_points: i32,
    pub games: Vec<String>,
    #[serde(default)]
    pub datapackage_versions: HashMap<String, i32>,
    #[serde(default)]
    pub datapackage_checksums: HashMap<String, String>,
    pub seed_name: String,
    pub time: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionRefused {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connected {
    pub team: i32,
    pub slot: i32,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: Vec<i64>,
    pub checked_locations: Vec<i64>,
    pub slot_data: Value,
    pub slot_info: HashMap<String, NetworkSlot>, // TODO: docs claim this is an int key. they are lying?
    pub hint_points: i32,
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
    // Copied from RoomInfo
    pub version: Option<NetworkVersion>,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "password")]
    pub password_required: bool,
    pub permissions: Option<HashMap<String, Permission>>,
    pub hint_cost: Option<i32>,
    pub location_check_points: Option<i32>,
    pub games: Option<Vec<String>>,
    pub datapackage_versions: Option<HashMap<String, i32>>,
    pub datapackage_checksums: Option<HashMap<String, String>>,
    pub seed_name: Option<String>,
    pub time: Option<f32>,
    // Exclusive to RoomUpdate
    pub hint_points: Option<i32>,
    pub players: Option<Vec<NetworkPlayer>>,
    pub checked_locations: Option<Vec<i64>>,
    pub missing_locations: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Print {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PrintJSON {
    ItemSend {
        data: Vec<JSONMessagePart>,
        receiving: i32,
        item: NetworkItem,
    },
    ItemCheat {
        data: Vec<JSONMessagePart>,
        receiving: i32,
        item: NetworkItem,
        team: i32,
    },
    Hint {
        data: Vec<JSONMessagePart>,
        receiving: i32,
        item: NetworkItem,
        found: bool,
    },
    Join {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
        tags: Vec<String>,
    },
    Part {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
    },
    Chat {
        data: Vec<JSONMessagePart>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        team: Option<i32>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        slot: Option<i32>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    ServerChat {
        data: Vec<JSONMessagePart>,
        message: String,
    },
    Tutorial { data: Vec<JSONMessagePart> },
    TagsChanged {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
        tags: Vec<String>,
    },
    CommandResult { data: Vec<JSONMessagePart> },
    AdminCommandResult { data: Vec<JSONMessagePart> },
    Goal {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
    },
    Release {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
    },
    Collect {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
    },
    Countdown {
        data: Vec<JSONMessagePart>,
        countdown: i32,
    },
    #[serde(untagged)]
    Text {
        data: Vec<JSONMessagePart>,
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JSONMessagePart {
    PlayerId {
        text: String,
    },
    PlayerName {
        text: String,
    },
    ItemId {
        text: String,
        flags: i32,
        player: i32,
    },
    ItemName {
        text: String,
        flags: i32,
        player: i32,
    },
    LocationId {
        text: String,
        player: i32,
    },
    LocationName {
        text: String,
        player: i32,
    },
    EntranceName {
        text: String,
    },
    Color {
        text: String,
        color: JSONColor,
    },
    #[serde(untagged)]
    Text {
        text: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JSONColor {
    Bold,
    Underline,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BlackBg,
    RedBg,
    GreenBg,
    YellowBg,
    BlueBg,
    MagentaBg,
    CyanBg,
    WhiteBg,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataPackage {
    pub data: DataPackageObject,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataPackageObject {
    pub games: HashMap<String, GameData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameData {
    pub item_name_to_id: HashMap<String, i64>,
    pub location_name_to_id: HashMap<String, i64>,
    //pub version: i32, // Shouldn't need this again
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounced {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub games: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots: Option<Vec<i32>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvalidPacket {
    pub r#type: String,
    pub original_cmd: Option<String>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Retrieved {
    pub keys: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetReply {
    pub key: String,
    pub value: Value,
    pub original_value: Value,
}