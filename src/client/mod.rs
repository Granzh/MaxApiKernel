pub mod auth;
pub mod group;
pub mod handler;
pub mod message;
pub mod profile;
pub mod scheduler;
pub mod user;

use serde_json::Value;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

use crate::constants::*;
use crate::db::Database;
use crate::enums::Opcode;
use crate::errors::{MaxError, MaxResult};
use crate::payloads::{SyncPayload, UserAgentPayload};
use crate::transport::{socket::SocketTransport, websocket::WebSocketTransport, ClientState};
use crate::types::{Channel, Chat, Dialog, Me, User};

pub enum TransportKind {
    WebSocket(WebSocketTransport),
    Socket(SocketTransport),
}

pub struct ClientConfig {
    pub phone: String,
    pub uri: String,
    pub session_name: String,
    pub work_dir: String,
    pub host: String,
    pub port: u16,
    pub proxy: Option<String>,
    pub reconnect: bool,
    pub reconnect_delay: Duration,
    pub registration: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub send_fake_telemetry: bool,
    pub token: Option<String>,
    pub user_agent: UserAgentPayload,
    pub device_id: Option<Uuid>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            phone: String::new(),
            uri: WEBSOCKET_URI.to_string(),
            session_name: SESSION_STORAGE_DB.to_string(),
            work_dir: ".".to_string(),
            host: HOST.to_string(),
            port: PORT,
            proxy: None,
            reconnect: true,
            reconnect_delay: Duration::from_secs(1),
            registration: false,
            first_name: String::new(),
            last_name: None,
            send_fake_telemetry: true,
            token: None,
            user_agent: UserAgentPayload::for_web(),
            device_id: None,
        }
    }
}

pub struct MaxClient {
    pub config: ClientConfig,
    pub state: Arc<ClientState>,
    pub transport: TransportKind,

    pub chats: Arc<RwLock<Vec<Chat>>>,
    pub dialogs: Arc<RwLock<Vec<Dialog>>>,
    pub channels: Arc<RwLock<Vec<Channel>>>,
    pub me: Arc<RwLock<Option<Me>>>,
    pub contacts: Arc<RwLock<Vec<User>>>,

    db: Arc<Database>,
    token: Arc<RwLock<Option<String>>>,
    device_id: Uuid,

    stop_tx: Arc<RwLock<Option<tokio::sync::broadcast::Sender<()>>>>,
    session_id: i64,
    action_id: Arc<std::sync::atomic::AtomicI64>,
    current_screen: Arc<RwLock<String>>,
}

impl MaxClient {
    pub fn new(phone: impl Into<String>) -> MaxResult<Self> {
        Self::with_config(ClientConfig {
            phone: phone.into(),
            user_agent: UserAgentPayload::for_web(),
            ..Default::default()
        })
    }

    pub fn with_config(config: ClientConfig) -> MaxResult<Self> {
        let phone_regex = regex::Regex::new(PHONE_REGEX).unwrap();
        if !phone_regex.is_match(&config.phone) {
            return Err(MaxError::InvalidPhone(config.phone.clone()));
        }

        let db = Database::new(&config.work_dir)
            .map_err(|e| MaxError::Other(format!("Database init error: {}", e)))?;

        let device_id = config
            .device_id
            .unwrap_or_else(|| db.get_device_id().unwrap_or_else(|_| Uuid::new_v4()));

        let stored_token = db.get_auth_token().ok().flatten();
        let initial_token = stored_token.or_else(|| config.token.clone());

        let state = ClientState::new();

        let user_agent_str = config.user_agent.header_user_agent.clone();
        let transport = if config.user_agent.device_type == "WEB" {
            TransportKind::WebSocket(WebSocketTransport::new(
                config.uri.clone(),
                user_agent_str,
                config.proxy.clone(),
            ))
        } else {
            TransportKind::Socket(SocketTransport::new(config.host.clone(), config.port))
        };

        let session_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        Ok(Self {
            transport,
            state,
            chats: Arc::new(RwLock::new(Vec::new())),
            dialogs: Arc::new(RwLock::new(Vec::new())),
            channels: Arc::new(RwLock::new(Vec::new())),
            me: Arc::new(RwLock::new(None)),
            contacts: Arc::new(RwLock::new(Vec::new())),
            db: Arc::new(db),
            token: Arc::new(RwLock::new(initial_token)),
            device_id,
            stop_tx: Arc::new(RwLock::new(None)),
            session_id,
            action_id: Arc::new(std::sync::atomic::AtomicI64::new(1)),
            current_screen: Arc::new(RwLock::new("chats_list_tab".to_string())),
            config,
        })
    }

