use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::constants::*;
use crate::enums::{AuthType, ContactAction, ReadAction};

fn to_camel(field: &str) -> String {
    let mut parts = field.split('_');
    let first = parts.next().unwrap_or("").to_string();
    let rest: String = parts
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();
    first + &rest
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseWebSocketMessage {
    pub ver: i32,
    pub cmd: i32,
    pub seq: i32,
    pub opcode: i32,
    pub payload: Value,
}

impl BaseWebSocketMessage {
    pub fn new(ver: i32, cmd: i32, seq: i32, opcode: i32, payload: Value) -> Self {
        Self {
            ver,
            cmd,
            seq,
            opcode,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentPayload {
    pub device_type: String,
    pub locale: String,
    pub device_locale: String,
    pub os_version: String,
    pub device_name: String,
    pub header_user_agent: String,
    pub app_version: String,
    pub screen: String,
    pub timezone: String,
    pub client_session_id: i32,
    pub build_number: i32,
}

impl Default for UserAgentPayload {
    fn default() -> Self {
        Self {
            device_type: "WEB".to_string(),
            locale: DEFAULT_LOCALE.to_string(),
            device_locale: DEFAULT_DEVICE_LOCALE.to_string(),
            os_version: random_os_version().to_string(),
            device_name: random_device_name().to_string(),
            header_user_agent: random_user_agent().to_string(),
            app_version: DEFAULT_APP_VERSION.to_string(),
            screen: DEFAULT_SCREEN.to_string(),
            timezone: random_timezone().to_string(),
            client_session_id: random_client_session_id(),
            build_number: DEFAULT_BUILD_NUMBER,
        }
    }
}

impl UserAgentPayload {
    pub fn for_web() -> Self {
        Self {
            device_type: "WEB".to_string(),
            ..Default::default()
        }
    }

    pub fn for_desktop() -> Self {
        Self {
            device_type: "DESKTOP".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestCodePayload {
    pub phone: String,
    #[serde(rename = "type")]
    pub type_: AuthType,
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCodePayload {
    pub token: String,
    pub verify_code: String,
    pub auth_token_type: AuthType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPayload {
    pub interactive: bool,
    pub token: String,
    pub chats_sync: i64,
    pub contacts_sync: i64,
    pub presence_sync: i64,
    pub drafts_sync: i64,
    pub chats_count: i32,
    pub user_agent: UserAgentPayload,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyLink {
    #[serde(rename = "type")]
    pub type_: String,
    pub message_id: String,
}

impl ReplyLink {
    pub fn reply(message_id: String) -> Self {
        Self {
            type_: "REPLY".to_string(),
            message_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPayload {
    pub count: i32,
    pub profile: bool,
}

impl Default for UploadPayload {
    fn default() -> Self {
        Self {
            count: 1,
            profile: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachPhotoPayload {
    #[serde(rename = "_type")]
    pub type_: String,
    pub photo_token: String,
}

impl AttachPhotoPayload {
    pub fn new(photo_token: String) -> Self {
        Self {
            type_: "PHOTO".to_string(),
            photo_token,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoAttachPayload {
    #[serde(rename = "_type")]
    pub type_: String,
    pub video_id: i64,
    pub token: String,
}

impl VideoAttachPayload {
    pub fn new(video_id: i64, token: String) -> Self {
        Self {
            type_: "VIDEO".to_string(),
            video_id,
            token,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachFilePayload {
    #[serde(rename = "_type")]
    pub type_: String,
    pub file_id: i64,
}

impl AttachFilePayload {
    pub fn new(file_id: i64) -> Self {
        Self {
            type_: "FILE".to_string(),
            file_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageElement {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "from")]
    pub from_: i32,
    pub length: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessagePayloadMessage {
    pub text: String,
    pub cid: i64,
    pub elements: Vec<MessageElement>,
    pub attaches: Vec<Value>,
    pub link: Option<ReplyLink>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessagePayload {
    pub chat_id: i64,
    pub message: SendMessagePayloadMessage,
    pub notify: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditMessagePayload {
    pub chat_id: i64,
    pub message_id: i64,
    pub text: String,
    pub elements: Vec<MessageElement>,
    pub attaches: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessagePayload {
    pub chat_id: i64,
    pub message_ids: Vec<i64>,
    pub for_me: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchContactsPayload {
    pub contact_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FetchHistoryPayload {
    #[serde(rename = "chatId")]
    pub chat_id: i64,
    #[serde(rename = "from")]
    pub from_time: i64,
    pub forward: i32,
    pub backward: i32,
    #[serde(rename = "getMessages")]
    pub get_messages: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeProfilePayload {
    pub first_name: String,
    pub last_name: Option<String>,
    pub description: Option<String>,
    pub photo_token: Option<String>,
    pub avatar_type: String,
}

impl ChangeProfilePayload {
    pub fn new(first_name: String) -> Self {
        Self {
            first_name,
            last_name: None,
            description: None,
            photo_token: None,
            avatar_type: "USER_AVATAR".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveLinkPayload {
    pub link: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinMessagePayload {
    pub chat_id: i64,
    pub notify_pin: bool,
    pub pin_message_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateGroupAttach {
    #[serde(rename = "_type")]
    pub type_: String,
    pub event: String,
    #[serde(rename = "chatType")]
    pub chat_type: String,
    pub title: String,
    #[serde(rename = "userIds")]
    pub user_ids: Vec<i64>,
}

impl CreateGroupAttach {
    pub fn new(title: String, user_ids: Vec<i64>) -> Self {
        Self {
            type_: "CONTROL".to_string(),
            event: "new".to_string(),
            chat_type: "CHAT".to_string(),
            title,
            user_ids,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupMessage {
    pub cid: i64,
    pub attaches: Vec<CreateGroupAttach>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupPayload {
    pub message: CreateGroupMessage,
    pub notify: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteUsersPayload {
    pub chat_id: i64,
    pub user_ids: Vec<i64>,
    pub show_history: bool,
    pub operation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveUsersPayload {
    pub chat_id: i64,
    pub user_ids: Vec<i64>,
    pub operation: String,
    pub clean_msg_period: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ChangeGroupSettingsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_owner_can_change_icon_title: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_can_pin_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_admin_can_add_member: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_admin_can_call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members_can_see_private_link: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeGroupSettingsPayload {
    pub chat_id: i64,
    pub options: ChangeGroupSettingsOptions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeGroupProfilePayload {
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGroupMembersPayload {
    #[serde(rename = "type")]
    pub type_: String,
    pub marker: Option<i64>,
    pub chat_id: i64,
    pub count: i32,
}

impl GetGroupMembersPayload {
    pub fn new(chat_id: i64, count: i32, marker: Option<i64>) -> Self {
        Self {
            type_: "MEMBER".to_string(),
            marker,
            chat_id,
            count,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchGroupMembersPayload {
    #[serde(rename = "type")]
    pub type_: String,
    pub query: String,
    pub chat_id: i64,
}

impl SearchGroupMembersPayload {
    pub fn new(chat_id: i64, query: String) -> Self {
        Self {
            type_: "MEMBER".to_string(),
            query,
            chat_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVideoPayload {
    pub chat_id: i64,
    pub message_id: Value,
    pub video_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFilePayload {
    pub chat_id: i64,
    pub message_id: Value,
    pub file_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchByPhonePayload {
    pub phone: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinChatPayload {
    pub link: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionInfoPayload {
    pub reaction_type: String,
    pub id: String,
}

impl ReactionInfoPayload {
    pub fn emoji(emoji: String) -> Self {
        Self {
            reaction_type: "EMOJI".to_string(),
            id: emoji,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddReactionPayload {
    pub chat_id: i64,
    pub message_id: String,
    pub reaction: ReactionInfoPayload,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReactionsPayload {
    pub chat_id: i64,
    pub message_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveReactionPayload {
    pub chat_id: i64,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReworkInviteLinkPayload {
    pub revoke_private_link: bool,
    pub chat_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactActionPayload {
    pub contact_id: i64,
    pub action: ContactAction,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPayload {
    pub last_name: Option<String>,
    pub first_name: String,
    pub token: String,
    pub token_type: AuthType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderPayload {
    pub id: String,
    pub title: String,
    pub include: Vec<i64>,
    pub filters: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatInfoPayload {
    pub chat_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFolderPayload {
    pub folder_sync: i64,
}

impl Default for GetFolderPayload {
    fn default() -> Self {
        Self { folder_sync: 0 }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolderPayload {
    pub id: String,
    pub title: String,
    pub include: Vec<i64>,
    #[serde(default)]
    pub filters: Vec<Value>,
    #[serde(default)]
    pub options: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFolderPayload {
    pub folder_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveChatPayload {
    pub chat_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchChatsPayload {
    pub marker: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessagesPayload {
    #[serde(rename = "type")]
    pub type_: ReadAction,
    pub chat_id: i64,
    pub message_id: String,
    pub mark: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckPasswordChallengePayload {
    pub track_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTrackPayload {
    #[serde(rename = "type")]
    pub type_: i32,
}

impl Default for CreateTrackPayload {
    fn default() -> Self {
        Self { type_: 0 }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPasswordPayload {
    pub track_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetHintPayload {
    pub track_id: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTwoFactorPayload {
    pub expected_capabilities: Vec<i32>,
    pub track_id: String,
    pub password: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEmailCodePayload {
    pub track_id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailCodePayload {
    pub track_id: String,
    pub verify_code: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsClosePayload {
    pub session_ids: Vec<i64>,
}
