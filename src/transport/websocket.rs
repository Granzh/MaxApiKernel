use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http::{Request, Uri};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::constants::{RECV_LOOP_BACKOFF_SECS, WEBSOCKET_ORIGIN};
use crate::enums::Opcode;
use crate::errors::{MaxError, MaxResult};
use crate::transport::{dispatch_incoming, make_message, ClientState};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, WsMessage>;

pub struct WebSocketTransport {
    pub uri: String,
    pub user_agent: String,
    pub proxy: Option<String>,
    sink: Arc<Mutex<Option<WsSink>>>,
}

impl WebSocketTransport {
    pub fn new(uri: String, user_agent: String, proxy: Option<String>) -> Self {
        Self {
            uri,
            user_agent,
            proxy,
            sink: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self, state: Arc<ClientState>) -> MaxResult<()> {
        let uri = self
            .uri
            .parse::<Uri>()
            .map_err(|e| MaxError::Other(format!("Invalid URI: {}", e)))?;
        let host = uri
            .host()
            .ok_or_else(|| MaxError::Other("URI has no host".to_string()))?
            .to_string();

        // When a custom Request is passed, tungstenite does NOT add WebSocket
        // handshake headers automatically — they must all be present here.
        let request = Request::builder()
            .uri(&self.uri)
            .header("Host", &host)
            .header("Origin", WEBSOCKET_ORIGIN)
            .header("User-Agent", &self.user_agent)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .body(())
            .map_err(|e| MaxError::Other(format!("Failed to build request: {}", e)))?;

        info!("Connecting to WebSocket {}", self.uri);

        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(MaxError::WebSocket)?;

        let (ws_sink, ws_source) = ws_stream.split();

        {
            let mut sink_guard = self.sink.lock().await;
            *sink_guard = Some(ws_sink);
        }

        state
            .is_connected
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let state_clone = Arc::clone(&state);
        let sink_clone = Arc::clone(&self.sink);
        tokio::spawn(async move {
            recv_loop(ws_source, state_clone, sink_clone).await;
        });

        info!("WebSocket connected");
        Ok(())
    }

    pub async fn disconnect(&self, state: &Arc<ClientState>) {
        state
            .is_connected
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let mut sink_guard = self.sink.lock().await;
        if let Some(mut sink) = sink_guard.take() {
            let _ = sink.close().await;
        }

        let mut pending = state.pending.lock().await;
        pending.clear();
    }

    pub async fn send_and_wait(
        &self,
        state: &Arc<ClientState>,
        opcode: Opcode,
        payload: Value,
        cmd: i32,
        timeout: Duration,
    ) -> MaxResult<Value> {
        let seq = state.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let msg = make_message(opcode, payload, cmd, seq);

        let (tx, rx) = oneshot::channel::<Value>();

        {
            let mut pending = state.pending.lock().await;
            if let Some(old) = pending.insert(seq, tx) {
                drop(old);
            }
        }

        let json_str = serde_json::to_string(&msg).map_err(MaxError::Json)?;

        {
            let mut sink_guard = self.sink.lock().await;
            match sink_guard.as_mut() {
                Some(sink) => {
                    sink.send(WsMessage::Text(json_str))
                        .await
                        .map_err(MaxError::WebSocket)?;
                }
                None => {
                    let mut pending = state.pending.lock().await;
                    pending.remove(&seq);
                    return Err(MaxError::WebSocketNotConnected);
                }
            }
        }

        debug!("Sent frame opcode={:?} cmd={} seq={}", opcode, cmd, seq);

        tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| {
                let mut p = state.pending.blocking_lock();
                p.remove(&seq);
                MaxError::Timeout
            })?
            .map_err(|_| MaxError::WebSocketNotConnected)
    }
}

async fn recv_loop(
    mut source: futures_util::stream::SplitStream<WsStream>,
    state: Arc<ClientState>,
    sink: Arc<Mutex<Option<WsSink>>>,
) {
    debug!("WebSocket recv loop started");

    while let Some(msg) = source.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                let data = match serde_json::from_str::<Value>(&text) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("JSON parse error: {}", e);
                        continue;
                    }
                };

                let opcode_val = data.get("opcode").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let opcode = Opcode::from(opcode_val);

                if !opcode.is_notification() {
                    let seq = data.get("seq").and_then(|v| v.as_i64()).map(|v| v as i32);

                    if let Some(seq_key) = seq {
                        let mut pending = state.pending.lock().await;
                        if let Some(tx) = pending.remove(&seq_key) {
                            debug!("Resolved pending seq={}", seq_key);
                            let _ = tx.send(data);
                            continue;
                        }
                    }
                }

                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    dispatch_incoming(&state_clone, data).await;
                });
            }
            Ok(WsMessage::Close(frame)) => {
                info!(
                    "WebSocket closed: {:?}",
                    frame.map(|f| f.reason.to_string())
                );
                state
                    .is_connected
                    .store(false, std::sync::atomic::Ordering::SeqCst);

                {
                    let mut pending = state.pending.lock().await;
                    pending.clear();
                }

                let mut sink_guard = sink.lock().await;
                *sink_guard = None;
                break;
            }
            Ok(WsMessage::Ping(data)) => {
                let mut sink_guard = sink.lock().await;
                if let Some(s) = sink_guard.as_mut() {
                    let _ = s.send(WsMessage::Pong(data)).await;
                }
            }
            Ok(_) => {}
            Err(e) => {
                error!("WebSocket recv error: {}; backing off", e);
                tokio::time::sleep(Duration::from_secs_f64(RECV_LOOP_BACKOFF_SECS)).await;
            }
        }
    }

    state
        .is_connected
        .store(false, std::sync::atomic::Ordering::SeqCst);
    debug!("WebSocket recv loop exited");
}
