use serde_json::Value;
use std::io::{self, BufRead, Write as IoWrite};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use crate::enums::{AuthType, Opcode};
use crate::errors::{MaxError, MaxResult};
use crate::payloads::{
    CheckPasswordChallengePayload, CreateTrackPayload, RegisterPayload, RequestCodePayload,
    SendCodePayload, SetHintPayload, SetPasswordPayload, SetTwoFactorPayload,
};

use super::MaxClient;

impl MaxClient {
    pub async fn request_code(&self, phone: &str, language: &str) -> MaxResult<String> {
        info!("Requesting auth code for {}", phone);

        let payload = serde_json::to_value(RequestCodePayload {
            phone: phone.to_string(),
            type_: AuthType::StartAuth,
            language: language.to_string(),
        })?;

        let data = self.send_default(Opcode::AuthRequest, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| p.get("token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No token in request_code response".to_string())
            })
    }

    pub async fn resend_code(&self, phone: &str, language: &str) -> MaxResult<String> {
        info!("Resending auth code for {}", phone);

        let payload = serde_json::to_value(RequestCodePayload {
            phone: phone.to_string(),
            type_: AuthType::Resend,
            language: language.to_string(),
        })?;

        let data = self.send_default(Opcode::AuthRequest, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload")
            .and_then(|p| p.get("token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No token in resend_code response".to_string())
            })
    }

    pub(crate) async fn send_code_internal(&self, code: &str, token: &str) -> MaxResult<Value> {
        info!("Sending verification code");

        let payload = serde_json::to_value(SendCodePayload {
            token: token.to_string(),
            verify_code: code.to_string(),
            auth_token_type: AuthType::CheckCode,
        })?;

        let data = self.send_default(Opcode::Auth, payload).await?;
        Self::handle_error(&data)?;

        data.get("payload").cloned().ok_or_else(|| {
            MaxError::ResponseStructure("No payload in send_code response".to_string())
        })
    }

    pub(crate) async fn login(&self) -> MaxResult<()> {
        info!("Starting login flow");

        let is_web = self.config.user_agent.device_type == "WEB";

        let login_resp = if is_web {
            self.login_by_qr().await?
        } else {
            let temp_token = self.request_code(&self.config.phone, "ru").await?;

            print!("Введите код: ");
            let _ = IoWrite::flush(&mut io::stdout());
            let code = tokio::task::spawn_blocking(|| {
                let stdin = io::stdin();
                stdin
                    .lock()
                    .lines()
                    .next()
                    .unwrap_or(Ok(String::new()))
                    .unwrap_or_default()
            })
            .await
            .map_err(|e| MaxError::Other(format!("stdin read error: {}", e)))?
            .trim()
            .to_string();

            if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err(MaxError::Other("Invalid code format".to_string()));
            }

            self.send_code_internal(&code, &temp_token).await?
        };

        let password_challenge = login_resp.get("passwordChallenge").cloned();
        let login_attrs = login_resp
            .get("tokenAttrs")
            .and_then(|t| t.get("LOGIN"))
            .cloned()
            .unwrap_or(Value::Null);

        let token = if password_challenge.is_some() && login_attrs.is_null() {
            self.two_factor_auth_interactive(&password_challenge.unwrap())
                .await?
        } else {
            login_attrs
                .get("token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    MaxError::Other("Login response did not contain token".to_string())
                })?
        };

        {
            let mut guard = self.token.write().await;
            *guard = Some(token.clone());
        }
        let _ = self.db.update_auth_token(&self.device_id, &token);
        info!("Login successful, token saved");
        Ok(())
    }

    pub(crate) async fn register(
        &self,
        first_name: &str,
        last_name: Option<&str>,
    ) -> MaxResult<()> {
        info!("Starting registration flow");

        let temp_token = self.request_code(&self.config.phone, "ru").await?;

        print!("Введите код: ");
        io::stdout().flush().ok();
        let code = tokio::task::spawn_blocking(|| {
            let stdin = io::stdin();
            stdin
                .lock()
                .lines()
                .next()
                .unwrap_or(Ok(String::new()))
                .unwrap_or_default()
        })
        .await
        .map_err(|e| MaxError::Other(format!("stdin read error: {}", e)))?
        .trim()
        .to_string();

        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(MaxError::Other("Invalid code format".to_string()));
        }

        let reg_resp = self.send_code_internal(&code, &temp_token).await?;

        let reg_token = reg_resp
            .get("tokenAttrs")
            .and_then(|t| t.get("REGISTER"))
            .and_then(|r| r.get("token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| MaxError::Other("Registration token missing".to_string()))?;

        let payload = serde_json::to_value(RegisterPayload {
            first_name: first_name.to_string(),
            last_name: last_name.map(|s| s.to_string()),
            token: reg_token,
            token_type: AuthType::Register,
        })?;

        let data = self.send_default(Opcode::AuthConfirm, payload).await?;
        Self::handle_error(&data)?;

        let token = data
            .get("payload")
            .and_then(|p| p.get("token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                MaxError::Other("Registration response did not contain token".to_string())
            })?;

        {
            let mut guard = self.token.write().await;
            *guard = Some(token.clone());
        }
        let _ = self.db.update_auth_token(&self.device_id, &token);
        info!("Registration successful");
        Ok(())
    }

    async fn login_by_qr(&self) -> MaxResult<Value> {
        info!("Starting QR login flow");

        let data = self
            .send_default(Opcode::GetQr, Value::Object(Default::default()))
            .await?;
        Self::handle_error(&data)?;

        let payload = data
            .get("payload")
            .cloned()
            .ok_or_else(|| MaxError::ResponseStructure("No payload in QR request".to_string()))?;

        let poll_interval = payload
            .get("pollingInterval")
            .and_then(|v| v.as_u64())
            .unwrap_or(2000);
        let link = payload
            .get("qrLink")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MaxError::ResponseStructure("No qrLink in QR response".to_string()))?
            .to_string();
        let track_id = payload
            .get("trackId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MaxError::ResponseStructure("No trackId in QR response".to_string()))?
            .to_string();
        let expires_at = payload
            .get("expiresAt")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        println!("\nQR Code link: {}\n", link);
        self.print_qr_ascii(&link);

        let poll_ms = poll_interval;
        let track_clone = track_id.clone();

        loop {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            if now_ms >= expires_at {
                return Err(MaxError::Other("QR code expired".to_string()));
            }

            let status_data = self
                .send_default(
                    Opcode::GetQrStatus,
                    serde_json::json!({ "trackId": track_clone }),
                )
                .await?;

            let status_payload = status_data.get("payload").cloned().unwrap_or_default();
            if let Some(_e) = status_payload.get("error") {
                return Err(MaxError::from_response(&status_data));
            }

            let login_available = status_payload
                .get("status")
                .and_then(|s| s.get("loginAvailable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if login_available {
                info!("QR login confirmed");
                let login_data = self
                    .send_default(
                        Opcode::LoginByQr,
                        serde_json::json!({ "trackId": track_id }),
                    )
                    .await?;
                Self::handle_error(&login_data)?;
                return login_data.get("payload").cloned().ok_or_else(|| {
                    MaxError::ResponseStructure("No payload in QR login data".to_string())
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(poll_ms)).await;
        }
    }

    fn print_qr_ascii(&self, data: &str) {
        use qrcode::render::unicode;
        use qrcode::QrCode;

        println!("\nОтсканируйте QR-код в приложении Max:\n");

        match QrCode::new(data.as_bytes()) {
            Ok(code) => {
                let image = code
                    .render::<unicode::Dense1x2>()
                    .dark_color(unicode::Dense1x2::Dark)
                    .light_color(unicode::Dense1x2::Light)
                    .build();
                println!("{}", image);
            }
            Err(e) => {
                println!("(Не удалось отрисовать QR-код: {})", e);
                println!("Ссылка вручную: {}", data);
            }
        }
    }

    async fn two_factor_auth_interactive(&self, challenge: &Value) -> MaxResult<String> {
        let track_id = challenge
            .get("trackId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MaxError::Other("Password challenge missing trackId".to_string()))?
            .to_string();

        let hint = challenge
            .get("hint")
            .and_then(|v| v.as_str())
            .unwrap_or("No hint provided")
            .to_string();

        loop {
            print!("Введите пароль (Подсказка: {}): ", hint);
            let _ = IoWrite::flush(&mut io::stdout());

            let password = tokio::task::spawn_blocking(|| {
                let stdin = io::stdin();
                stdin
                    .lock()
                    .lines()
                    .next()
                    .unwrap_or(Ok(String::new()))
                    .unwrap_or_default()
            })
            .await
            .map_err(|e| MaxError::Other(format!("stdin read error: {}", e)))?
            .trim()
            .to_string();

            if password.is_empty() {
                warn!("Password is empty, please try again");
                continue;
            }

            let token_attrs = self.check_password(&password, &track_id).await?;
            if let Some(attrs) = token_attrs {
                if let Some(token) = attrs
                    .get("LOGIN")
                    .and_then(|l| l.get("token"))
                    .and_then(|v| v.as_str())
                {
                    return Ok(token.to_string());
                }
            }
        }
    }

    pub async fn two_factor_auth_with_password(
        &self,
        challenge: &Value,
        password: &str,
    ) -> MaxResult<String> {
        let track_id = challenge
            .get("trackId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MaxError::Other("Password challenge missing trackId".to_string()))?;

        let token_attrs = self
            .check_password(password, track_id)
            .await?
            .ok_or_else(|| MaxError::Other("Incorrect password".to_string()))?;

        token_attrs
            .get("LOGIN")
            .and_then(|l| l.get("token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| MaxError::Other("No LOGIN token in two-factor response".to_string()))
    }

    async fn check_password(&self, password: &str, track_id: &str) -> MaxResult<Option<Value>> {
        let payload = serde_json::to_value(CheckPasswordChallengePayload {
            track_id: track_id.to_string(),
            password: password.to_string(),
        })?;

        let data = self
            .send_default(Opcode::AuthLoginCheckPassword, payload)
            .await?;

        let p = data.get("payload").cloned().unwrap_or_default();
        if p.get("error").is_some() {
            return Ok(None);
        }

        Ok(p.get("tokenAttrs").cloned())
    }

    pub async fn set_password(
        &self,
        password: &str,
        hint: Option<&str>,
        email: &str,
    ) -> MaxResult<bool> {
        info!("Setting account password");

        let track_payload = serde_json::to_value(CreateTrackPayload { type_: 0 })?;
        let data = self
            .send_default(Opcode::AuthCreateTrack, track_payload)
            .await?;
        Self::handle_error(&data)?;

        let track_id = data
            .get("payload")
            .and_then(|p| p.get("trackId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MaxError::ResponseStructure("No trackId in create track response".to_string())
            })?
            .to_string();

        let pw_payload = serde_json::to_value(SetPasswordPayload {
            track_id: track_id.clone(),
            password: password.to_string(),
        })?;
        let pw_data = self
            .send_default(Opcode::AuthValidatePassword, pw_payload)
            .await?;
        let pw_ok = pw_data
            .get("payload")
            .map(|p| p.is_null() || p == &Value::Null)
            .unwrap_or(true);
        if !pw_ok {
            return Err(MaxError::Other("Failed to set password".to_string()));
        }

        if let Some(h) = hint {
            if !h.is_empty() {
                let hint_payload = serde_json::to_value(SetHintPayload {
                    track_id: track_id.clone(),
                    hint: h.to_string(),
                })?;
                self.send_default(Opcode::AuthValidateHint, hint_payload)
                    .await?;
            }
        }

        let email_payload = serde_json::json!({
            "trackId": track_id,
            "email": email
        });
        self.send_default(Opcode::AuthVerifyEmail, email_payload)
            .await?;

        use crate::enums::Capability;
        let two_fa_payload = serde_json::to_value(SetTwoFactorPayload {
            expected_capabilities: vec![
                i32::from(Capability::Default),
                i32::from(Capability::SecondFactorHasHint),
                i32::from(Capability::SecondFactorHasEmail),
            ],
            track_id: track_id.clone(),
            password: password.to_string(),
            hint: hint.map(|s| s.to_string()),
        })?;

        let final_data = self
            .send_default(Opcode::AuthSet2fa, two_fa_payload)
            .await?;
        Self::handle_error(&final_data)?;

        Ok(true)
    }
}
