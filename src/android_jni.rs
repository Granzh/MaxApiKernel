// Copyright (c) 2026 FlintWithBlackCrown
// JNI bridge for Android integration

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong, jstring};
use jni::JNIEnv;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::client::{ClientConfig, MaxClient};
use crate::payloads::UserAgentPayload;

pub struct NativeHandle {
    pub rt: Runtime,
    pub client: Arc<MaxClient>,
}

unsafe impl Send for NativeHandle {}
unsafe impl Sync for NativeHandle {}

fn ptr_to_handle(ptr: jlong) -> Option<&'static NativeHandle> {
    if ptr == 0 {
        return None;
    }
    Some(unsafe { &*(ptr as *const NativeHandle) })
}

/// Create a native MaxClient handle. Returns 0 on failure.
/// phone: e.g. "+79991234567", workDir: Android files directory path.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    j_phone: JString,
    j_work_dir: JString,
) -> jlong {
    let phone: String = match env.get_string(&j_phone) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };
    let work_dir: String = match env.get_string(&j_work_dir) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };

    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(_) => return 0,
    };

    let config = ClientConfig {
        phone,
        work_dir,
        user_agent: UserAgentPayload::for_desktop(),
        reconnect: false,
        ..Default::default()
    };

    let client = match MaxClient::with_config(config) {
        Ok(c) => Arc::new(c),
        Err(_) => return 0,
    };

    Box::into_raw(Box::new(NativeHandle { rt, client })) as jlong
}

/// Free native handle.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    if ptr != 0 {
        unsafe {
            drop(Box::from_raw(ptr as *mut NativeHandle));
        }
    }
}

/// Connect WebSocket and perform session handshake. Must be called before auth operations.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeConnect(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jboolean {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return 0,
    };
    match h.rt.block_on(h.client.connect_transport()) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// Check if a saved auth token exists (no network needed).
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeIsAuthenticated(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jboolean {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return 0,
    };
    match h.client.db_has_token() {
        true => 1,
        false => 0,
    }
}

/// Request SMS code for the phone number. Returns temp token string, or empty string on error.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeRequestCode<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass,
    ptr: jlong,
    j_phone: JString,
) -> jstring {
    let empty = env.new_string("").unwrap().into_raw();

    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return empty,
    };
    let phone: String = match env.get_string(&j_phone) {
        Ok(s) => s.into(),
        Err(_) => return empty,
    };

    match h.rt.block_on(h.client.request_code(&phone, "ru")) {
        Ok(token) => env.new_string(token).map(|s| s.into_raw()).unwrap_or(empty),
        Err(_) => empty,
    }
}

/// Verify SMS code and save auth token. Returns 1 on success.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeLoginWithCode(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    j_token: JString,
    j_code: JString,
) -> jboolean {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return 0,
    };
    let token: String = match env.get_string(&j_token) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };
    let code: String = match env.get_string(&j_code) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };

    match h.rt.block_on(h.client.login_with_code(&token, &code, false)) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// Sync data (chats, me info) after login. Returns 1 on success.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeSyncData(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jboolean {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return 0,
    };
    match h.rt.block_on(h.client.sync_data()) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// Get authenticated user's ID. Returns 0 if not available.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeGetMyUserId(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jlong {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return 0,
    };
    h.rt.block_on(async {
        let me = h.client.me.read().await;
        me.as_ref().map(|m| m.id).unwrap_or(0)
    })
}

/// Send a text message to the given chatId. Returns 1 on success.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeSendMessage(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    chat_id: jlong,
    j_text: JString,
) -> jboolean {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return 0,
    };
    let text: String = match env.get_string(&j_text) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };

    match h.rt.block_on(h.client.send_message(chat_id, &text, true, None, None)) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// Compute Max chatId for a 1-on-1 dialog. For Favorites: get_chat_id(userId, userId) should
/// be avoided — pass the actual self-dialog cid found after sync_data instead.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeGetChatId(
    _env: JNIEnv,
    _class: JClass,
    _ptr: jlong,
    first_user_id: jlong,
    second_user_id: jlong,
) -> jlong {
    MaxClient::get_chat_id(first_user_id, second_user_id)
}

/// Stop the background event loop.
#[no_mangle]
pub extern "system" fn Java_org_telegram_messenger_max_MaxApiJni_nativeStop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let h = match ptr_to_handle(ptr) {
        Some(h) => h,
        None => return,
    };
    h.rt.block_on(h.client.stop());
}