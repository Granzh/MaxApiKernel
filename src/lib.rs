// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

pub mod client;
pub mod constants;
pub mod db;
pub mod enums;
pub mod errors;
pub mod files;
pub mod filters;
pub mod formatting;
pub mod payloads;
pub mod transport;
pub mod types;

pub use client::message::AttachmentKind;
pub use client::MaxClient;
pub use enums::{
    AccessType, AttachType, AuthType, Capability, ChatType, ContactAction, DeviceType,
    FormattingType, MessageStatus, MessageType, Opcode, ReadAction,
};
pub use errors::{MaxError, MaxResult};
pub use files::{File, Photo, Video};
pub use filters::{Filters, MessageFilter};
pub use types::{
    Attach, AudioAttach, Channel, Chat, Contact, ContactAttach, ControlAttach, Dialog, Element,
    FileAttach, FileRequest, Folder, FolderList, FolderUpdate, Me, Member, Message, MessageLink,
    Name, Names, PhotoAttach, Presence, ReactionCounter, ReactionInfo, ReadState, Session,
    StickerAttach, User, VideoAttach, VideoRequest,
};
