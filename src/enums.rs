// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum Opcode {
    Ping,
    Debug,
    Reconnect,
    Log,
    SessionInit,
    Profile,
    AuthRequest,
    Auth,
    Login,
    Logout,
    Sync,
    Config,
    AuthConfirm,
    PresetAvatars,
    AssetsGet,
    AssetsUpdate,
    AssetsGetByIds,
    AssetsAdd,
    SearchFeedback,
    ContactInfo,
    ContactAdd,
    ContactUpdate,
    ContactPresence,
    ContactList,
    ContactSearch,
    ContactMutual,
    ContactPhotos,
    ContactSort,
    ContactVerify,
    RemoveContactPhoto,
    ContactInfoByPhone,
    ChatInfo,
    ChatHistory,
    ChatMark,
    ChatMedia,
    ChatDelete,
    ChatsList,
    ChatClear,
    ChatUpdate,
    ChatCheckLink,
    ChatJoin,
    ChatLeave,
    ChatMembers,
    PublicSearch,
    ChatClose,
    ChatCreate,
    MsgSend,
    MsgTyping,
    MsgDelete,
    MsgEdit,
    ChatSearch,
    MsgSharePreview,
    MsgGet,
    MsgSearchTouch,
    MsgSearch,
    MsgGetStat,
    ChatSubscribe,
    VideoChatStart,
    ChatMembersUpdate,
    VideoChatHistory,
    PhotoUpload,
    StickerUpload,
    VideoUpload,
    VideoPlay,
    ChatPinSetVisibility,
    FileUpload,
    FileDownload,
    LinkInfo,
    MsgDeleteRange,
    SessionsInfo,
    SessionsClose,
    PhoneBindRequest,
    PhoneBindConfirm,
    ConfirmPresent,
    GetInboundCalls,
    ExternalCallback,
    AuthValidatePassword,
    AuthValidateHint,
    AuthVerifyEmail,
    AuthCheckEmail,
    AuthSet2fa,
    AuthCreateTrack,
    AuthLoginCheckPassword,
    ChatComplain,
    MsgSendCallback,
    SuspendBot,
    LocationStop,
    LocationSend,
    LocationRequest,
    GetLastMentions,
    NotifMessage,
    NotifTyping,
    NotifMark,
    NotifContact,
    NotifPresence,
    NotifConfig,
    NotifChat,
    NotifAttach,
    NotifCallStart,
    NotifContactSort,
    NotifMsgDeleteRange,
    NotifMsgDelete,
    NotifCallbackAnswer,
    ChatBotCommands,
    BotInfo,
    NotifLocation,
    NotifLocationRequest,
    NotifAssetsUpdate,
    NotifDraft,
    NotifDraftDiscard,
    NotifMsgDelayed,
    NotifMsgReactionsChanged,
    NotifMsgYouReacted,
    CallsToken,
    NotifProfile,
    WebAppInitData,
    DraftSave,
    DraftDiscard,
    MsgReaction,
    MsgCancelReaction,
    MsgGetReactions,
    MsgGetDetailedReactions,
    StickerCreate,
    StickerSuggest,
    VideoChatMembers,
    ChatHide,
    ChatSearchCommonParticipants,
    ProfileDelete,
    ProfileDeleteTime,
    AssetsRemove,
    AssetsMove,
    AssetsListModify,
    FoldersGet,
    FoldersGetById,
    FoldersUpdate,
    FoldersReorder,
    FoldersDelete,
    NotifFolders,
    GetQr,
    GetQrStatus,
    LoginByQr,
    Unknown(i32),
}

impl Opcode {
    pub fn value(self) -> i32 {
        i32::from(self)
    }

    pub fn is_notification(self) -> bool {
        matches!(
            self,
            Opcode::NotifMessage
                | Opcode::NotifTyping
                | Opcode::NotifMark
                | Opcode::NotifContact
                | Opcode::NotifPresence
                | Opcode::NotifConfig
                | Opcode::NotifChat
                | Opcode::NotifAttach
                | Opcode::NotifCallStart
                | Opcode::NotifContactSort
                | Opcode::NotifMsgDeleteRange
                | Opcode::NotifMsgDelete
                | Opcode::NotifCallbackAnswer
                | Opcode::NotifLocation
                | Opcode::NotifLocationRequest
                | Opcode::NotifAssetsUpdate
                | Opcode::NotifDraft
                | Opcode::NotifDraftDiscard
                | Opcode::NotifMsgDelayed
                | Opcode::NotifMsgReactionsChanged
                | Opcode::NotifMsgYouReacted
                | Opcode::NotifProfile
                | Opcode::NotifFolders
        )
    }
}

