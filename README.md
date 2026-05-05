# MaxApiKernel

Rust‑клиент для Max API: авторизация, сессии, сообщения, события и управление соединением. Подходит как ядро/библиотека для встраивания в бэкенд или другие приложения.

## Установка

Проект не опубликован в crates.io, используйте зависимость по пути или git:

```toml
[dependencies]
max_api_kernel = { path = "../MaxApiKernel" }
```

## Быстрый старт

```rust
use max_api_kernel::MaxClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = MaxClient::new("+79991234567")?;

    client.on_message(|msg| async move {
        println!("New message: {:?}", msg.text);
    });

    client.start().await?;
    Ok(())
}
```

## Авторизация и сессия

Сессия сохраняется в `session.db` внутри `work_dir` (по умолчанию текущая директория). При наличии токена повторный вход не требуется.

По умолчанию для WEB используется QR‑логин. Для SMS‑логина используйте socket‑клиент:

```rust
use max_api_kernel::MaxClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = MaxClient::new_socket("+79991234567", "messenger.max.ru", 443)?;
    let temp = client.request_code("+79991234567", "ru").await?;
    client.login_with_code(&temp, "123456", false).await?;
    client.start().await?;
    Ok(())
}
```

## Управление соединением

- `start()` — запускает соединение и основной цикл.
- `stop()` / `close()` — останавливает клиент.
- `reconnect()` — инициирует переподключение (требуется запущенный `start()` и `config.reconnect = true`).

## Сообщения

Основные методы:

```rust
client.send_message(chat_id, "hi", true, None, None).await?;
let history = client.fetch_history(chat_id, None, 0, 20).await?;
client.read_message(message_id, chat_id).await?;
```

## События

Ключевой хук:

```rust
client.on_message(|msg| async move { /* ... */ });
```

Также доступны `on_message_edit`, `on_message_delete`, `on_raw_receive`, `on_start`.

## Чаты и chat_id

Для диалогов 1‑на‑1 можно вычислить `chat_id` из двух `user_id`:

```rust
let chat_id = MaxClient::get_chat_id(my_user_id, other_user_id);
```

## Конфигурация

Используйте `ClientConfig` для настройки `work_dir`, `reconnect`, прокси и user‑agent:

```rust
use max_api_kernel::{ClientConfig, MaxClient};
use max_api_kernel::payloads::UserAgentPayload;

let cfg = ClientConfig {
    phone: "+79991234567".to_string(),
    work_dir: "./data".to_string(),
    reconnect: true,
    reconnect_delay: std::time::Duration::from_secs(2),
    user_agent: UserAgentPayload::for_web(),
    ..Default::default()
};
let client = MaxClient::with_config(cfg)?;
```

## Пример CLI

В репозитории есть пример `src/main.rs`:

```bash
cargo run -- --phone +79991234567
```

## Тесты

```bash
cargo test
```
