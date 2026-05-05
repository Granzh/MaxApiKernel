use max_api_kernel::{
    AuthType, Capability, ChatType, ContactAction, DeviceType, MessageStatus, Opcode, ReadAction,
};

#[test]
fn opcode_roundtrip_known_values() {
    let cases: &[(i32, Opcode)] = &[
        (1, Opcode::Ping),
        (6, Opcode::SessionInit),
        (19, Opcode::Login),
        (21, Opcode::Sync),
        (64, Opcode::MsgSend),
        (66, Opcode::MsgDelete),
        (128, Opcode::NotifMessage),
        (272, Opcode::FoldersGet),
        (288, Opcode::GetQr),
        (291, Opcode::LoginByQr),
    ];
    for (n, variant) in cases {
        let from = Opcode::from(*n);
        assert_eq!(from, *variant, "Opcode::from({n}) mismatch");
        let back = i32::from(from);
        assert_eq!(back, *n, "i32::from(Opcode) roundtrip failed for {n}");
    }
}

#[test]
fn opcode_unknown_roundtrip() {
    let op = Opcode::from(9999_i32);
    assert_eq!(op, Opcode::Unknown(9999));
    assert_eq!(i32::from(op), 9999);
}

#[test]
fn opcode_value_method() {
    assert_eq!(Opcode::Ping.value(), 1);
    assert_eq!(Opcode::MsgSend.value(), 64);
    assert_eq!(Opcode::LoginByQr.value(), 291);
}

#[test]
fn device_type_as_str() {
    assert_eq!(DeviceType::Web.as_str(), "WEB");
    assert_eq!(DeviceType::Android.as_str(), "ANDROID");
    assert_eq!(DeviceType::Ios.as_str(), "IOS");
    assert_eq!(DeviceType::Desktop.as_str(), "DESKTOP");
}

#[test]
fn read_action_as_str() {
    assert_eq!(ReadAction::ReadMessage.as_str(), "READ_MESSAGE");
    assert_eq!(ReadAction::ReadReaction.as_str(), "READ_REACTION");
}

#[test]
fn contact_action_as_str() {
    assert_eq!(ContactAction::Add.as_str(), "ADD");
    assert_eq!(ContactAction::Remove.as_str(), "REMOVE");
}

#[test]
fn capability_into_i32() {
    assert_eq!(i32::from(Capability::Default), 0);
    assert_eq!(i32::from(Capability::EsiaVerifiedFlag), 1);
    assert_eq!(i32::from(Capability::SecondFactorPasswordEnabled), 2);
    assert_eq!(i32::from(Capability::SecondFactorHasEmail), 3);
    assert_eq!(i32::from(Capability::SecondFactorHasHint), 4);
}

#[test]
fn chat_type_serde_roundtrip() {
    for (json, expected) in [
        (r#""DIALOG""#, ChatType::Dialog),
        (r#""CHAT""#, ChatType::Chat),
        (r#""CHANNEL""#, ChatType::Channel),
    ] {
        let parsed: ChatType = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }
}

#[test]
fn message_status_serde() {
    let edited: MessageStatus = serde_json::from_str(r#""EDITED""#).unwrap();
    assert_eq!(edited, MessageStatus::Edited);

    let removed: MessageStatus = serde_json::from_str(r#""REMOVED""#).unwrap();
    assert_eq!(removed, MessageStatus::Removed);

    assert_eq!(
        serde_json::to_string(&MessageStatus::Edited).unwrap(),
        r#""EDITED""#
    );
}

#[test]
fn auth_type_serde() {
    let v: AuthType = serde_json::from_str(r#""START_AUTH""#).unwrap();
    assert_eq!(v, AuthType::StartAuth);

    let v: AuthType = serde_json::from_str(r#""CHECK_CODE""#).unwrap();
    assert_eq!(v, AuthType::CheckCode);
}

#[test]
fn opcode_serde_via_i32() {
    let json = serde_json::to_string(&Opcode::MsgSend).unwrap();
    assert_eq!(json, "64");
    let back: Opcode = serde_json::from_str("64").unwrap();
    assert_eq!(back, Opcode::MsgSend);
}
