// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use crate::errors::{MaxError, MaxResult};
use reqwest::Client;
use std::path::Path;
use tokio::fs;

const ALLOWED_PHOTO_EXTENSIONS: &[&str] = &[".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp"];

pub enum FileSource {
    Path(String),
    Url(String),
    Bytes(Vec<u8>),
}

pub struct Photo {
    source: FileSource,
    pub file_name: String,
}

impl Photo {
    pub fn from_path(path: impl Into<String>) -> Self {
        let path = path.into();
        let file_name = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("photo")
            .to_string();
        Self {
            source: FileSource::Path(path),
            file_name,
        }
    }

    pub fn from_url(url: impl Into<String>) -> Self {
        let url = url.into();
        let file_name = url.split('/').last().unwrap_or("photo").to_string();
        Self {
            source: FileSource::Url(url),
            file_name,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, name: impl Into<String>) -> Self {
        Self {
            source: FileSource::Bytes(bytes),
            file_name: name.into(),
        }
    }

    pub fn validate(&self) -> MaxResult<(String, String)> {
        let path_str = match &self.source {
            FileSource::Path(p) => p.clone(),
            FileSource::Url(u) => u.split('?').next().unwrap_or(u).to_string(),
            FileSource::Bytes(_) => {
                let ext = Path::new(&self.file_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("jpg");
                let mime = format!("image/{}", ext);
                return Ok((ext.to_string(), mime));
            }
        };

        let ext = Path::new(&path_str)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default();

        if !ALLOWED_PHOTO_EXTENSIONS.contains(&ext.as_str()) {
            return Err(MaxError::Other(format!(
                "Invalid photo extension: {}. Allowed: {:?}",
                ext, ALLOWED_PHOTO_EXTENSIONS
            )));
        }

        let ext_no_dot = ext.trim_start_matches('.').to_string();
        let mime = format!("image/{}", ext_no_dot);
        Ok((ext_no_dot, mime))
    }

    pub async fn read(&self) -> MaxResult<Vec<u8>> {
        match &self.source {
            FileSource::Path(p) => fs::read(p).await.map_err(MaxError::Io),
            FileSource::Url(u) => {
                let client = Client::new();
                let resp = client.get(u).send().await.map_err(MaxError::Http)?;
                resp.error_for_status()
                    .map_err(MaxError::Http)?
                    .bytes()
                    .await
                    .map(|b| b.to_vec())
                    .map_err(MaxError::Http)
            }
            FileSource::Bytes(b) => Ok(b.clone()),
        }
    }
}

pub struct File {
    source: FileSource,
    pub file_name: String,
    pub path: Option<String>,
}

impl File {
    pub fn from_path(path: impl Into<String>) -> Self {
        let path = path.into();
        let file_name = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();
        Self {
            path: Some(path.clone()),
            source: FileSource::Path(path),
            file_name,
        }
    }

    pub fn from_url(url: impl Into<String>) -> MaxResult<Self> {
        let url = url.into();
        let file_name = url
            .split('/')
            .last()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| MaxError::Other("Cannot determine file name from URL".to_string()))?;
        Ok(Self {
            path: None,
            source: FileSource::Url(url),
            file_name,
        })
    }

    pub async fn read(&self) -> MaxResult<Vec<u8>> {
        match &self.source {
            FileSource::Path(p) => fs::read(p).await.map_err(MaxError::Io),
            FileSource::Url(u) => {
                let client = Client::new();
                let resp = client.get(u).send().await.map_err(MaxError::Http)?;
                resp.error_for_status()
                    .map_err(MaxError::Http)?
                    .bytes()
                    .await
                    .map(|b| b.to_vec())
                    .map_err(MaxError::Http)
            }
            FileSource::Bytes(b) => Ok(b.clone()),
        }
    }

    pub async fn size(&self) -> MaxResult<u64> {
        match &self.source {
            FileSource::Path(p) => {
                let meta = fs::metadata(p).await.map_err(MaxError::Io)?;
                Ok(meta.len())
            }
            FileSource::Url(_) => {
                let bytes = self.read().await?;
                Ok(bytes.len() as u64)
            }
            FileSource::Bytes(b) => Ok(b.len() as u64),
        }
    }
}

pub struct Video {
    source: FileSource,
    pub file_name: String,
    pub path: Option<String>,
}

impl Video {
    pub fn from_path(path: impl Into<String>) -> Self {
        let path = path.into();
        let file_name = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("video")
            .to_string();
        Self {
            path: Some(path.clone()),
            source: FileSource::Path(path),
            file_name,
        }
    }

    pub fn from_url(url: impl Into<String>) -> Self {
        let url = url.into();
        let file_name = url.split('/').last().unwrap_or("video").to_string();
        Self {
            path: None,
            source: FileSource::Url(url),
            file_name,
        }
    }

    pub async fn read(&self) -> MaxResult<Vec<u8>> {
        match &self.source {
            FileSource::Path(p) => fs::read(p).await.map_err(MaxError::Io),
            FileSource::Url(u) => {
                let client = Client::new();
                let resp = client.get(u).send().await.map_err(MaxError::Http)?;
                resp.error_for_status()
                    .map_err(MaxError::Http)?
                    .bytes()
                    .await
                    .map(|b| b.to_vec())
                    .map_err(MaxError::Http)
            }
            FileSource::Bytes(b) => Ok(b.clone()),
        }
    }
}
