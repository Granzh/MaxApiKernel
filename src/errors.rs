use thiserror::Error;

#[derive(Debug, Error)]
pub enum MaxError {
    #[error("Invalid phone number format: {0}")]
    InvalidPhone(String),

    #[error("WebSocket is not connected")]
    WebSocketNotConnected,

    #[error("Socket is not connected")]
    SocketNotConnected,

    #[error("Send and wait failed (socket)")]
    SocketSend,

    #[error("Response error: {0}")]
    Response(String),

    #[error("Response structure error: {0}")]
    ResponseStructure(String),

    #[error("API Error [{error}]: {message} ({title})")]
    Api {
        error: String,
        message: String,
        title: String,
        localized_message: Option<String>,
    },

    #[error("Rate limit exceeded [{error}]: {message}")]
    RateLimit {
        error: String,
        message: String,
        title: String,
        localized_message: Option<String>,
    },

    #[error("Login error [{error}]: {message}")]
    Login {
        error: String,
        message: String,
        title: String,
        localized_message: Option<String>,
    },

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Channel closed")]
    ChannelClosed,

    #[error("{0}")]
    Other(String),
}

pub type MaxResult<T> = Result<T, MaxError>;

impl MaxError {
    pub fn from_response(data: &serde_json::Value) -> Self {
        let payload = data.get("payload").and_then(|p| p.as_object());
        let error = payload
            .and_then(|p| p.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let message = payload
            .and_then(|p| p.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = payload
            .and_then(|p| p.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let localized_message = payload
            .and_then(|p| p.get("localizedMessage"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if error == "too.many.requests" {
            return MaxError::RateLimit {
                error,
                message,
                title,
                localized_message,
            };
        }

        MaxError::Api {
            error,
            message,
            title,
            localized_message,
        }
    }
}
