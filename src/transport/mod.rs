// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

pub mod socket;
pub mod websocket;

use crate::enums::Opcode;
use crate::payloads::BaseWebSocketMessage;
use crate::types::{Chat, Message, ReactionCounter, ReactionInfo};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex, RwLock};

pub type BoxFuture = Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>;
pub type MessageHandlerFn = Arc<dyn Fn(Message) -> BoxFuture + Send + Sync>;
pub type ChatHandlerFn = Arc<dyn Fn(Chat) -> BoxFuture + Send + Sync>;
pub type RawHandlerFn = Arc<dyn Fn(Value) -> BoxFuture + Send + Sync>;
pub type ReactionHandlerFn = Arc<dyn Fn(String, i64, ReactionInfo) -> BoxFuture + Send + Sync>;
pub type SimpleHandlerFn = Arc<dyn Fn() -> BoxFuture + Send + Sync>;

use crate::filters::MessageFilter;

pub struct FilteredMessageHandler {
    pub handler: MessageHandlerFn,
    pub filter: Option<Arc<dyn MessageFilter>>,
}

pub struct ClientState {
    pub is_connected: AtomicBool,
    pub seq: AtomicI32,

    pub pending: Mutex<HashMap<i32, oneshot::Sender<Value>>>,
    pub file_upload_waiters: Mutex<HashMap<i64, oneshot::Sender<Value>>>,

    pub on_message_handlers: RwLock<Vec<FilteredMessageHandler>>,
    pub on_message_edit_handlers: RwLock<Vec<FilteredMessageHandler>>,
    pub on_message_delete_handlers: RwLock<Vec<FilteredMessageHandler>>,
    pub on_reaction_handlers: RwLock<Vec<ReactionHandlerFn>>,
    pub on_chat_update_handlers: RwLock<Vec<ChatHandlerFn>>,
    pub on_raw_receive_handlers: RwLock<Vec<RawHandlerFn>>,
    pub on_start_handler: RwLock<Option<SimpleHandlerFn>>,
    pub scheduled_tasks: RwLock<Vec<(SimpleHandlerFn, Duration)>>,
}

impl ClientState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            is_connected: AtomicBool::new(false),
            seq: AtomicI32::new(0),
            pending: Mutex::new(HashMap::new()),
            file_upload_waiters: Mutex::new(HashMap::new()),
            on_message_handlers: RwLock::new(Vec::new()),
            on_message_edit_handlers: RwLock::new(Vec::new()),
            on_message_delete_handlers: RwLock::new(Vec::new()),
            on_reaction_handlers: RwLock::new(Vec::new()),
            on_chat_update_handlers: RwLock::new(Vec::new()),
            on_raw_receive_handlers: RwLock::new(Vec::new()),
            on_start_handler: RwLock::new(None),
            scheduled_tasks: RwLock::new(Vec::new()),
        })
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            is_connected: AtomicBool::new(false),
            seq: AtomicI32::new(0),
            pending: Mutex::new(HashMap::new()),
            file_upload_waiters: Mutex::new(HashMap::new()),
            on_message_handlers: RwLock::new(Vec::new()),
            on_message_edit_handlers: RwLock::new(Vec::new()),
            on_message_delete_handlers: RwLock::new(Vec::new()),
            on_reaction_handlers: RwLock::new(Vec::new()),
            on_chat_update_handlers: RwLock::new(Vec::new()),
            on_raw_receive_handlers: RwLock::new(Vec::new()),
            on_start_handler: RwLock::new(None),
            scheduled_tasks: RwLock::new(Vec::new()),
        }
    }
}

pub fn make_message(opcode: Opcode, payload: Value, cmd: i32, seq: i32) -> Value {
    serde_json::to_value(BaseWebSocketMessage {
        ver: 11,
        cmd,
        seq,
        opcode: opcode.value(),
        payload,
    })
    .unwrap_or_default()
}

pub async fn dispatch_incoming(state: &Arc<ClientState>, data: Value) {
    let opcode_val = data.get("opcode").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let opcode = Opcode::from(opcode_val);

    dispatch_raw_receive(state, data.clone()).await;
    dispatch_file_upload(state, &data).await;
    dispatch_message_notification(state, opcode, &data).await;
    dispatch_reaction_change(state, opcode, &data).await;
    dispatch_chat_update(state, opcode, &data).await;
}

