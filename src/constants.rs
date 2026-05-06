// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use rand::seq::SliceRandom;
use rand::Rng;

pub const WEBSOCKET_URI: &str = "wss://ws-api.oneme.ru/websocket";
pub const WEBSOCKET_ORIGIN: &str = "https://web.max.ru";
pub const HOST: &str = "api.oneme.ru";
pub const PORT: u16 = 443;
pub const DEFAULT_TIMEOUT_SECS: f64 = 20.0;
pub const DEFAULT_PING_INTERVAL_SECS: f64 = 30.0;
pub const RECV_LOOP_BACKOFF_SECS: f64 = 0.5;
pub const SESSION_STORAGE_DB: &str = "session.db";
pub const DEFAULT_LOCALE: &str = "ru";
pub const DEFAULT_DEVICE_LOCALE: &str = "ru";
pub const DEFAULT_APP_VERSION: &str = "25.12.14";
pub const DEFAULT_BUILD_NUMBER: i32 = 0x97CB;
pub const DEFAULT_SCREEN: &str = "1080x1920 1.0x";
pub const DEFAULT_CHAT_MEMBERS_LIMIT: i32 = 50;
pub const PHONE_REGEX: &str = r"^\+?\d{10,15}$";

const DEVICE_NAMES: &[&str] = &[
    "Chrome",
    "Firefox",
    "Edge",
    "Safari",
    "Opera",
    "Vivaldi",
    "Brave",
    "Chromium",
    "Windows 10",
    "Windows 11",
    "macOS Big Sur",
    "macOS Monterey",
    "macOS Ventura",
    "Ubuntu 20.04",
    "Ubuntu 22.04",
    "Fedora 35",
    "Fedora 36",
    "Debian 11",
];

const OS_VERSIONS: &[&str] = &[
    "Windows 10",
    "Windows 11",
    "macOS Big Sur",
    "macOS Monterey",
    "macOS Ventura",
    "Ubuntu 20.04",
    "Ubuntu 22.04",
    "Fedora 35",
    "Fedora 36",
    "Debian 11",
];

const TIMEZONES: &[&str] = &[
    "Europe/Moscow",
    "Europe/Kaliningrad",
    "Europe/Samara",
    "Asia/Yekaterinburg",
    "Asia/Omsk",
    "Asia/Krasnoyarsk",
    "Asia/Irkutsk",
    "Asia/Yakutsk",
    "Asia/Vladivostok",
    "Asia/Kamchatka",
];

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_2_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
];

pub fn random_device_name() -> &'static str {
    let mut rng = rand::thread_rng();
    DEVICE_NAMES.choose(&mut rng).copied().unwrap_or("Chrome")
}

pub fn random_os_version() -> &'static str {
    let mut rng = rand::thread_rng();
    OS_VERSIONS
        .choose(&mut rng)
        .copied()
        .unwrap_or("Windows 10")
}

pub fn random_timezone() -> &'static str {
    let mut rng = rand::thread_rng();
    TIMEZONES
        .choose(&mut rng)
        .copied()
        .unwrap_or("Europe/Moscow")
}

pub fn random_user_agent() -> &'static str {
    let mut rng = rand::thread_rng();
    USER_AGENTS
        .choose(&mut rng)
        .copied()
        .unwrap_or(USER_AGENTS[0])
}

pub fn random_client_session_id() -> i32 {
    rand::thread_rng().gen_range(1..=15)
}