    pub fn new_socket(
        phone: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> MaxResult<Self> {
        Self::with_config(ClientConfig {
            phone: phone.into(),
            host: host.into(),
            port,
            user_agent: UserAgentPayload::for_desktop(),
            ..Default::default()
        })
    }

    pub fn is_connected(&self) -> bool {
        self.state.is_connected.load(Ordering::SeqCst)
    }

    pub async fn send_and_wait(
        &self,
        opcode: Opcode,
        payload: Value,
        cmd: i32,
        timeout_secs: f64,
    ) -> MaxResult<Value> {
        let timeout = Duration::from_secs_f64(timeout_secs);
        match &self.transport {
            TransportKind::WebSocket(ws) => {
                ws.send_and_wait(&self.state, opcode, payload, cmd, timeout)
                    .await
            }
            TransportKind::Socket(sock) => {
                sock.send_and_wait(&self.state, opcode, payload, cmd, timeout)
                    .await
            }
        }
    }

    pub async fn send_default(&self, opcode: Opcode, payload: Value) -> MaxResult<Value> {
        self.send_and_wait(opcode, payload, 0, DEFAULT_TIMEOUT_SECS)
            .await
    }

    pub fn check_error(data: &Value) -> Option<MaxError> {
        if data.get("payload").and_then(|p| p.get("error")).is_some() {
            Some(MaxError::from_response(data))
        } else {
            None
        }
    }

    pub fn handle_error(data: &Value) -> MaxResult<()> {
        if let Some(err) = Self::check_error(data) {
            Err(err)
        } else {
            Ok(())
        }
    }

    pub async fn get_chat(&self, chat_id: i64) -> Option<Chat> {
        let chats = self.chats.read().await;
        chats.iter().find(|c| c.id == chat_id).cloned()
    }

    async fn update_chat_cache(&self, chat: Chat) {
        let mut chats = self.chats.write().await;
        if let Some(pos) = chats.iter().position(|c| c.id == chat.id) {
            chats[pos] = chat;
        } else {
            chats.push(chat);
        }
    }

    pub async fn start(&self) -> MaxResult<()> {
        info!("Client starting");

        let (stop_tx, _stop_rx) = tokio::sync::broadcast::channel::<()>(1);
        {
            let mut guard = self.stop_tx.write().await;
            *guard = Some(stop_tx.clone());
        }

        loop {
            if self.stop_tx.read().await.is_none() {
                break;
            }

            match self.run_once().await {
                Ok(_) => {
                    info!("Client connection ended cleanly");
                }
                Err(e) => {
                    error!("Client connection error: {}", e);
                }
            }

            self.cleanup().await;

            if !self.config.reconnect {
                info!("Reconnect disabled, exiting");
                break;
            }

            info!(
                "Reconnecting in {}s",
                self.config.reconnect_delay.as_secs_f64()
            );
            tokio::time::sleep(self.config.reconnect_delay).await;
        }

        info!("Client exited cleanly");
        Ok(())
    }

    async fn run_once(&self) -> MaxResult<()> {
        self.connect_transport().await?;

        if self.config.registration {
            if self.config.first_name.is_empty() {
                return Err(MaxError::Other(
                    "First name is required for registration".to_string(),
                ));
            }
            self.register(&self.config.first_name, self.config.last_name.as_deref())
                .await?;
        }

        {
            let token_guard = self.token.read().await;
            if let Some(token) = token_guard.as_ref() {
                if self.db.get_auth_token().ok().flatten().is_none() {
                    drop(token_guard);
                    let token_clone = { self.token.read().await.clone() };
                    if let Some(t) = token_clone {
                        let _ = self.db.update_auth_token(&self.device_id, &t);
                    }
                }
            }
        }

        {
            let token_guard = self.token.read().await;
            if token_guard.is_none() {
                drop(token_guard);
                self.login().await?;
            }
        }

        self.sync_data().await?;
        self.post_login_tasks().await?;

        let mut stop_rx = {
            let guard = self.stop_tx.read().await;
            guard
                .as_ref()
                .map(|tx| tx.subscribe())
                .ok_or_else(|| MaxError::Other("Stop channel not initialized".to_string()))?
        };

        tokio::select! {
            _ = stop_rx.recv() => {
                info!("Stop signal received");
            }
            _ = self.wait_for_disconnect() => {
                info!("Transport disconnected");
            }
        }

        Ok(())
    }

    async fn connect_transport(&self) -> MaxResult<()> {
        match &self.transport {
            TransportKind::WebSocket(ws) => ws.connect(Arc::clone(&self.state)).await?,
            TransportKind::Socket(sock) => sock.connect(Arc::clone(&self.state)).await?,
        }
        self.handshake().await
    }

    async fn handshake(&self) -> MaxResult<()> {
        let user_agent_json = serde_json::to_value(&self.config.user_agent)?;
        let payload = serde_json::json!({
            "deviceId": self.device_id.to_string(),
            "userAgent": user_agent_json
        });

        let data = self
            .send_and_wait(Opcode::SessionInit, payload, 0, DEFAULT_TIMEOUT_SECS)
            .await?;

        Self::handle_error(&data)?;
        info!("Handshake completed");
        Ok(())
    }

    pub async fn sync_data(&self) -> MaxResult<()> {
        info!("Starting sync");

        let token = {
            let guard = self.token.read().await;
            guard
                .clone()
                .ok_or_else(|| MaxError::Other("No token for sync".to_string()))?
        };

        let payload = serde_json::to_value(SyncPayload {
            interactive: true,
            token,
            chats_sync: 0,
            contacts_sync: 0,
            presence_sync: 0,
            drafts_sync: 0,
            chats_count: 40,
            user_agent: self.config.user_agent.clone(),
        })?;

        let data = self
            .send_and_wait(Opcode::Login, payload, 0, DEFAULT_TIMEOUT_SECS)
            .await?;

        let raw_payload = data.get("payload").cloned().unwrap_or_default();
        Self::handle_error(&data)?;

        if let Some(chats) = raw_payload.get("chats").and_then(|v| v.as_array()) {
            let mut chats_guard = self.chats.write().await;
            let mut dialogs_guard = self.dialogs.write().await;
            let mut channels_guard = self.channels.write().await;

            for raw_chat in chats {
                let type_str = raw_chat.get("type").and_then(|v| v.as_str()).unwrap_or("");

                match type_str {
                    "DIALOG" => {
                        if let Ok(d) = serde_json::from_value::<Dialog>(raw_chat.clone()) {
                            dialogs_guard.push(d);
                        }
                    }
                    "CHAT" => {
                        if let Ok(c) = serde_json::from_value::<Chat>(raw_chat.clone()) {
                            chats_guard.push(c);
                        }
                    }
                    "CHANNEL" => {
                        if let Ok(c) = serde_json::from_value::<Chat>(raw_chat.clone()) {
                            channels_guard.push(Channel(c));
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(contacts) = raw_payload.get("contacts").and_then(|v| v.as_array()) {
            let mut contacts_guard = self.contacts.write().await;
            for raw_user in contacts {
                if let Ok(u) = serde_json::from_value::<User>(raw_user.clone()) {
                    contacts_guard.push(u);
                }
            }
        }

        if let Some(me_data) = raw_payload.get("profile").and_then(|p| p.get("contact")) {
            if let Ok(me) = serde_json::from_value::<Me>(me_data.clone()) {
                let mut me_guard = self.me.write().await;
                *me_guard = Some(me);
            }
        }

        info!(
            "Sync completed: dialogs={} chats={} channels={}",
            self.dialogs.read().await.len(),
            self.chats.read().await.len(),
            self.channels.read().await.len()
        );

        Ok(())
    }

    async fn post_login_tasks(&self) -> MaxResult<()> {
        let state_clone = Arc::clone(&self.state);
        tokio::spawn(async move {
            run_ping_loop(state_clone).await;
        });

        let on_start = self.state.on_start_handler.read().await;
        if let Some(handler) = on_start.as_ref() {
            let fut = handler();
            tokio::spawn(fut);
        }

        self.run_scheduled_tasks();

        Ok(())
    }

    fn run_scheduled_tasks(&self) {
        let tasks = {
            let rt = self
                .state
                .scheduled_tasks
                .try_read()
                .ok()
                .map(|g| g.iter().map(|(h, d)| (h.clone(), *d)).collect::<Vec<_>>());
            rt.unwrap_or_default()
        };

        for (handler, interval) in tasks {
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(interval).await;
                    let fut = handler();
                    fut.await;
                }
            });
        }
    }

    async fn wait_for_disconnect(&self) {
        loop {
            if !self.is_connected() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn cleanup(&self) {
        match &self.transport {
            TransportKind::WebSocket(ws) => ws.disconnect(&self.state).await,
            TransportKind::Socket(sock) => sock.disconnect(&self.state).await,
        }

        let mut chats = self.chats.write().await;
        chats.clear();
        let mut dialogs = self.dialogs.write().await;
        dialogs.clear();
        let mut channels = self.channels.write().await;
        channels.clear();
    }

    pub async fn stop(&self) {
        info!("Stopping client");
        let mut guard = self.stop_tx.write().await;
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(());
        }
        *guard = None;
    }

    pub async fn reconnect(&self) -> MaxResult<()> {
        if !self.config.reconnect {
            return Err(MaxError::Other(
                "Reconnect is disabled; set config.reconnect = true".to_string(),
            ));
        }

        let guard = self.stop_tx.read().await;
        let tx = guard.as_ref().ok_or_else(|| {
            MaxError::Other("Client is not running; call start() first".to_string())
        })?;
        tx.send(())
            .map_err(|_| MaxError::Other("Client is not running; call start() first".to_string()))?;

        Ok(())
    }

    pub async fn close(&self) {
        self.stop().await;
    }

    pub async fn idle(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }

    pub async fn login_with_code(
        &self,
        temp_token: &str,
        code: &str,
        start: bool,
    ) -> MaxResult<()> {
        let resp = self.send_code(code, temp_token).await?;

        let login_attrs = resp
            .get("tokenAttrs")
            .and_then(|t| t.get("LOGIN"))
            .cloned()
            .unwrap_or_default();

        let password_challenge = resp.get("passwordChallenge").cloned();

        let token = if password_challenge.is_some() && login_attrs.is_null() {
            let challenge = password_challenge.unwrap();
            self.two_factor_auth(&challenge).await?
        } else {
            login_attrs
                .get("token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    MaxError::Other("Login response did not contain token".to_string())
                })?
        };

        {
            let mut guard = self.token.write().await;
            *guard = Some(token.clone());
        }
        let _ = self.db.update_auth_token(&self.device_id, &token);

        if start {
            self.start().await?;
        } else {
            info!("Login successful, token saved");
        }

        Ok(())
    }

    pub fn inspect(&self) {
        info!("MaxApiKernel");
        info!("---------");
        info!("Connected: {}", self.is_connected());
    }
}

async fn run_ping_loop(state: Arc<ClientState>) {
    let interval = Duration::from_secs_f64(DEFAULT_PING_INTERVAL_SECS);
    loop {
        tokio::time::sleep(interval).await;
        if !state.is_connected.load(Ordering::SeqCst) {
            break;
        }
    }
}

impl MaxClient {
    async fn send_code(&self, code: &str, token: &str) -> MaxResult<Value> {
        use crate::enums::AuthType;
        use crate::payloads::SendCodePayload;

        let payload = serde_json::to_value(SendCodePayload {
            token: token.to_string(),
            verify_code: code.to_string(),
            auth_token_type: AuthType::CheckCode,
        })?;

        let data = self.send_default(Opcode::Auth, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload").cloned().ok_or_else(|| {
            MaxError::ResponseStructure("No payload in send_code response".to_string())
        })
    }

    async fn two_factor_auth(&self, _challenge: &Value) -> MaxResult<String> {
        Err(MaxError::Other(
            "Two-factor auth requires interactive input; use login_with_code with the password flow".to_string(),
        ))
    }
}
