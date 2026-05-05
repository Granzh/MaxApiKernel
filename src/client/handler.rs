use std::sync::Arc;

use crate::filters::MessageFilter;
use crate::transport::{
    ChatHandlerFn, FilteredMessageHandler, MessageHandlerFn, RawHandlerFn, ReactionHandlerFn,
    SimpleHandlerFn,
};
use crate::types::{Chat, Message, ReactionInfo};

use super::MaxClient;

impl MaxClient {
    pub fn on_message<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(Message) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.add_message_handler_with_filter(Arc::new(move |m| Box::pin(handler(m))), None);
        self
    }

    pub fn on_message_with_filter<F, Fut>(
        &self,
        handler: F,
        filter: Arc<dyn MessageFilter>,
    ) -> &Self
    where
        F: Fn(Message) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.add_message_handler_with_filter(Arc::new(move |m| Box::pin(handler(m))), Some(filter));
        self
    }

    pub fn add_message_handler_with_filter(
        &self,
        handler: MessageHandlerFn,
        filter: Option<Arc<dyn MessageFilter>>,
    ) {
        let entry = FilteredMessageHandler { handler, filter };
        let handlers = self.state.on_message_handlers.try_write();
        if let Ok(mut guard) = handlers {
            guard.push(entry);
        }
    }

    pub fn on_message_edit<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(Message) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.add_message_edit_handler_with_filter(Arc::new(move |m| Box::pin(handler(m))), None);
        self
    }

    pub fn add_message_edit_handler_with_filter(
        &self,
        handler: MessageHandlerFn,
        filter: Option<Arc<dyn MessageFilter>>,
    ) {
        let entry = FilteredMessageHandler { handler, filter };
        if let Ok(mut guard) = self.state.on_message_edit_handlers.try_write() {
            guard.push(entry);
        }
    }

    pub fn on_message_delete<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(Message) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.add_message_delete_handler_with_filter(Arc::new(move |m| Box::pin(handler(m))), None);
        self
    }

    pub fn add_message_delete_handler_with_filter(
        &self,
        handler: MessageHandlerFn,
        filter: Option<Arc<dyn MessageFilter>>,
    ) {
        let entry = FilteredMessageHandler { handler, filter };
        if let Ok(mut guard) = self.state.on_message_delete_handlers.try_write() {
            guard.push(entry);
        }
    }

    pub fn on_reaction_change<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(String, i64, ReactionInfo) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let h: ReactionHandlerFn =
            Arc::new(move |msg_id, chat_id, ri| Box::pin(handler(msg_id, chat_id, ri)));
        if let Ok(mut guard) = self.state.on_reaction_handlers.try_write() {
            guard.push(h);
        }
        self
    }

    pub fn on_chat_update<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(Chat) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let h: ChatHandlerFn = Arc::new(move |chat| Box::pin(handler(chat)));
        if let Ok(mut guard) = self.state.on_chat_update_handlers.try_write() {
            guard.push(h);
        }
        self
    }

    pub fn on_raw_receive<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let h: RawHandlerFn = Arc::new(move |data| Box::pin(handler(data)));
        if let Ok(mut guard) = self.state.on_raw_receive_handlers.try_write() {
            guard.push(h);
        }
        self
    }

    pub fn on_start<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let h: SimpleHandlerFn = Arc::new(move || Box::pin(handler()));
        if let Ok(mut guard) = self.state.on_start_handler.try_write() {
            *guard = Some(h);
        }
        self
    }

    pub fn add_task<F, Fut>(&self, handler: F, interval: std::time::Duration) -> &Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let h: SimpleHandlerFn = Arc::new(move || Box::pin(handler()));
        if let Ok(mut guard) = self.state.scheduled_tasks.try_write() {
            guard.push((h, interval));
        }
        self
    }
}
