// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

use crate::enums::Opcode;
use crate::errors::{MaxError, MaxResult};
use crate::types::{Chat, Message};

use super::MaxClient;

impl MaxClient {
    pub async fn create_group(
        &self,
        name: &str,
        participant_ids: Vec<i64>,
        notify: bool,
    ) -> MaxResult<(Chat, Message)> {
        info!("Creating group '{}'", name);

        let cid = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let payload = serde_json::json!({
            "message": {
                "cid": cid,
                "attaches": [{
                    "_type": "CONTROL",
                    "event": "new",
                    "chatType": "CHAT",
                    "title": name,
                    "userIds": participant_ids
                }]
            },
            "notify": notify
        });

        let data = self.send_default(Opcode::MsgSend, payload).await?;
        Self::handle_error(&data)?;

        let raw_payload = data.get("payload").cloned().unwrap_or_default();

        let chat = raw_payload
            .get("chat")
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No chat in create_group response".to_string())
            })?;

        let message = crate::types::Message::from_value(&raw_payload).ok_or_else(|| {
            MaxError::ResponseStructure("No message in create_group response".to_string())
        })?;

        self.update_chat_cache(chat.clone()).await;

        Ok((chat, message))
    }

    pub async fn invite_users_to_group(
        &self,
        chat_id: i64,
        user_ids: Vec<i64>,
        show_history: bool,
    ) -> MaxResult<Chat> {
        info!("Inviting {} users to chat_id={}", user_ids.len(), chat_id);

        let payload = serde_json::json!({
            "chatId": chat_id,
            "userIds": user_ids,
            "showHistory": show_history,
            "operation": "add"
        });

        let data = self
            .send_default(Opcode::ChatMembersUpdate, payload)
            .await?;
        Self::handle_error(&data)?;

        let chat = data
            .get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No chat in invite_users response".to_string())
            })?;

        self.update_chat_cache(chat.clone()).await;
        Ok(chat)
    }

    pub async fn remove_users_from_group(
        &self,
        chat_id: i64,
        user_ids: Vec<i64>,
        clean_msg_period: i32,
    ) -> MaxResult<bool> {
        info!("Removing {} users from chat_id={}", user_ids.len(), chat_id);

        let payload = serde_json::json!({
            "chatId": chat_id,
            "userIds": user_ids,
            "operation": "remove",
            "cleanMsgPeriod": clean_msg_period
        });

        let data = self
            .send_default(Opcode::ChatMembersUpdate, payload)
            .await?;
        Self::handle_error(&data)?;

        if let Some(chat) = data
            .get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
        {
            self.update_chat_cache(chat).await;
        }

        Ok(true)
    }

    pub async fn change_group_settings(
        &self,
        chat_id: i64,
        all_can_pin_message: Option<bool>,
        only_owner_can_change_icon_title: Option<bool>,
        only_admin_can_add_member: Option<bool>,
        only_admin_can_call: Option<bool>,
        members_can_see_private_link: Option<bool>,
    ) -> MaxResult<()> {
        info!("Changing group settings for chat_id={}", chat_id);

        let mut options = serde_json::Map::new();
        if let Some(v) = all_can_pin_message {
            options.insert("ALL_CAN_PIN_MESSAGE".to_string(), serde_json::json!(v));
        }
        if let Some(v) = only_owner_can_change_icon_title {
            options.insert(
                "ONLY_OWNER_CAN_CHANGE_ICON_TITLE".to_string(),
                serde_json::json!(v),
            );
        }
        if let Some(v) = only_admin_can_add_member {
            options.insert(
                "ONLY_ADMIN_CAN_ADD_MEMBER".to_string(),
                serde_json::json!(v),
            );
        }
        if let Some(v) = only_admin_can_call {
            options.insert("ONLY_ADMIN_CAN_CALL".to_string(), serde_json::json!(v));
        }
        if let Some(v) = members_can_see_private_link {
            options.insert(
                "MEMBERS_CAN_SEE_PRIVATE_LINK".to_string(),
                serde_json::json!(v),
            );
        }

        let payload = serde_json::json!({
            "chatId": chat_id,
            "options": options
        });

        let data = self.send_default(Opcode::ChatUpdate, payload).await?;
        Self::handle_error(&data)?;

        if let Some(chat) = data
            .get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
        {
            self.update_chat_cache(chat).await;
        }

        Ok(())
    }

    pub async fn change_group_profile(
        &self,
        chat_id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> MaxResult<()> {
        info!("Changing group profile for chat_id={}", chat_id);

        let mut payload = serde_json::json!({ "chatId": chat_id });
        if let Some(n) = name {
            payload["theme"] = serde_json::json!(n);
        }
        if let Some(d) = description {
            payload["description"] = serde_json::json!(d);
        }

        let data = self.send_default(Opcode::ChatUpdate, payload).await?;
        Self::handle_error(&data)?;

        if let Some(chat) = data
            .get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
        {
            self.update_chat_cache(chat).await;
        }

        Ok(())
    }

    pub async fn join_group(&self, link: &str) -> MaxResult<Chat> {
        info!("Joining group via link");

        let proceed_link = Self::extract_join_path(link)?;

        let payload = serde_json::json!({ "link": proceed_link });

        let data = self.send_default(Opcode::ChatJoin, payload).await?;
        Self::handle_error(&data)?;

        let chat = data
            .get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No chat in join_group response".to_string())
            })?;

        self.update_chat_cache(chat.clone()).await;
        Ok(chat)
    }

    pub async fn resolve_group_by_link(&self, link: &str) -> MaxResult<Option<Chat>> {
        let proceed_link = Self::extract_join_path(link)?;

        let data = self
            .send_default(
                Opcode::LinkInfo,
                serde_json::json!({ "link": proceed_link }),
            )
            .await?;
        Self::handle_error(&data)?;

        Ok(data
            .get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok()))
    }

    pub async fn rework_invite_link(&self, chat_id: i64) -> MaxResult<Chat> {
        info!("Reworking invite link for chat_id={}", chat_id);

        let payload = serde_json::json!({
            "revokePrivateLink": true,
            "chatId": chat_id
        });

        let data = self.send_default(Opcode::ChatUpdate, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| p.get("chat"))
            .and_then(|c| serde_json::from_value::<Chat>(c.clone()).ok())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No chat in rework_invite_link response".to_string())
            })
    }

    pub async fn get_chats(&self, chat_ids: Vec<i64>) -> MaxResult<Vec<Chat>> {
        info!("Getting chats: {:?}", chat_ids);

        let mut result = Vec::new();
        let mut missing = Vec::new();

        for id in &chat_ids {
            if let Some(c) = self.get_chat(*id).await {
                result.push(c);
            } else {
                missing.push(*id);
            }
        }

        if !missing.is_empty() {
            let payload = serde_json::json!({ "chatIds": missing });
            let data = self.send_default(Opcode::ChatInfo, payload).await?;
            Self::handle_error(&data)?;

            if let Some(chats_arr) = data
                .get("payload")
                .and_then(|p| p.get("chats"))
                .and_then(|c| c.as_array())
            {
                for raw in chats_arr {
                    if let Ok(c) = serde_json::from_value::<Chat>(raw.clone()) {
                        self.update_chat_cache(c.clone()).await;
                        result.push(c);
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn fetch_chats(&self, marker: Option<i64>) -> MaxResult<Vec<Chat>> {
        let ts = marker.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        });

        info!("Fetching chats with marker={}", ts);

        let payload = serde_json::json!({ "marker": ts });
        let data = self.send_default(Opcode::ChatsList, payload).await?;
        Self::handle_error(&data)?;

        let chats: Vec<Chat> = data
            .get("payload")
            .and_then(|p| p.get("chats"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<Chat>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        for chat in &chats {
            self.update_chat_cache(chat.clone()).await;
        }

        Ok(chats)
    }

    pub async fn leave_group(&self, chat_id: i64) -> MaxResult<()> {
        info!("Leaving chat_id={}", chat_id);

        let payload = serde_json::json!({ "chatId": chat_id });
        let data = self.send_default(Opcode::ChatLeave, payload).await?;
        Self::handle_error(&data)?;

        let mut chats = self.chats.write().await;
        chats.retain(|c| c.id != chat_id);

        Ok(())
    }

    fn extract_join_path(link: &str) -> MaxResult<String> {
        link.find("join/")
            .map(|idx| link[idx..].to_string())
            .ok_or_else(|| MaxError::Other("Invalid group link: no 'join/' segment".to_string()))
    }
}
