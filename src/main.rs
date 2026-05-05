use max_api_kernel::MaxClient;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();

    match parse_phone_arg(&args) {
        Some(phone) => run_client(phone).await?,
        None => (),
    }

    Ok(())
}

fn parse_phone_arg(args: &[String]) -> Option<&str> {
    for i in 0..args.len() {
        if args[i] == "--phone" {
            return args.get(i + 1).map(|s| s.as_str());
        }
    }
    args.get(1)
        .filter(|a| a.starts_with('+') || a.chars().next().map_or(false, |c| c.is_ascii_digit()))
        .map(|s| s.as_str())
}

async fn run_client(phone: &str) -> anyhow::Result<()> {
    println!("Запуск MaxApiKernel для {}...", phone);

    let client = Arc::new(MaxClient::new(phone)?);

    {
        let c = Arc::clone(&client);
        client.on_start(move || {
            let c = Arc::clone(&c);
            async move {
                let me_guard = c.me.read().await;
                if let Some(me) = me_guard.as_ref() {
                    let name = me
                        .names
                        .first()
                        .and_then(|n| n.name.as_deref().or(n.first_name.as_deref()))
                        .unwrap_or("(без имени)");
                    println!("\n{}", "=".repeat(50));
                    println!("Подключено! Мой ID: {}", me.id);
                    println!("  Имя: {} | Телефон: {}", name, me.phone);
                    println!("{}", "=".repeat(50));
                } else {
                    println!("\n{}", "=".repeat(50));
                    println!("Подключено! (профиль ещё не загружен)");
                    println!("{}", "=".repeat(50));
                }
                drop(me_guard);

                println!("\nИстория из Избранного (последние 5):");
                match c.fetch_history(0, None, 0, 5).await {
                    Ok(msgs) if !msgs.is_empty() => {
                        for msg in &msgs {
                            let preview: String = if msg.text.is_empty() {
                                format!("[вложение: {} шт.]", msg.attaches.len())
                            } else {
                                msg.text.chars().take(80).collect()
                            };
                            println!("  - {}", preview);
                        }
                    }
                    Ok(_) => println!("  (пусто)"),
                    Err(e) => println!("  (ошибка при загрузке истории: {})", e),
                }

                println!("\nСлушаем входящие сообщения... (Ctrl+C для выхода)\n");
            }
        });
    }

    client.on_message(|msg| async move {
        let chat = msg
            .chat_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "?".to_string());
        let sender = msg
            .sender
            .map(|id| id.to_string())
            .unwrap_or_else(|| "?".to_string());
        let text = if msg.text.is_empty() {
            "[нет текста]".to_string()
        } else {
            msg.text.clone()
        };
        let attach_info = if msg.attaches.is_empty() {
            String::new()
        } else {
            format!(" [{} вложений]", msg.attaches.len())
        };

        println!("Новое сообщение в чате {}:", chat);
        println!("  От:    {}", sender);
        println!("  Текст: {}{}", text, attach_info);
    });

    {
        let c = Arc::clone(&client);
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                println!("\nОстановлено (Ctrl+C)");
                c.close().await;
            }
        });
    }

    client.start().await?;
    Ok(())
}
