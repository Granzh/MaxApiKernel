// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use crate::enums::{AttachType, MessageStatus};
use crate::types::Message;
use regex::Regex;
use std::sync::Arc;

pub trait MessageFilter: Send + Sync {
    fn check(&self, message: &Message) -> bool;

    fn and(self, other: impl MessageFilter + 'static) -> AndFilter
    where
        Self: Sized + 'static,
    {
        AndFilter::new(vec![Arc::new(self), Arc::new(other)])
    }

    fn or(self, other: impl MessageFilter + 'static) -> OrFilter
    where
        Self: Sized + 'static,
    {
        OrFilter::new(vec![Arc::new(self), Arc::new(other)])
    }

    fn not(self) -> NotFilter
    where
        Self: Sized + 'static,
    {
        NotFilter::new(Arc::new(self))
    }
}

pub struct AndFilter {
    filters: Vec<Arc<dyn MessageFilter>>,
}

impl AndFilter {
    pub fn new(filters: Vec<Arc<dyn MessageFilter>>) -> Self {
        Self { filters }
    }
}

impl MessageFilter for AndFilter {
    fn check(&self, message: &Message) -> bool {
        self.filters.iter().all(|f| f.check(message))
    }
}

pub struct OrFilter {
    filters: Vec<Arc<dyn MessageFilter>>,
}

impl OrFilter {
    pub fn new(filters: Vec<Arc<dyn MessageFilter>>) -> Self {
        Self { filters }
    }
}

impl MessageFilter for OrFilter {
    fn check(&self, message: &Message) -> bool {
        self.filters.iter().any(|f| f.check(message))
    }
}

pub struct NotFilter {
    base: Arc<dyn MessageFilter>,
}

impl NotFilter {
    pub fn new(base: Arc<dyn MessageFilter>) -> Self {
        Self { base }
    }
}

impl MessageFilter for NotFilter {
    fn check(&self, message: &Message) -> bool {
        !self.base.check(message)
    }
}

pub struct ChatFilter {
    chat_id: i64,
}

impl ChatFilter {
    pub fn new(chat_id: i64) -> Self {
        Self { chat_id }
    }
}

impl MessageFilter for ChatFilter {
    fn check(&self, message: &Message) -> bool {
        message.chat_id == Some(self.chat_id)
    }
}

pub struct TextFilter {
    text: String,
}

impl TextFilter {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl MessageFilter for TextFilter {
    fn check(&self, message: &Message) -> bool {
        message.text.contains(&self.text)
    }
}

pub struct SenderFilter {
    user_id: i64,
}

impl SenderFilter {
    pub fn new(user_id: i64) -> Self {
        Self { user_id }
    }
}

impl MessageFilter for SenderFilter {
    fn check(&self, message: &Message) -> bool {
        message.sender == Some(self.user_id)
    }
}

pub struct StatusFilter {
    status: MessageStatus,
}

impl StatusFilter {
    pub fn new(status: MessageStatus) -> Self {
        Self { status }
    }
}

impl MessageFilter for StatusFilter {
    fn check(&self, message: &Message) -> bool {
        message.status.as_ref() == Some(&self.status)
    }
}

pub struct TextContainsFilter {
    substring: String,
}

impl TextContainsFilter {
    pub fn new(substring: String) -> Self {
        Self { substring }
    }
}

impl MessageFilter for TextContainsFilter {
    fn check(&self, message: &Message) -> bool {
        message.text.contains(&self.substring)
    }
}

pub struct RegexTextFilter {
    regex: Regex,
}

impl RegexTextFilter {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: Regex::new(pattern)?,
        })
    }
}

impl MessageFilter for RegexTextFilter {
    fn check(&self, message: &Message) -> bool {
        self.regex.is_match(&message.text)
    }
}

pub struct MediaFilter;

impl MessageFilter for MediaFilter {
    fn check(&self, message: &Message) -> bool {
        !message.attaches.is_empty()
    }
}

pub struct FileFilter;

impl MessageFilter for FileFilter {
    fn check(&self, message: &Message) -> bool {
        message
            .attaches
            .iter()
            .any(|a| a.attach_type() == AttachType::File)
    }
}

pub struct Filters;

impl Filters {
    pub fn chat(chat_id: i64) -> Arc<dyn MessageFilter> {
        Arc::new(ChatFilter::new(chat_id))
    }

    pub fn text(text: impl Into<String>) -> Arc<dyn MessageFilter> {
        Arc::new(TextFilter::new(text.into()))
    }

    pub fn sender(user_id: i64) -> Arc<dyn MessageFilter> {
        Arc::new(SenderFilter::new(user_id))
    }

    pub fn status(status: MessageStatus) -> Arc<dyn MessageFilter> {
        Arc::new(StatusFilter::new(status))
    }

    pub fn text_contains(substring: impl Into<String>) -> Arc<dyn MessageFilter> {
        Arc::new(TextContainsFilter::new(substring.into()))
    }

    pub fn text_matches(pattern: &str) -> Result<Arc<dyn MessageFilter>, regex::Error> {
        Ok(Arc::new(RegexTextFilter::new(pattern)?))
    }

    pub fn has_media() -> Arc<dyn MessageFilter> {
        Arc::new(MediaFilter)
    }

    pub fn has_file() -> Arc<dyn MessageFilter> {
        Arc::new(FileFilter)
    }
}