impl From<i32> for Opcode {
    fn from(v: i32) -> Self {
        match v {
            1 => Opcode::Ping,
            2 => Opcode::Debug,
            3 => Opcode::Reconnect,
            5 => Opcode::Log,
            6 => Opcode::SessionInit,
            16 => Opcode::Profile,
            17 => Opcode::AuthRequest,
            18 => Opcode::Auth,
            19 => Opcode::Login,
            20 => Opcode::Logout,
            21 => Opcode::Sync,
            22 => Opcode::Config,
            23 => Opcode::AuthConfirm,
            25 => Opcode::PresetAvatars,
            26 => Opcode::AssetsGet,
            27 => Opcode::AssetsUpdate,
            28 => Opcode::AssetsGetByIds,
            29 => Opcode::AssetsAdd,
            31 => Opcode::SearchFeedback,
            32 => Opcode::ContactInfo,
            33 => Opcode::ContactAdd,
            34 => Opcode::ContactUpdate,
            35 => Opcode::ContactPresence,
            36 => Opcode::ContactList,
            37 => Opcode::ContactSearch,
            38 => Opcode::ContactMutual,
            39 => Opcode::ContactPhotos,
            40 => Opcode::ContactSort,
            42 => Opcode::ContactVerify,
            43 => Opcode::RemoveContactPhoto,
            46 => Opcode::ContactInfoByPhone,
            48 => Opcode::ChatInfo,
            49 => Opcode::ChatHistory,
            50 => Opcode::ChatMark,
            51 => Opcode::ChatMedia,
            52 => Opcode::ChatDelete,
            53 => Opcode::ChatsList,
            54 => Opcode::ChatClear,
            55 => Opcode::ChatUpdate,
            56 => Opcode::ChatCheckLink,
            57 => Opcode::ChatJoin,
            58 => Opcode::ChatLeave,
            59 => Opcode::ChatMembers,
            60 => Opcode::PublicSearch,
            61 => Opcode::ChatClose,
            63 => Opcode::ChatCreate,
            64 => Opcode::MsgSend,
            65 => Opcode::MsgTyping,
            66 => Opcode::MsgDelete,
            67 => Opcode::MsgEdit,
            68 => Opcode::ChatSearch,
            70 => Opcode::MsgSharePreview,
            71 => Opcode::MsgGet,
            72 => Opcode::MsgSearchTouch,
            73 => Opcode::MsgSearch,
            74 => Opcode::MsgGetStat,
            75 => Opcode::ChatSubscribe,
            76 => Opcode::VideoChatStart,
            77 => Opcode::ChatMembersUpdate,
            79 => Opcode::VideoChatHistory,
            80 => Opcode::PhotoUpload,
            81 => Opcode::StickerUpload,
            82 => Opcode::VideoUpload,
            83 => Opcode::VideoPlay,
            86 => Opcode::ChatPinSetVisibility,
            87 => Opcode::FileUpload,
            88 => Opcode::FileDownload,
            89 => Opcode::LinkInfo,
            92 => Opcode::MsgDeleteRange,
            96 => Opcode::SessionsInfo,
            97 => Opcode::SessionsClose,
            98 => Opcode::PhoneBindRequest,
            99 => Opcode::PhoneBindConfirm,
            101 => Opcode::ConfirmPresent,
            103 => Opcode::GetInboundCalls,
            105 => Opcode::ExternalCallback,
            107 => Opcode::AuthValidatePassword,
            108 => Opcode::AuthValidateHint,
            109 => Opcode::AuthVerifyEmail,
            110 => Opcode::AuthCheckEmail,
            111 => Opcode::AuthSet2fa,
            112 => Opcode::AuthCreateTrack,
            115 => Opcode::AuthLoginCheckPassword,
            117 => Opcode::ChatComplain,
            118 => Opcode::MsgSendCallback,
            119 => Opcode::SuspendBot,
            124 => Opcode::LocationStop,
            125 => Opcode::LocationSend,
            126 => Opcode::LocationRequest,
            127 => Opcode::GetLastMentions,
            128 => Opcode::NotifMessage,
            129 => Opcode::NotifTyping,
            130 => Opcode::NotifMark,
            131 => Opcode::NotifContact,
            132 => Opcode::NotifPresence,
            134 => Opcode::NotifConfig,
            135 => Opcode::NotifChat,
            136 => Opcode::NotifAttach,
            137 => Opcode::NotifCallStart,
            139 => Opcode::NotifContactSort,
            140 => Opcode::NotifMsgDeleteRange,
            142 => Opcode::NotifMsgDelete,
            143 => Opcode::NotifCallbackAnswer,
            144 => Opcode::ChatBotCommands,
            145 => Opcode::BotInfo,
            147 => Opcode::NotifLocation,
            148 => Opcode::NotifLocationRequest,
            150 => Opcode::NotifAssetsUpdate,
            152 => Opcode::NotifDraft,
            153 => Opcode::NotifDraftDiscard,
            154 => Opcode::NotifMsgDelayed,
            155 => Opcode::NotifMsgReactionsChanged,
            156 => Opcode::NotifMsgYouReacted,
            158 => Opcode::CallsToken,
            159 => Opcode::NotifProfile,
            160 => Opcode::WebAppInitData,
            176 => Opcode::DraftSave,
            177 => Opcode::DraftDiscard,
            178 => Opcode::MsgReaction,
            179 => Opcode::MsgCancelReaction,
            180 => Opcode::MsgGetReactions,
            181 => Opcode::MsgGetDetailedReactions,
            193 => Opcode::StickerCreate,
            194 => Opcode::StickerSuggest,
            195 => Opcode::VideoChatMembers,
            196 => Opcode::ChatHide,
            198 => Opcode::ChatSearchCommonParticipants,
            199 => Opcode::ProfileDelete,
            200 => Opcode::ProfileDeleteTime,
            259 => Opcode::AssetsRemove,
            260 => Opcode::AssetsMove,
            261 => Opcode::AssetsListModify,
            272 => Opcode::FoldersGet,
            273 => Opcode::FoldersGetById,
            274 => Opcode::FoldersUpdate,
            275 => Opcode::FoldersReorder,
            276 => Opcode::FoldersDelete,
            277 => Opcode::NotifFolders,
            288 => Opcode::GetQr,
            289 => Opcode::GetQrStatus,
            291 => Opcode::LoginByQr,
            other => Opcode::Unknown(other),
        }
    }
}

