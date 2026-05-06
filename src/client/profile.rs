// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use reqwest::Client as HttpClient;
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::enums::Opcode;
use crate::errors::{MaxError, MaxResult};
use crate::files::Photo;
use crate::types::{FolderList, FolderUpdate, Me};

use super::MaxClient;

impl MaxClient {
    pub async fn change_profile(
        &self,
        first_name: &str,
        last_name: Option<&str>,
        description: Option<&str>,
        photo: Option<&Photo>,
    ) -> MaxResult<bool> {
        info!("Changing profile");

        let photo_token = if let Some(p) = photo {
            Some(self.upload_profile_photo(p).await?)
        } else {
            None
        };

        let mut payload = serde_json::json!({
            "firstName": first_name,
            "avatarType": "USER_AVATAR"
        });

        if let Some(ln) = last_name {
            payload["lastName"] = serde_json::json!(ln);
        }
        if let Some(desc) = description {
            payload["description"] = serde_json::json!(desc);
        }
        if let Some(token) = photo_token {
            payload["photoToken"] = serde_json::json!(token);
        }

        let data = self.send_default(Opcode::Profile, payload).await?;
        Self::handle_error(&data)?;

        if let Some(me_data) = data
            .get("payload")
            .and_then(|p| p.get("profile"))
            .and_then(|p| p.get("contact"))
        {
            if let Ok(me) = serde_json::from_value::<Me>(me_data.clone()) {
                let mut guard = self.me.write().await;
                *guard = Some(me);
            }
        }

        Ok(true)
    }

    pub async fn close_all_sessions(&self) -> MaxResult<bool> {
        info!("Closing all other sessions");

        let data = self
            .send_default(Opcode::SessionsClose, serde_json::json!({}))
            .await?;
        Self::handle_error(&data)?;

        Ok(true)
    }

    pub async fn logout(&self) -> MaxResult<bool> {
        info!("Logging out");

        let data = self
            .send_default(Opcode::Logout, serde_json::json!({}))
            .await?;
        Self::handle_error(&data)?;

        Ok(true)
    }

    pub async fn create_folder(
        &self,
        title: &str,
        chat_include: Vec<i64>,
        filters: Vec<Value>,
    ) -> MaxResult<FolderUpdate> {
        info!("Creating folder '{}'", title);

        let payload = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "title": title,
            "include": chat_include,
            "filters": filters
        });

        let data = self.send_default(Opcode::FoldersUpdate, payload).await?;
        Self::handle_error(&data)?;

        serde_json::from_value(data.get("payload").cloned().unwrap_or_default())
            .map_err(|e| MaxError::Other(format!("create_folder parse error: {}", e)))
    }

    pub async fn get_folders(&self, folder_sync: i64) -> MaxResult<FolderList> {
        info!("Fetching folders");

        let payload = serde_json::json!({ "folderSync": folder_sync });
        let data = self.send_default(Opcode::FoldersGet, payload).await?;
        Self::handle_error(&data)?;

        serde_json::from_value(data.get("payload").cloned().unwrap_or_default())
            .map_err(|e| MaxError::Other(format!("get_folders parse error: {}", e)))
    }

    pub async fn update_folder(
        &self,
        folder_id: &str,
        title: &str,
        chat_include: Vec<i64>,
        filters: Vec<Value>,
        options: Vec<Value>,
    ) -> MaxResult<FolderUpdate> {
        info!("Updating folder '{}'", folder_id);

        let payload = serde_json::json!({
            "id": folder_id,
            "title": title,
            "include": chat_include,
            "filters": filters,
            "options": options
        });

        let data = self.send_default(Opcode::FoldersUpdate, payload).await?;
        Self::handle_error(&data)?;

        serde_json::from_value(data.get("payload").cloned().unwrap_or_default())
            .map_err(|e| MaxError::Other(format!("update_folder parse error: {}", e)))
    }

    pub async fn delete_folder(&self, folder_id: &str) -> MaxResult<FolderUpdate> {
        info!("Deleting folder '{}'", folder_id);

        let payload = serde_json::json!({ "folderIds": [folder_id] });
        let data = self.send_default(Opcode::FoldersDelete, payload).await?;
        Self::handle_error(&data)?;

        serde_json::from_value(data.get("payload").cloned().unwrap_or_default())
            .map_err(|e| MaxError::Other(format!("delete_folder parse error: {}", e)))
    }

    async fn upload_profile_photo(&self, photo: &Photo) -> MaxResult<String> {
        info!("Uploading profile photo");

        let data = self
            .send_default(
                Opcode::PhotoUpload,
                serde_json::json!({ "count": 1, "profile": true }),
            )
            .await?;
        Self::handle_error(&data)?;

        let upload_url = data
            .get("payload")
            .and_then(|p| p.get("url"))
            .and_then(|u| u.as_str())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No upload URL for profile photo".to_string())
            })?
            .to_string();

        let photo_id = reqwest::Url::parse(&upload_url)
            .ok()
            .and_then(|u| {
                u.query_pairs()
                    .find(|(k, _)| k == "photoIds")
                    .map(|(_, v)| v.to_string())
            })
            .ok_or_else(|| MaxError::Other("No photoIds in upload URL".to_string()))?;

        let bytes = photo.read().await?;
        let (_, mime) = photo.validate()?;

        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(photo.file_name.clone())
            .mime_str(&mime)
            .map_err(|e| MaxError::Other(format!("mime error: {}", e)))?;

        let form = reqwest::multipart::Form::new().part("file", part);
        let client = HttpClient::new();

        let resp = client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(MaxError::Http)?;

        let result: Value = resp
            .error_for_status()
            .map_err(MaxError::Http)?
            .json()
            .await
            .map_err(MaxError::Http)?;

        result
            .get("photos")
            .and_then(|p| p.get(&photo_id))
            .and_then(|v| v.get("token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No photo token in profile upload response".to_string())
            })
    }
}
