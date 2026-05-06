// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use tracing::info;

use crate::enums::{ContactAction, Opcode};
use crate::errors::{MaxError, MaxResult};
use crate::types::{Contact, Session, User};

use super::MaxClient;

impl MaxClient {
    pub fn get_cached_user(&self, _user_id: i64) -> Option<User> {
        let contacts = self.contacts.try_read().ok()?;
        contacts.iter().find(|u| u.id == _user_id).cloned()
    }

    pub async fn get_user(&self, user_id: i64) -> MaxResult<Option<User>> {
        if let Some(u) = self.get_cached_user(user_id) {
            return Ok(Some(u));
        }
        let mut users = self.fetch_users(vec![user_id]).await?;
        Ok(users.pop())
    }

    pub async fn get_users(&self, user_ids: Vec<i64>) -> MaxResult<Vec<User>> {
        let mut cached = Vec::new();
        let mut missing = Vec::new();

        for id in &user_ids {
            if let Some(u) = self.get_cached_user(*id) {
                cached.push(u);
            } else {
                missing.push(*id);
            }
        }

        if !missing.is_empty() {
            let fetched = self.fetch_users(missing).await?;
            cached.extend(fetched);
        }

        Ok(cached)
    }

    pub async fn fetch_users(&self, user_ids: Vec<i64>) -> MaxResult<Vec<User>> {
        info!("Fetching {} users", user_ids.len());

        let payload = serde_json::json!({ "contactIds": user_ids });
        let data = self.send_default(Opcode::ContactInfo, payload).await?;
        Self::handle_error(&data)?;

        let users: Vec<User> = data
            .get("payload")
            .and_then(|p| p.get("contacts"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<User>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        {
            let mut contacts = self.contacts.write().await;
            for user in &users {
                if let Some(pos) = contacts.iter().position(|u| u.id == user.id) {
                    contacts[pos] = user.clone();
                } else {
                    contacts.push(user.clone());
                }
            }
        }

        Ok(users)
    }

    pub async fn search_by_phone(&self, phone: &str) -> MaxResult<User> {
        info!("Searching user by phone: {}", phone);

        let payload = serde_json::json!({ "phone": phone });
        let data = self
            .send_default(Opcode::ContactInfoByPhone, payload)
            .await?;
        Self::handle_error(&data)?;

        let user = data
            .get("payload")
            .and_then(|p| p.get("contact"))
            .and_then(|c| serde_json::from_value::<User>(c.clone()).ok())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No contact in search_by_phone response".to_string())
            })?;

        {
            let mut contacts = self.contacts.write().await;
            if let Some(pos) = contacts.iter().position(|u| u.id == user.id) {
                contacts[pos] = user.clone();
            } else {
                contacts.push(user.clone());
            }
        }

        Ok(user)
    }

    pub async fn add_contact(&self, contact_id: i64) -> MaxResult<Contact> {
        info!("Adding contact {}", contact_id);

        let payload = serde_json::json!({
            "contactId": contact_id,
            "action": ContactAction::Add.as_str()
        });

        let data = self.send_default(Opcode::ContactUpdate, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| p.get("contact"))
            .and_then(|c| serde_json::from_value::<Contact>(c.clone()).ok())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No contact in add_contact response".to_string())
            })
    }

    pub async fn remove_contact(&self, contact_id: i64) -> MaxResult<bool> {
        info!("Removing contact {}", contact_id);

        let payload = serde_json::json!({
            "contactId": contact_id,
            "action": ContactAction::Remove.as_str()
        });

        let data = self.send_default(Opcode::ContactUpdate, payload).await?;
        Self::handle_error(&data)?;

        Ok(true)
    }

    pub async fn get_sessions(&self) -> MaxResult<Vec<Session>> {
        info!("Fetching sessions");

        let data = self
            .send_default(Opcode::SessionsInfo, serde_json::json!({}))
            .await?;
        Self::handle_error(&data)?;

        let sessions: Vec<Session> = data
            .get("payload")
            .and_then(|p| p.get("sessions"))
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<Session>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(sessions)
    }

    pub fn get_chat_id(first_user_id: i64, second_user_id: i64) -> i64 {
        first_user_id ^ second_user_id
    }
}
