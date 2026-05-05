use max_api_kernel::MaxError;
use serde_json::json;

#[test]
fn from_response_generic_api_error() {
    let data = json!({
        "payload": {
            "error": "not.found",
            "message": "User not found",
            "title": "Error"
        }
    });
    let err = MaxError::from_response(&data);
    assert!(matches!(err, MaxError::Api { .. }));
    let msg = err.to_string();
    assert!(msg.contains("not.found"), "display: {msg}");
    assert!(msg.contains("User not found"), "display: {msg}");
}

#[test]
fn from_response_rate_limit() {
    let data = json!({
        "payload": {
            "error": "too.many.requests",
            "message": "Slow down please",
            "title": "Rate Limited"
        }
    });
    let err = MaxError::from_response(&data);
    assert!(matches!(err, MaxError::RateLimit { .. }));
    let msg = err.to_string();
    assert!(msg.contains("too.many.requests"), "display: {msg}");
}

#[test]
fn from_response_with_localized_message() {
    let data = json!({
        "payload": {
            "error": "some.error",
            "message": "msg",
            "title": "t",
            "localizedMessage": "Локальное сообщение"
        }
    });
    let err = MaxError::from_response(&data);
    if let MaxError::Api {
        localized_message, ..
    } = err
    {
        assert_eq!(localized_message.as_deref(), Some("Локальное сообщение"));
    } else {
        panic!("Expected MaxError::Api");
    }
}

#[test]
fn from_response_no_payload_uses_defaults() {
    let data = json!({});
    let err = MaxError::from_response(&data);
    assert!(matches!(err, MaxError::Api { .. }));
    let msg = err.to_string();
    assert!(msg.contains("unknown"), "display: {msg}");
}

#[test]
fn invalid_phone_display() {
    let err = MaxError::InvalidPhone("+123".to_string());
    let msg = err.to_string();
    assert!(msg.contains("+123"), "display: {msg}");
    assert!(msg.contains("Invalid phone"), "display: {msg}");
}

#[test]
fn timeout_display() {
    let err = MaxError::Timeout;
    assert_eq!(err.to_string(), "Timeout waiting for response");
}

#[test]
fn channel_closed_display() {
    let err = MaxError::ChannelClosed;
    assert_eq!(err.to_string(), "Channel closed");
}

#[test]
fn other_error_display() {
    let err = MaxError::Other("something went wrong".to_string());
    assert_eq!(err.to_string(), "something went wrong");
}

#[test]
fn websocket_not_connected_display() {
    let err = MaxError::WebSocketNotConnected;
    assert_eq!(err.to_string(), "WebSocket is not connected");
}

#[test]
fn json_error_from_conversion() {
    let json_err: serde_json::Error = serde_json::from_str::<i32>("not-a-number").unwrap_err();
    let err: MaxError = json_err.into();
    assert!(matches!(err, MaxError::Json(_)));
    assert!(err.to_string().contains("JSON error"));
}
