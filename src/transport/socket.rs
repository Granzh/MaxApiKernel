// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex};
use tokio_native_tls::TlsStream;
use tracing::{debug, error, info, warn};

use crate::constants::RECV_LOOP_BACKOFF_SECS;
use crate::enums::Opcode;
use crate::errors::{MaxError, MaxResult};
use crate::transport::{dispatch_incoming, ClientState};

type TlsTcpStream = TlsStream<TcpStream>;

pub struct SocketTransport {
    pub host: String,
    pub port: u16,
    write_half: Arc<Mutex<Option<tokio::io::WriteHalf<TlsTcpStream>>>>,
}

impl SocketTransport {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            write_half: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self, state: Arc<ClientState>) -> MaxResult<()> {
        info!("Connecting to socket {}:{}", self.host, self.port);

        let tcp = TcpStream::connect((self.host.as_str(), self.port))
            .await
            .map_err(MaxError::Io)?;

        let tls_connector = native_tls::TlsConnector::builder()
            .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
            .build()
            .map_err(MaxError::Tls)?;

        let connector = tokio_native_tls::TlsConnector::from(tls_connector);
        let tls_stream = connector
            .connect(&self.host, tcp)
            .await
            .map_err(|e| MaxError::Other(format!("TLS connect: {}", e)))?;

        let (read_half, write_half) = tokio::io::split(tls_stream);

        {
            let mut guard = self.write_half.lock().await;
            *guard = Some(write_half);
        }

        state
            .is_connected
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let state_clone = Arc::clone(&state);
        let write_clone = Arc::clone(&self.write_half);
        tokio::spawn(async move {
            recv_loop(read_half, state_clone, write_clone).await;
        });

        info!("Socket connected");
        Ok(())
    }

    pub async fn disconnect(&self, state: &Arc<ClientState>) {
        state
            .is_connected
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let mut guard = self.write_half.lock().await;
        *guard = None;

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
        if !state.is_connected.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(MaxError::SocketNotConnected);
        }

        let seq = state.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let seq_key = (seq % 256 + 256) % 256;

        let packet = pack_packet(11, cmd, seq, opcode.value(), &payload)?;

        let (tx, rx) = oneshot::channel::<Value>();

        {
            let mut pending = state.pending.lock().await;
            if let Some(old) = pending.insert(seq_key, tx) {
                drop(old);
            }
        }

        {
            let mut guard = self.write_half.lock().await;
            match guard.as_mut() {
                Some(w) => {
                    w.write_all(&packet).await.map_err(MaxError::Io)?;
                }
                None => {
                    let mut pending = state.pending.lock().await;
                    pending.remove(&seq_key);
                    return Err(MaxError::SocketNotConnected);
                }
            }
        }

        debug!("Sent socket frame opcode={:?} seq={}", opcode, seq);

        tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| {
                let mut p = state.pending.blocking_lock();
                p.remove(&seq_key);
                MaxError::Timeout
            })?
            .map_err(|_| MaxError::SocketNotConnected)
    }
}

fn pack_packet(ver: i32, cmd: i32, seq: i32, opcode: i32, payload: &Value) -> MaxResult<Vec<u8>> {
    let payload_bytes = rmp_serde::to_vec(payload)
        .map_err(|e| MaxError::Other(format!("msgpack encode: {}", e)))?;

    let payload_len = payload_bytes.len() & 0xFFFFFF;

    let mut buf = Vec::with_capacity(10 + payload_bytes.len());
    buf.push(ver as u8);
    buf.extend_from_slice(&(cmd as u16).to_be_bytes());
    buf.push((seq % 256) as u8);
    buf.extend_from_slice(&(opcode as u16).to_be_bytes());
    buf.extend_from_slice(&(payload_len as u32).to_be_bytes());
    buf.extend_from_slice(&payload_bytes);

    Ok(buf)
}

