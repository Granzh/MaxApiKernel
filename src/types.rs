use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::enums::{AccessType, AttachType, ChatType, MessageStatus};

mod de_helpers {
    use serde::{de, Deserializer};
    use std::fmt;

    struct I64OrStr;
    impl<'de> de::Visitor<'de> for I64OrStr {
        type Value = i64;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("integer or string")
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<i64, E> {
            Ok(v)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<i64, E> {
            v.parse().map_err(de::Error::custom)
        }
    }
    pub fn i64_or_str<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
        d.deserialize_any(I64OrStr)
    }

    struct OptI64OrStr;
    impl<'de> de::Visitor<'de> for OptI64OrStr {
        type Value = Option<i64>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("optional integer or string")
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<i64>, E> {
            Ok(None)
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<i64>, E> {
            Ok(Some(v))
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<i64>, E> {
            Ok(Some(v as i64))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<i64>, E> {
            v.parse().map(Some).map_err(de::Error::custom)
        }
    }
    pub fn opt_i64_or_str<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
        d.deserialize_any(OptI64OrStr)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Presence {
    pub seen: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

pub type Names = Name;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: Option<i64>,
    pub account_status: Option<i64>,
    pub base_raw_url: Option<String>,
    pub base_url: Option<String>,
    pub names: Option<Vec<Name>>,
    pub options: Option<Vec<String>>,
    pub photo_id: Option<i64>,
    pub update_time: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub presence: Option<Presence>,
    pub read_mark: Option<i64>,
    pub contact: Option<Contact>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoAttach {
    pub base_url: String,
    pub height: i32,
    pub width: i32,
    pub photo_id: i64,
    pub photo_token: String,
    pub preview_data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAttach {
    pub height: i32,
    pub width: i32,
    pub video_id: i64,
    pub duration: i32,
    pub preview_data: Option<String>,
    pub thumbnail: Option<String>,
    pub token: Option<String>,
    pub video_type: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAttach {
    pub file_id: i64,
    pub name: String,
    pub size: i64,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlAttach {
    pub event: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickerAttach {
    pub author_type: Option<String>,
    pub lottie_url: Option<String>,
    pub url: Option<String>,
    pub sticker_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub width: Option<i32>,
    pub set_id: Option<i64>,
    pub time: Option<i64>,
    pub sticker_type: Option<String>,
    pub audio: Option<bool>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioAttach {
    pub duration: Option<i32>,
    pub audio_id: Option<i64>,
    pub url: Option<String>,
    pub wave: Option<String>,
    pub transcription_status: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactAttach {
    pub contact_id: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub name: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Attach {
    Photo(PhotoAttach),
    Video(VideoAttach),
    File(FileAttach),
    Control(ControlAttach),
    Sticker(StickerAttach),
    Audio(AudioAttach),
    Contact(ContactAttach),
    Unknown,
}

impl<'de> Deserialize<'de> for Attach {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let type_str = value.get("_type").and_then(|v| v.as_str()).unwrap_or("");

        match type_str {
            "PHOTO" => Ok(Attach::Photo(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            "VIDEO" => Ok(Attach::Video(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            "FILE" => Ok(Attach::File(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            "CONTROL" => Ok(Attach::Control(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            "STICKER" => Ok(Attach::Sticker(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            "AUDIO" => Ok(Attach::Audio(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            "CONTACT" => Ok(Attach::Contact(
                serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            )),
            _ => Ok(Attach::Unknown),
        }
    }
}

impl Attach {
    pub fn attach_type(&self) -> AttachType {
        match self {
            Attach::Photo(_) => AttachType::Photo,
            Attach::Video(_) => AttachType::Video,
            Attach::File(_) => AttachType::File,
            Attach::Control(_) => AttachType::Control,
            Attach::Sticker(_) => AttachType::Sticker,
            Attach::Audio(_) => AttachType::Audio,
            Attach::Contact(_) => AttachType::Contact,
            Attach::Unknown => AttachType::File,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    #[serde(rename = "type")]
    pub type_: String,
    pub length: i32,
    #[serde(rename = "from")]
    pub from_: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionCounter {
    pub count: i32,
    pub reaction: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionInfo {
    #[serde(default)]
    pub total_count: i32,
    #[serde(default)]
    pub counters: Vec<ReactionCounter>,
    pub your_reaction: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageLink {
    #[serde(deserialize_with = "de_helpers::i64_or_str")]
    pub chat_id: i64,
    pub message: Box<Message>,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(default, deserialize_with = "de_helpers::opt_i64_or_str")]
    pub chat_id: Option<i64>,
    #[serde(default, deserialize_with = "de_helpers::opt_i64_or_str")]
    pub sender: Option<i64>,
    #[serde(default)]
    pub elements: Vec<Element>,
    pub reaction_info: Option<ReactionInfo>,
    pub options: Option<i64>,
    #[serde(default, deserialize_with = "de_helpers::i64_or_str")]
    pub id: i64,
    #[serde(default, deserialize_with = "de_helpers::i64_or_str")]
    pub time: i64,
    pub link: Option<Box<MessageLink>>,
    #[serde(default)]
    pub text: String,
    pub status: Option<MessageStatus>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub attaches: Vec<Attach>,
}

impl Message {
    pub fn from_value(data: &Value) -> Option<Self> {
        let msg_data = if let Some(msg) = data.get("message") {
            msg
        } else {
            data
        };

        let mut msg: Message = serde_json::from_value(msg_data.clone()).ok()?;

        if msg.chat_id.is_none() {
            msg.chat_id = data.get("chatId").and_then(|v| v.as_i64());
        }

        Some(msg)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dialog {
    pub cid: Option<i64>,
    pub owner: i64,
    pub has_bots: Option<bool>,
    pub join_time: i64,
    pub created: i64,
    pub last_message: Option<Message>,
    #[serde(rename = "type")]
    pub type_: ChatType,
    #[serde(default)]
    pub last_fire_delayed_error_time: i64,
    #[serde(default)]
    pub last_delayed_update_time: i64,
    pub prev_message_id: Option<String>,
    #[serde(default)]
    pub options: HashMap<String, Value>,
    pub modified: i64,
    pub last_event_time: i64,
    pub id: i64,
    pub status: String,
    #[serde(default)]
    pub participants: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    #[serde(default)]
    pub participants_count: i64,
    pub access: Option<AccessType>,
    pub invited_by: Option<i64>,
    pub link: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<ChatType>,
    pub title: Option<String>,
    #[serde(default)]
    pub last_fire_delayed_error_time: i64,
    #[serde(default)]
    pub last_delayed_update_time: i64,
    #[serde(default)]
    pub options: HashMap<String, Value>,
    pub base_raw_icon_url: Option<String>,
    pub base_icon_url: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub modified: i64,
    pub id: i64,
    #[serde(default)]
    pub admin_participants: HashMap<String, Value>,
    #[serde(default)]
    pub participants: HashMap<String, Value>,
    #[serde(default)]
    pub owner: i64,
    #[serde(default)]
    pub join_time: i64,
    #[serde(default)]
    pub created: i64,
    pub last_message: Option<Message>,
    pub prev_message_id: Option<String>,
    #[serde(default)]
    pub last_event_time: i64,
    #[serde(default)]
    pub messages_count: i64,
    #[serde(default)]
    pub admins: Vec<i64>,
    pub restrictions: Option<i64>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub cid: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel(pub Chat);

impl std::ops::Deref for Channel {
    type Target = Chat;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Channel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    #[serde(deserialize_with = "de_helpers::i64_or_str")]
    pub id: i64,
    #[serde(deserialize_with = "de_helpers::i64_or_str")]
    pub account_status: i64,
    pub phone: String,
    pub names: Vec<Names>,
    #[serde(deserialize_with = "de_helpers::i64_or_str")]
    pub update_time: i64,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub account_status: i64,
    pub update_time: i64,
    pub id: i64,
    #[serde(default)]
    pub names: Vec<Names>,
    pub options: Option<Vec<String>>,
    pub base_url: Option<String>,
    pub base_raw_url: Option<String>,
    pub photo_id: Option<i64>,
    pub description: Option<String>,
    pub gender: Option<i64>,
    pub link: Option<String>,
    pub web_app: Option<String>,
    pub menu_button: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub client: String,
    pub info: String,
    pub location: String,
    pub time: i64,
    pub current: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRequest {
    #[serde(rename = "unsafe")]
    pub unsafe_: bool,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoRequest {
    #[serde(rename = "EXTERNAL")]
    pub external: String,
    pub cache: bool,
    pub url: String,
}

impl VideoRequest {
    pub fn from_value(data: &Value) -> Option<Self> {
        let map = data.as_object()?;
        let external = map.get("EXTERNAL")?.as_str()?.to_string();
        let cache = map.get("cache")?.as_bool()?;
        let url = map
            .iter()
            .filter(|(k, _)| k.as_str() != "EXTERNAL" && k.as_str() != "cache")
            .next()
            .and_then(|(_, v)| v.as_str())?
            .to_string();
        Some(VideoRequest {
            external,
            cache,
            url,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadState {
    pub unread: i64,
    pub mark: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    #[serde(default)]
    pub source_id: i64,
    #[serde(default)]
    pub include: Vec<i64>,
    #[serde(default)]
    pub options: Vec<Value>,
    #[serde(default)]
    pub update_time: i64,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub filters: Vec<Value>,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FolderUpdate {
    pub folders_order: Option<Vec<String>>,
    pub folder: Option<Folder>,
    pub folder_sync: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FolderList {
    #[serde(default)]
    pub folders_order: Vec<String>,
    #[serde(default)]
    pub folders: Vec<Folder>,
    pub folder_sync: Option<i64>,
    #[serde(default)]
    pub all_filter_exclude_folders: Vec<Value>,
}
