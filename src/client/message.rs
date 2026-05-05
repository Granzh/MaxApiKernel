use reqwest::Client as HttpClient;
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use tracing::{debug, info};

use crate::constants::DEFAULT_TIMEOUT_SECS;
use crate::enums::{Opcode, ReadAction};
use crate::errors::{MaxError, MaxResult};
use crate::files::{File, Photo, Video};
use crate::formatting::Formatting;
use crate::types::{FileRequest, Message, ReactionInfo, ReadState, VideoRequest};

use super::MaxClient;

impl MaxClient {
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
        notify: bool,
        attachment: Option<&AttachmentKind>,
        reply_to: Option<i64>,
    ) -> MaxResult<Message> {
        info!("Sending message to chat_id={} notify={}", chat_id, notify);

        let attach_payload = if let Some(a) = attachment {
            Some(self.upload_attachment(a).await?)
        } else {
            None
        };

        let (elements, clean_text) = Formatting::get_elements_from_markdown(text);

        let final_text = if !elements.is_empty() {
            clean_text.clone()
        } else {
            text.to_string()
        };

        let elem_arr: Vec<Value> = elements
            .iter()
            .map(|e| {
                serde_json::json!({
                    "type": e.type_,
                    "from": e.from_,
                    "length": e.length
                })
            })
            .collect();

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let mut message_obj = serde_json::json!({
            "text": final_text,
            "cid": now_ms,
            "elements": elem_arr,
            "attaches": attach_payload.map(|a| vec![a]).unwrap_or_default()
        });

        if let Some(reply_id) = reply_to {
            message_obj["link"] = serde_json::json!({
                "type": "REPLY",
                "messageId": reply_id.to_string()
            });
        }

        let payload = serde_json::json!({
            "chatId": chat_id,
            "message": message_obj,
            "notify": notify
        });

        let data = self.send_default(Opcode::MsgSend, payload).await?;
        Self::handle_error(&data)?;

        let msg = data
            .get("payload")
            .and_then(|p| Message::from_value(p))
            .ok_or_else(|| {
                MaxError::ResponseStructure("No message in send_message response".to_string())
            })?;

        debug!("send_message result: {:?}", msg.id);
        Ok(msg)
    }

    pub async fn edit_message(
        &self,
        chat_id: i64,
        message_id: i64,
        text: &str,
        attachment: Option<&AttachmentKind>,
    ) -> MaxResult<Message> {
        info!(
            "Editing message chat_id={} message_id={}",
            chat_id, message_id
        );

        let attach_payload = if let Some(a) = attachment {
            Some(self.upload_attachment(a).await?)
        } else {
            None
        };

        let (elements, clean_text) = Formatting::get_elements_from_markdown(text);
        let final_text = if !elements.is_empty() {
            clean_text
        } else {
            text.to_string()
        };

        let elem_arr: Vec<Value> = elements
            .iter()
            .map(|e| serde_json::json!({ "type": e.type_, "from": e.from_, "length": e.length }))
            .collect();

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageId": message_id,
            "text": final_text,
            "elements": elem_arr,
            "attaches": attach_payload.map(|a| vec![a]).unwrap_or_default()
        });

        let data = self.send_default(Opcode::MsgEdit, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| Message::from_value(p))
            .ok_or_else(|| {
                MaxError::ResponseStructure("No message in edit_message response".to_string())
            })
    }

    pub async fn delete_message(
        &self,
        chat_id: i64,
        message_ids: Vec<i64>,
        for_me: bool,
    ) -> MaxResult<bool> {
        info!("Deleting messages chat_id={} for_me={}", chat_id, for_me);

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageIds": message_ids,
            "forMe": for_me
        });

        let data = self.send_default(Opcode::MsgDelete, payload).await?;
        Self::handle_error(&data)?;
        Ok(true)
    }

    pub async fn pin_message(
        &self,
        chat_id: i64,
        message_id: i64,
        notify_pin: bool,
    ) -> MaxResult<bool> {
        let payload = serde_json::json!({
            "chatId": chat_id,
            "notifyPin": notify_pin,
            "pinMessageId": message_id
        });

        let data = self.send_default(Opcode::ChatUpdate, payload).await?;
        Self::handle_error(&data)?;
        Ok(true)
    }

    pub async fn fetch_history(
        &self,
        chat_id: i64,
        from_time: Option<i64>,
        forward: i32,
        backward: i32,
    ) -> MaxResult<Vec<Message>> {
        let ts = from_time.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        });

        info!(
            "Fetching history chat_id={} from={} forward={} backward={}",
            chat_id, ts, forward, backward
        );

        let payload = serde_json::json!({
            "chatId": chat_id,
            "from": ts,
            "forward": forward,
            "backward": backward,
            "getMessages": true
        });

        let data = self
            .send_and_wait(Opcode::ChatHistory, payload, 0, 10.0)
            .await?;
        Self::handle_error(&data)?;

        let messages: Vec<Message> = data
            .get("payload")
            .and_then(|p| p.get("messages"))
            .and_then(|m| m.as_array())
            .map(|arr| arr.iter().filter_map(|v| Message::from_value(v)).collect())
            .unwrap_or_default();

        debug!("Fetched {} messages", messages.len() as i32);
        Ok(messages)
    }

    pub async fn get_video_by_id(
        &self,
        chat_id: i64,
        message_id: i64,
        video_id: i64,
    ) -> MaxResult<VideoRequest> {
        info!("Getting video_id={} message_id={}", video_id, message_id);

        let is_socket = matches!(self.transport, super::TransportKind::Socket(_));
        let msg_id_val = if is_socket {
            serde_json::json!(message_id)
        } else {
            serde_json::json!(message_id.to_string())
        };

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageId": msg_id_val,
            "videoId": video_id
        });

        let data = self.send_default(Opcode::VideoPlay, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| VideoRequest::from_value(p))
            .ok_or_else(|| MaxError::ResponseStructure("No video in response".to_string()))
    }

    pub async fn get_file_by_id(
        &self,
        chat_id: i64,
        message_id: i64,
        file_id: i64,
    ) -> MaxResult<FileRequest> {
        info!("Getting file_id={} message_id={}", file_id, message_id);

        let is_socket = matches!(self.transport, super::TransportKind::Socket(_));
        let msg_id_val = if is_socket {
            serde_json::json!(message_id)
        } else {
            serde_json::json!(message_id.to_string())
        };

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageId": msg_id_val,
            "fileId": file_id
        });

        let data = self.send_default(Opcode::FileDownload, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| serde_json::from_value::<FileRequest>(p.clone()).ok())
            .ok_or_else(|| MaxError::ResponseStructure("No file in response".to_string()))
    }

    pub async fn add_reaction(
        &self,
        chat_id: i64,
        message_id: &str,
        reaction: &str,
    ) -> MaxResult<Option<ReactionInfo>> {
        info!(
            "Adding reaction to message chat_id={} message_id={}",
            chat_id, message_id
        );

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageId": message_id,
            "reaction": {
                "reactionType": "EMOJI",
                "id": reaction
            }
        });

        let data = self.send_default(Opcode::MsgReaction, payload).await?;
        Self::handle_error(&data)?;

        Ok(data
            .get("payload")
            .and_then(|p| p.get("reactionInfo"))
            .and_then(|ri| serde_json::from_value(ri.clone()).ok()))
    }

    pub async fn get_reactions(
        &self,
        chat_id: i64,
        message_ids: Vec<String>,
    ) -> MaxResult<std::collections::HashMap<String, ReactionInfo>> {
        info!(
            "Getting reactions for {} messages in chat_id={}",
            message_ids.len(),
            chat_id
        );

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageIds": message_ids
        });

        let data = self.send_default(Opcode::MsgGetReactions, payload).await?;
        Self::handle_error(&data)?;

        let mut result = std::collections::HashMap::new();
        if let Some(map) = data
            .get("payload")
            .and_then(|p| p.get("messagesReactions"))
            .and_then(|m| m.as_object())
        {
            for (k, v) in map {
                if let Ok(ri) = serde_json::from_value::<ReactionInfo>(v.clone()) {
                    result.insert(k.clone(), ri);
                }
            }
        }
        Ok(result)
    }

    pub async fn remove_reaction(
        &self,
        chat_id: i64,
        message_id: &str,
    ) -> MaxResult<Option<ReactionInfo>> {
        info!(
            "Removing reaction from message chat_id={} message_id={}",
            chat_id, message_id
        );

        let payload = serde_json::json!({
            "chatId": chat_id,
            "messageId": message_id
        });

        let data = self
            .send_default(Opcode::MsgCancelReaction, payload)
            .await?;
        Self::handle_error(&data)?;

        Ok(data
            .get("payload")
            .and_then(|p| p.get("reactionInfo"))
            .and_then(|ri| serde_json::from_value(ri.clone()).ok()))
    }

    pub async fn read_message(&self, message_id: i64, chat_id: i64) -> MaxResult<ReadState> {
        info!(
            "Marking message as read chat_id={} message_id={}",
            chat_id, message_id
        );

        let mark = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let payload = serde_json::json!({
            "type": ReadAction::ReadMessage.as_str(),
            "chatId": chat_id,
            "messageId": message_id.to_string(),
            "mark": mark
        });

        let data = self.send_default(Opcode::ChatMark, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .ok_or_else(|| MaxError::ResponseStructure("No ReadState in response".to_string()))
    }

    async fn upload_attachment(&self, kind: &AttachmentKind) -> MaxResult<Value> {
        match kind {
            AttachmentKind::Photo(photo) => self.upload_photo(photo).await,
            AttachmentKind::File(file) => self.upload_file(file).await,
            AttachmentKind::Video(video) => self.upload_video(video).await,
        }
    }

    async fn upload_photo(&self, photo: &Photo) -> MaxResult<Value> {
        info!("Uploading photo");

        let data = self
            .send_default(
                Opcode::PhotoUpload,
                serde_json::json!({ "count": 1, "profile": false }),
            )
            .await?;
        Self::handle_error(&data)?;

        let url = data
            .get("payload")
            .and_then(|p| p.get("url"))
            .and_then(|u| u.as_str())
            .ok_or_else(|| MaxError::ResponseStructure("No upload URL for photo".to_string()))?
            .to_string();

        let (ext, mime) = photo.validate()?;
        let bytes = photo.read().await?;

        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(format!("image.{}", ext))
            .mime_str(&mime)
            .map_err(|e| MaxError::Other(format!("mime error: {}", e)))?;

        let form = reqwest::multipart::Form::new().part("file", part);
        let client = HttpClient::new();
        let resp = client
            .post(&url)
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

        let token = result
            .get("photos")
            .and_then(|p| p.as_object())
            .and_then(|m| m.values().next())
            .and_then(|v| v.get("token"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No photo token in upload response".to_string())
            })?
            .to_string();

        Ok(serde_json::json!({ "_type": "PHOTO", "photoToken": token }))
    }

    async fn upload_file(&self, file: &File) -> MaxResult<Value> {
        info!("Uploading file");

        let data = self
            .send_default(
                Opcode::FileUpload,
                serde_json::json!({ "count": 1, "profile": false }),
            )
            .await?;
        Self::handle_error(&data)?;

        let info0 = data
            .get("payload")
            .and_then(|p| p.get("info"))
            .and_then(|i| i.as_array())
            .and_then(|a| a.first())
            .cloned()
            .ok_or_else(|| MaxError::ResponseStructure("No upload info for file".to_string()))?;

        let url = info0
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| MaxError::ResponseStructure("No upload URL for file".to_string()))?
            .to_string();

        let file_id = info0
            .get("fileId")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MaxError::ResponseStructure("No fileId in upload info".to_string()))?;

        let file_bytes = file.read().await?;
        let file_size = file_bytes.len();

        let (tx, rx) = oneshot::channel::<Value>();
        {
            let mut waiters = self.state.file_upload_waiters.lock().await;
            waiters.insert(file_id, tx);
        }

        let client = HttpClient::new();
        let resp = client
            .post(&url)
            .header(
                "Content-Disposition",
                format!("attachment; filename={}", file.file_name),
            )
            .header("Content-Length", file_size.to_string())
            .header(
                "Content-Range",
                format!("0-{}/{}", file_size - 1, file_size),
            )
            .body(file_bytes)
            .send()
            .await
            .map_err(MaxError::Http)?;

        if !resp.status().is_success() {
            let mut waiters = self.state.file_upload_waiters.lock().await;
            waiters.remove(&file_id);
            return Err(MaxError::Other(format!(
                "File upload HTTP error: {}",
                resp.status()
            )));
        }

        tokio::time::timeout(Duration::from_secs_f64(DEFAULT_TIMEOUT_SECS), rx)
            .await
            .map_err(|_| {
                MaxError::Other(format!(
                    "Timeout waiting for file upload notification (fileId={})",
                    file_id
                ))
            })?
            .map_err(|_| MaxError::Other("File upload waiter channel closed".to_string()))?;

        Ok(serde_json::json!({ "_type": "FILE", "fileId": file_id }))
    }

    async fn upload_video(&self, video: &Video) -> MaxResult<Value> {
        info!("Uploading video");

        let data = self
            .send_default(
                Opcode::VideoUpload,
                serde_json::json!({ "count": 1, "profile": false }),
            )
            .await?;
        Self::handle_error(&data)?;

        let info0 = data
            .get("payload")
            .and_then(|p| p.get("info"))
            .and_then(|i| i.as_array())
            .and_then(|a| a.first())
            .cloned()
            .ok_or_else(|| MaxError::ResponseStructure("No upload info for video".to_string()))?;

        let url = info0
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| MaxError::ResponseStructure("No upload URL for video".to_string()))?
            .to_string();

        let video_id = info0
            .get("videoId")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MaxError::ResponseStructure("No videoId in upload info".to_string()))?;

        let token = info0
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No token in video upload info".to_string())
            })?
            .to_string();

        let video_bytes = video.read().await?;
        let file_size = video_bytes.len();

        let (tx, rx) = oneshot::channel::<Value>();
        {
            let mut waiters = self.state.file_upload_waiters.lock().await;
            waiters.insert(video_id, tx);
        }

        let client = HttpClient::new();
        let resp = client
            .post(&url)
            .header(
                "Content-Disposition",
                format!("attachment; filename={}", video.file_name),
            )
            .header("Content-Length", file_size.to_string())
            .header(
                "Content-Range",
                format!("0-{}/{}", file_size - 1, file_size),
            )
            .body(video_bytes)
            .send()
            .await
            .map_err(MaxError::Http)?;

        if !resp.status().is_success() {
            let mut waiters = self.state.file_upload_waiters.lock().await;
            waiters.remove(&video_id);
            return Err(MaxError::Other(format!(
                "Video upload HTTP error: {}",
                resp.status()
            )));
        }

        tokio::time::timeout(Duration::from_secs(900), rx)
            .await
            .map_err(|_| {
                MaxError::Other(format!(
                    "Timeout waiting for video upload notification (videoId={})",
                    video_id
                ))
            })?
            .map_err(|_| MaxError::Other("Video upload waiter channel closed".to_string()))?;

        Ok(serde_json::json!({ "_type": "VIDEO", "videoId": video_id, "token": token }))
    }
}

pub enum AttachmentKind {
    Photo(Photo),
    File(File),
    Video(Video),
}