impl From<Opcode> for i32 {
    fn from(op: Opcode) -> i32 {
        match op {
            Opcode::Ping => 1,
            Opcode::Debug => 2,
            Opcode::Reconnect => 3,
            Opcode::Log => 5,
            Opcode::SessionInit => 6,
            Opcode::Profile => 16,
            Opcode::AuthRequest => 17,
            Opcode::Auth => 18,
            Opcode::Login => 19,
            Opcode::Logout => 20,
            Opcode::Sync => 21,
            Opcode::Config => 22,
            Opcode::AuthConfirm => 23,
            Opcode::PresetAvatars => 25,
            Opcode::AssetsGet => 26,
            Opcode::AssetsUpdate => 27,
            Opcode::AssetsGetByIds => 28,
            Opcode::AssetsAdd => 29,
            Opcode::SearchFeedback => 31,
            Opcode::ContactInfo => 32,
            Opcode::ContactAdd => 33,
            Opcode::ContactUpdate => 34,
            Opcode::ContactPresence => 35,
            Opcode::ContactList => 36,
            Opcode::ContactSearch => 37,
            Opcode::ContactMutual => 38,
            Opcode::ContactPhotos => 39,
            Opcode::ContactSort => 40,
            Opcode::ContactVerify => 42,
            Opcode::RemoveContactPhoto => 43,
            Opcode::ContactInfoByPhone => 46,
            Opcode::ChatInfo => 48,
            Opcode::ChatHistory => 49,
            Opcode::ChatMark => 50,
            Opcode::ChatMedia => 51,
            Opcode::ChatDelete => 52,
            Opcode::ChatsList => 53,
            Opcode::ChatClear => 54,
            Opcode::ChatUpdate => 55,
            Opcode::ChatCheckLink => 56,
            Opcode::ChatJoin => 57,
            Opcode::ChatLeave => 58,
            Opcode::ChatMembers => 59,
            Opcode::PublicSearch => 60,
            Opcode::ChatClose => 61,
            Opcode::ChatCreate => 63,
            Opcode::MsgSend => 64,
            Opcode::MsgTyping => 65,
            Opcode::MsgDelete => 66,
            Opcode::MsgEdit => 67,
            Opcode::ChatSearch => 68,
            Opcode::MsgSharePreview => 70,
            Opcode::MsgGet => 71,
            Opcode::MsgSearchTouch => 72,
            Opcode::MsgSearch => 73,
            Opcode::MsgGetStat => 74,
            Opcode::ChatSubscribe => 75,
            Opcode::VideoChatStart => 76,
            Opcode::ChatMembersUpdate => 77,
            Opcode::VideoChatHistory => 79,
            Opcode::PhotoUpload => 80,
            Opcode::StickerUpload => 81,
            Opcode::VideoUpload => 82,
            Opcode::VideoPlay => 83,
            Opcode::ChatPinSetVisibility => 86,
            Opcode::FileUpload => 87,
            Opcode::FileDownload => 88,
            Opcode::LinkInfo => 89,
            Opcode::MsgDeleteRange => 92,
            Opcode::SessionsInfo => 96,
            Opcode::SessionsClose => 97,
            Opcode::PhoneBindRequest => 98,
            Opcode::PhoneBindConfirm => 99,
            Opcode::ConfirmPresent => 101,
            Opcode::GetInboundCalls => 103,
            Opcode::ExternalCallback => 105,
            Opcode::AuthValidatePassword => 107,
            Opcode::AuthValidateHint => 108,
            Opcode::AuthVerifyEmail => 109,
            Opcode::AuthCheckEmail => 110,
            Opcode::AuthSet2fa => 111,
            Opcode::AuthCreateTrack => 112,
            Opcode::AuthLoginCheckPassword => 115,
            Opcode::ChatComplain => 117,
            Opcode::MsgSendCallback => 118,
            Opcode::SuspendBot => 119,
            Opcode::LocationStop => 124,
            Opcode::LocationSend => 125,
            Opcode::LocationRequest => 126,
            Opcode::GetLastMentions => 127,
            Opcode::NotifMessage => 128,
            Opcode::NotifTyping => 129,
            Opcode::NotifMark => 130,
            Opcode::NotifContact => 131,
            Opcode::NotifPresence => 132,
            Opcode::NotifConfig => 134,
            Opcode::NotifChat => 135,
            Opcode::NotifAttach => 136,
            Opcode::NotifCallStart => 137,
            Opcode::NotifContactSort => 139,
            Opcode::NotifMsgDeleteRange => 140,
            Opcode::NotifMsgDelete => 142,
            Opcode::NotifCallbackAnswer => 143,
            Opcode::ChatBotCommands => 144,
            Opcode::BotInfo => 145,
            Opcode::NotifLocation => 147,
            Opcode::NotifLocationRequest => 148,
            Opcode::NotifAssetsUpdate => 150,
            Opcode::NotifDraft => 152,
            Opcode::NotifDraftDiscard => 153,
            Opcode::NotifMsgDelayed => 154,
            Opcode::NotifMsgReactionsChanged => 155,
            Opcode::NotifMsgYouReacted => 156,
            Opcode::CallsToken => 158,
            Opcode::NotifProfile => 159,
            Opcode::WebAppInitData => 160,
            Opcode::DraftSave => 176,
            Opcode::DraftDiscard => 177,
            Opcode::MsgReaction => 178,
            Opcode::MsgCancelReaction => 179,
            Opcode::MsgGetReactions => 180,
            Opcode::MsgGetDetailedReactions => 181,
            Opcode::StickerCreate => 193,
            Opcode::StickerSuggest => 194,
            Opcode::VideoChatMembers => 195,
            Opcode::ChatHide => 196,
            Opcode::ChatSearchCommonParticipants => 198,
            Opcode::ProfileDelete => 199,
            Opcode::ProfileDeleteTime => 200,
            Opcode::AssetsRemove => 259,
            Opcode::AssetsMove => 260,
            Opcode::AssetsListModify => 261,
            Opcode::FoldersGet => 272,
            Opcode::FoldersGetById => 273,
            Opcode::FoldersUpdate => 274,
            Opcode::FoldersReorder => 275,
            Opcode::FoldersDelete => 276,
            Opcode::NotifFolders => 277,
            Opcode::GetQr => 288,
            Opcode::GetQrStatus => 289,
            Opcode::LoginByQr => 291,
            Opcode::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChatType {
    Dialog,
    Chat,
    Channel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    Text,
    System,
    Service,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageStatus {
    Edited,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthType {
    StartAuth,
    CheckCode,
    Register,
    Resend,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessType {
    Public,
    Private,
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceType {
    Web,
    Android,
    Ios,
    Desktop,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Web => "WEB",
            DeviceType::Android => "ANDROID",
            DeviceType::Ios => "IOS",
            DeviceType::Desktop => "DESKTOP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttachType {
    Photo,
    Video,
    File,
    Sticker,
    Audio,
    Control,
    Contact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormattingType {
    Strong,
    Emphasized,
    Underline,
    Strikethrough,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContactAction {
    Add,
    Remove,
}

impl ContactAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContactAction::Add => "ADD",
            ContactAction::Remove => "REMOVE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReadAction {
    ReadMessage,
    ReadReaction,
}

impl ReadAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadAction::ReadMessage => "READ_MESSAGE",
            ReadAction::ReadReaction => "READ_REACTION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "0")]
    Default,
    #[serde(rename = "1")]
    EsiaVerifiedFlag,
    #[serde(rename = "2")]
    SecondFactorPasswordEnabled,
    #[serde(rename = "3")]
    SecondFactorHasEmail,
    #[serde(rename = "4")]
    SecondFactorHasHint,
}

impl From<Capability> for i32 {
    fn from(c: Capability) -> i32 {
        match c {
            Capability::Default => 0,
            Capability::EsiaVerifiedFlag => 1,
            Capability::SecondFactorPasswordEnabled => 2,
            Capability::SecondFactorHasEmail => 3,
            Capability::SecondFactorHasHint => 4,
        }
    }
}
