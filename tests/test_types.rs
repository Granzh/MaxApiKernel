use max_api_kernel::{Attach, AttachType, Message, VideoRequest};
use serde_json::json;

// --- Attach deserialization ---

#[test]
fn attach_photo_deser() {
    let data = json!({
        "_type": "PHOTO",
        "baseUrl": "https://example.com/",
        "height": 100,
        "width": 200,
        "photoId": 999,
        "photoToken": "tok"
    });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Photo(_)));
    assert_eq!(a.attach_type(), AttachType::Photo);
}

#[test]
fn attach_video_deser() {
    let data = json!({
        "_type": "VIDEO",
        "height": 720,
        "width": 1280,
        "videoId": 42,
        "duration": 30
    });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Video(_)));
    assert_eq!(a.attach_type(), AttachType::Video);
}

#[test]
fn attach_file_deser() {
    let data = json!({
        "_type": "FILE",
        "fileId": 1,
        "name": "test.txt",
        "size": 42,
        "token": "tok"
    });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::File(_)));
    assert_eq!(a.attach_type(), AttachType::File);
}

#[test]
fn attach_sticker_deser() {
    let data = json!({ "_type": "STICKER" });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Sticker(_)));
    assert_eq!(a.attach_type(), AttachType::Sticker);
}

#[test]
fn attach_audio_deser() {
    let data = json!({ "_type": "AUDIO" });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Audio(_)));
    assert_eq!(a.attach_type(), AttachType::Audio);
}

#[test]
fn attach_contact_deser() {
    let data = json!({ "_type": "CONTACT" });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Contact(_)));
    assert_eq!(a.attach_type(), AttachType::Contact);
}

#[test]
fn attach_control_deser() {
    let data = json!({ "_type": "CONTROL", "event": "typing" });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Control(_)));
    assert_eq!(a.attach_type(), AttachType::Control);
}

#[test]
fn attach_unknown_type_deser() {
    let data = json!({ "_type": "FUTURE_UNKNOWN", "foo": "bar" });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Unknown));
}

#[test]
fn attach_missing_type_field() {
    let data = json!({ "foo": "bar" });
    let a: Attach = serde_json::from_value(data).unwrap();
    assert!(matches!(a, Attach::Unknown));
}

#[test]
fn attach_photo_serialize_roundtrip() {
    let data = json!({
        "_type": "PHOTO",
        "baseUrl": "https://x.com/",
        "height": 50,
        "width": 50,
        "photoId": 1,
        "photoToken": "t"
    });
    let a: Attach = serde_json::from_value(data).unwrap();
    let serialized = serde_json::to_value(&a).unwrap();
    assert_eq!(serialized["_type"], "PHOTO");
    assert_eq!(serialized["photoId"], 1);
}

// --- Message::from_value ---

#[test]
fn message_from_value_flat_fields() {
    let data = json!({
        "id": 100,
        "time": 1_000_000,
        "chatId": 55,
        "text": "hello"
    });
    let msg = Message::from_value(&data).unwrap();
    assert_eq!(msg.id, 100);
    assert_eq!(msg.time, 1_000_000);
    assert_eq!(msg.chat_id, Some(55));
    assert_eq!(msg.text, "hello");
}

#[test]
fn message_from_value_nested_message_key() {
    let data = json!({
        "chatId": 77,
        "message": {
            "id": 200,
            "time": 2000,
            "text": "nested msg"
        }
    });
    let msg = Message::from_value(&data).unwrap();
    assert_eq!(msg.id, 200);
    assert_eq!(msg.text, "nested msg");
    assert_eq!(msg.chat_id, Some(77));
}

#[test]
fn message_from_value_chat_id_from_nested_data() {
    // chat_id inside message takes priority; outer chatId as fallback
    let data = json!({
        "chatId": 99,
        "message": {
            "id": 1,
            "time": 0,
            "chatId": 11
        }
    });
    let msg = Message::from_value(&data).unwrap();
    assert_eq!(msg.chat_id, Some(11));
}

#[test]
fn message_default_empty_text() {
    let msg = Message::default();
    assert_eq!(msg.text, "");
    assert!(msg.attaches.is_empty());
    assert!(msg.elements.is_empty());
}

// --- VideoRequest::from_value ---

#[test]
fn video_request_from_value() {
    let data = json!({
        "EXTERNAL": "hls",
        "cache": true,
        "stream_url": "https://cdn.example.com/video.m3u8"
    });
    let vr = VideoRequest::from_value(&data).unwrap();
    assert_eq!(vr.external, "hls");
    assert!(vr.cache);
    assert_eq!(vr.url, "https://cdn.example.com/video.m3u8");
}

#[test]
fn video_request_missing_required_field_returns_none() {
    let data = json!({ "cache": true });
    assert!(VideoRequest::from_value(&data).is_none());
}
