use std::collections::HashMap;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd")]
pub enum ServerMessage<S> {
    RoomInfo(RoomInfo),
    ConnectionRefused(ConnectionRefused),
    Connected(Connected<S>),
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

impl<S> ServerMessage<S> {
    /// Returns the name of this message's type.
    pub fn type_name(&self) -> &'static str {
        use ServerMessage::*;
        match self {
            RoomInfo(_) => "RoomInfo",
            ConnectionRefused(_) => "ConnectionRefused",
            Connected(_) => "Connected",
            ReceivedItems(_) => "ReceivedItems",
            LocationInfo(_) => "LocationInfo",
            RoomUpdate(_) => "RoomUpdate",
            Print(_) => "Print",
            PrintJSON(_) => "PrintJSON",
            DataPackage(_) => "DataPackage",
            Bounced(_) => "Bounced",
            InvalidPacket(_) => "InvalidPacket",
            Retrieved(_) => "Retrieved",
            SetReply(_) => "SetReply",
        }
    }
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum Permission {
    Disabled = 0,
    Enabled = 1,
    Goal = 2,
    Auto = 6,
    AutoEnabled = 7,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkVersion {
    pub major: i32,
    pub minor: i32,
    pub build: i32,
    pub class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlayer {
    pub team: i32,
    pub slot: i32,
    pub alias: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkItem {
    pub item: i64,
    pub location: i64,
    pub player: i32,
    pub flags: NetworkItemFlags,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(from = "u8")]
    #[serde(into = "u8")]
    pub struct NetworkItemFlags: u8 {
        /// The item can unlock logical advancement.
        const PROGRESSION = 0b001;

        /// The item is especially useful.
        const USEFUL = 0b010;

        /// The item is a trap.
        const TRAP = 0b100;
    }
}

impl From<u8> for NetworkItemFlags {
    fn from(value: u8) -> NetworkItemFlags {
        NetworkItemFlags::from_bits_retain(value)
    }
}

impl Into<u8> for NetworkItemFlags {
    fn into(self) -> u8 {
        self.bits()
    }
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum SlotType {
    Spectator = 0,
    Player = 1,
    Group = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSlot {
    pub name: String,
    pub game: String,
    pub r#type: SlotType,
    pub group_members: Vec<i32>,
}

pub fn network_version() -> NetworkVersion {
    NetworkVersion {
        major: 0,
        minor: 6,
        build: 0,
        class: "Version".to_string(),
    }
}

// REQUESTS

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connect {
    pub password: Option<String>,
    pub game: String,
    pub name: String,
    pub uuid: String,
    pub version: NetworkVersion,
    pub items_handling: u32,
    pub tags: Vec<String>,
    #[serde(rename = "slot_data")]
    pub slot_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectUpdate {
    pub items_handling: i32,
    pub tags: Vec<String>,
}

bitflags! {
    #[repr(transparent)]
    pub struct ItemsHandlingFlags: u32 {
        /// Items are sent from other worlds.
        const OTHER_WORLDS = 0b001;

        /// Items are sent from your own world. Setting this automatically sets
        /// [OTHER_WORLDS] as well.
        const OWN_WORLD = 0b011;

        /// Items are sent from your starting inventory. Setting this
        /// automatically sets [OTHER_WORLDS] as well.
        const STARTING_INVENTORY = 0b101;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationChecks {
    pub locations: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationScouts {
    pub locations: Vec<i32>,
    pub create_as_hint: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHint {
    pub player: i32,
    pub location: i64,
    pub status: HintStatus,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum HintStatus {
    HintFound = 0,
    HintUnspecified = 1,
    HintNoPriority = 10,
    HintAvoid = 20,
    HintPriority = 30,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub status: ClientStatus,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum ClientStatus {
    ClientUnknown = 0,
    ClientReady = 10,
    ClientPlaying = 20,
    ClientGoal = 30,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Say {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDataPackage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub games: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounce {
    pub games: Option<Vec<String>>,
    pub slots: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Get {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Set {
    pub key: String,
    pub default: Value,
    pub want_reply: bool,
    pub operations: Vec<DataStorageOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", content = "value", rename_all = "snake_case")]
pub enum DataStorageOperation {
    Replace(serde_json::Value),
    Default,
    Add(serde_json::Value),
    Mul(serde_json::Value),
    Pow(serde_json::Value),
    Mod(serde_json::Value),
    Floor,
    Ceil,
    Max(serde_json::Value),
    Min(serde_json::Value),
    And(serde_json::Value),
    Or(serde_json::Value),
    Xor(serde_json::Value),
    LeftShift(serde_json::Value),
    RightShift(serde_json::Value),
    Remove(serde_json::Value),
    Pop(serde_json::Value),
    Update(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetNotify {
    pub keys: Vec<String>,
}

// RESPONSES

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRefused {
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Connected<S> {
    pub team: i32,
    pub slot: i32,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: Vec<i32>,
    pub checked_locations: Vec<i32>,
    pub slot_data: S,
    pub slot_info: HashMap<String, NetworkSlot>, // TODO: docs claim this is an int key. they are lying?
    pub hint_points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedItems {
    pub index: i32,
    pub items: Vec<NetworkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub locations: Vec<NetworkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomUpdate {
    // Copied from RoomInfo
    pub version: Option<NetworkVersion>,
    pub tags: Option<Vec<String>>,
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
    pub checked_locations: Option<Vec<i32>>,
    pub missing_locations: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Print {
    pub text: String,
}

// Not a very elegant way to handle this. See
// https://github.com/serde-rs/serde/issues/1799.

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        team: i32,
        slot: i32,
        message: String,
    },
    ServerChat {
        data: Vec<JSONMessagePart>,
        message: String,
    },
    Tutorial {
        data: Vec<JSONMessagePart>,
    },
    TagsChanged {
        data: Vec<JSONMessagePart>,
        team: i32,
        slot: i32,
        tags: Vec<String>,
    },
    CommandResult {
        data: Vec<JSONMessagePart>,
    },
    AdminCommandResult {
        data: Vec<JSONMessagePart>,
    },
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
    Unknown {
        data: Vec<JSONMessagePart>,
    },
}

impl PrintJSON {
    /// A utility method that returns a message of an unknown type that just
    /// contains the given unformatted [text].
    pub fn message(text: String) -> PrintJSON {
        PrintJSON::Unknown {
            data: vec![JSONMessagePart::Text { text }],
        }
    }

    /// Returns the data field for any JSONMessagePart.
    pub fn data(&self) -> &Vec<JSONMessagePart> {
        use PrintJSON::*;
        match self {
            ItemSend { data, .. } => data,
            ItemCheat { data, .. } => data,
            Hint { data, .. } => data,
            Join { data, .. } => data,
            Part { data, .. } => data,
            Chat { data, .. } => data,
            ServerChat { data, .. } => data,
            Tutorial { data, .. } => data,
            TagsChanged { data, .. } => data,
            CommandResult { data, .. } => data,
            AdminCommandResult { data, .. } => data,
            Goal { data, .. } => data,
            Release { data, .. } => data,
            Collect { data, .. } => data,
            Countdown { data, .. } => data,
            Unknown { data, .. } => data,
        }
    }
}

impl fmt::Display for PrintJSON {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for part in self.data() {
            f.write_str(&part.text().as_str())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JSONMessagePart {
    PlayerId {
        text: String,
        player: i32,
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

impl JSONMessagePart {
    /// Returns the text field for any JSONMessagePart.
    pub fn text(&self) -> &String {
        use JSONMessagePart::*;
        match self {
            PlayerId { text, .. } => text,
            PlayerName { text, .. } => text,
            ItemId { text, .. } => text,
            ItemName { text, .. } => text,
            LocationId { text, .. } => text,
            LocationName { text, .. } => text,
            EntranceName { text, .. } => text,
            Color { text, .. } => text,
            Text { text, .. } => text,
        }
    }
}

impl fmt::Display for JSONMessagePart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.text().as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPackage {
    pub data: DataPackageObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPackageObject {
    pub games: HashMap<String, GameData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub item_name_to_id: HashMap<String, i64>,
    pub location_name_to_id: HashMap<String, i64>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounced {
    pub games: Vec<String>,
    pub slots: Vec<i32>,
    pub tags: Vec<String>,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidPacket {
    pub r#type: String,
    pub original_cmd: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retrieved {
    keys: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetReply {
    key: String,
    value: Value,
    original_value: Value,
}