async fn dispatch_raw_receive(state: &Arc<ClientState>, data: Value) {
    let handlers = state.on_raw_receive_handlers.read().await;
    for handler in handlers.iter() {
        let fut = handler(data.clone());
        tokio::spawn(fut);
    }
}

async fn dispatch_file_upload(state: &Arc<ClientState>, data: &Value) {
    let opcode_val = data.get("opcode").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    if opcode_val != Opcode::NotifAttach.value() {
        return;
    }

    let payload = match data.get("payload") {
        Some(p) => p,
        None => return,
    };

    for key in &["fileId", "videoId"] {
        if let Some(id) = payload.get(key).and_then(|v| v.as_i64()) {
            let mut waiters = state.file_upload_waiters.lock().await;
            if let Some(tx) = waiters.remove(&id) {
                let _ = tx.send(data.clone());
                return;
            }
        }
    }
}

async fn dispatch_message_notification(state: &Arc<ClientState>, opcode: Opcode, data: &Value) {
    if opcode != Opcode::NotifMessage {
        return;
    }

    let payload = match data.get("payload") {
        Some(p) => p,
        None => return,
    };

    let msg = match Message::from_value(payload) {
        Some(m) => m,
        None => return,
    };

    use crate::enums::MessageStatus;

    let handlers_to_use = match &msg.status {
        Some(MessageStatus::Edited) => {
            let handlers = state.on_message_edit_handlers.read().await;
            let fns: Vec<_> = handlers
                .iter()
                .map(|h| (h.handler.clone(), h.filter.clone()))
                .collect();
            fns
        }
        Some(MessageStatus::Removed) => {
            let handlers = state.on_message_delete_handlers.read().await;
            let fns: Vec<_> = handlers
                .iter()
                .map(|h| (h.handler.clone(), h.filter.clone()))
                .collect();
            fns
        }
        None => {
            let handlers = state.on_message_handlers.read().await;
            let fns: Vec<_> = handlers
                .iter()
                .map(|h| (h.handler.clone(), h.filter.clone()))
                .collect();
            fns
        }
    };

    for (handler, filter) in handlers_to_use {
        let passes = match &filter {
            Some(f) => f.check(&msg),
            None => true,
        };
        if passes {
            let fut = handler(msg.clone());
            tokio::spawn(fut);
        }
    }
}

async fn dispatch_reaction_change(state: &Arc<ClientState>, opcode: Opcode, data: &Value) {
    if opcode != Opcode::NotifMsgReactionsChanged {
        return;
    }

    let payload = match data.get("payload") {
        Some(p) => p,
        None => return,
    };

    let chat_id = match payload.get("chatId").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return,
    };

    let message_id = match payload.get("messageId").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return,
    };

    let total_count = payload
        .get("totalCount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let your_reaction = payload
        .get("yourReaction")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let counters: Vec<ReactionCounter> = payload
        .get("counters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| serde_json::from_value(c.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    let reaction_info = ReactionInfo {
        total_count,
        counters,
        your_reaction,
    };

    let handlers = state.on_reaction_handlers.read().await;
    for handler in handlers.iter() {
        let fut = handler(message_id.clone(), chat_id, reaction_info.clone());
        tokio::spawn(fut);
    }
}

async fn dispatch_chat_update(state: &Arc<ClientState>, opcode: Opcode, data: &Value) {
    if opcode != Opcode::NotifChat {
        return;
    }

    let payload = match data.get("payload") {
        Some(p) => p,
        None => return,
    };

    let chat_data = match payload.get("chat") {
        Some(c) => c,
        None => return,
    };

    let chat: crate::types::Chat = match serde_json::from_value(chat_data.clone()) {
        Ok(c) => c,
        Err(_) => return,
    };

    let handlers = state.on_chat_update_handlers.read().await;
    for handler in handlers.iter() {
        let fut = handler(chat.clone());
        tokio::spawn(fut);
    }
}