fn unpack_packet(data: &[u8]) -> Option<Value> {
    if data.len() < 10 {
        return None;
    }

    let ver = data[0] as i32;
    let cmd = u16::from_be_bytes([data[1], data[2]]) as i32;
    let seq = data[3] as i32;
    let opcode = u16::from_be_bytes([data[4], data[5]]) as i32;
    let packed_len = u32::from_be_bytes([data[6], data[7], data[8], data[9]]);
    let comp_flag = packed_len >> 24;
    let payload_length = (packed_len & 0xFFFFFF) as usize;

    if data.len() < 10 + payload_length {
        return None;
    }

    let payload_bytes = &data[10..10 + payload_length];

    if payload_bytes.is_empty() {
        return Some(serde_json::json!({
            "ver": ver,
            "cmd": cmd,
            "seq": seq,
            "opcode": opcode,
            "payload": null
        }));
    }

    let decompressed;
    let final_bytes = if comp_flag != 0 {
        match lz4_flex::block::decompress(payload_bytes, 1024 * 1024) {
            Ok(d) => {
                decompressed = d;
                &decompressed[..]
            }
            Err(e) => {
                warn!("LZ4 decompress failed: {}", e);
                return None;
            }
        }
    } else {
        payload_bytes
    };

    let payload: Value = match rmp_serde::from_slice(final_bytes) {
        Ok(v) => v,
        Err(e) => {
            warn!("msgpack decode failed: {}", e);
            return None;
        }
    };

    Some(serde_json::json!({
        "ver": ver,
        "cmd": cmd,
        "seq": seq,
        "opcode": opcode,
        "payload": payload
    }))
}

async fn recv_loop(
    mut read_half: tokio::io::ReadHalf<TlsTcpStream>,
    state: Arc<ClientState>,
    _write_half: Arc<Mutex<Option<tokio::io::WriteHalf<TlsTcpStream>>>>,
) {
    debug!("Socket recv loop started");

    loop {
        let mut header = [0u8; 10];
        match read_half.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    info!("Socket connection closed");
                } else {
                    error!("Socket read header error: {}", e);
                }
                break;
            }
        }

        let packed_len = u32::from_be_bytes([header[6], header[7], header[8], header[9]]);
        let payload_length = (packed_len & 0xFFFFFF) as usize;

        let mut payload_buf = vec![0u8; payload_length];
        if payload_length > 0 {
            match read_half.read_exact(&mut payload_buf).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Socket read payload error: {}", e);
                    tokio::time::sleep(Duration::from_secs_f64(RECV_LOOP_BACKOFF_SECS)).await;
                    continue;
                }
            }
        }

        let mut raw = Vec::with_capacity(10 + payload_length);
        raw.extend_from_slice(&header);
        raw.extend_from_slice(&payload_buf);

        let data = match unpack_packet(&raw) {
            Some(d) => d,
            None => {
                warn!("Failed to unpack socket packet");
                continue;
            }
        };

        let seq_raw = data.get("seq").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let seq_key = ((seq_raw % 256) + 256) % 256;

        let payload_arr = data.get("payload").and_then(|p| p.as_array()).cloned();

        let items: Vec<Value> = if let Some(arr) = payload_arr {
            arr.iter()
                .map(|obj| {
                    let mut item = data.clone();
                    item["payload"] = obj.clone();
                    item
                })
                .collect()
        } else {
            vec![data]
        };

        for item in items {
            {
                let mut pending = state.pending.lock().await;
                if let Some(tx) = pending.remove(&seq_key) {
                    debug!("Resolved socket pending seq={}", seq_key);
                    let _ = tx.send(item);
                    continue;
                }
            }

            let state_clone = Arc::clone(&state);
            let item_clone = item.clone();
            tokio::spawn(async move {
                dispatch_incoming(&state_clone, item_clone).await;
            });
        }
    }

    state
        .is_connected
        .store(false, std::sync::atomic::Ordering::SeqCst);
    debug!("Socket recv loop exited");
}
