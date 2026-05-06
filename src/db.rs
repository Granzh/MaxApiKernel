// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::errors::{MaxError, MaxResult};

pub struct Database {
    path: String,
}

impl Database {
    pub fn new(workdir: &str) -> MaxResult<Self> {
        let path = format!("{}/session.db", workdir);
        let db = Self { path };
        db.init()?;
        Ok(db)
    }

    fn connect(&self) -> MaxResult<Connection> {
        Connection::open(&self.path).map_err(MaxError::Database)
    }

    fn init(&self) -> MaxResult<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS auth (
                device_id TEXT PRIMARY KEY,
                token TEXT
            );",
        )
        .map_err(MaxError::Database)?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM auth", [], |row| row.get(0))
            .map_err(MaxError::Database)?;

        if count == 0 {
            let device_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO auth (device_id, token) VALUES (?1, NULL)",
                params![device_id],
            )
            .map_err(MaxError::Database)?;
        } else if count > 1 {
            conn.execute(
                "DELETE FROM auth WHERE rowid NOT IN (SELECT MIN(rowid) FROM auth)",
                [],
            )
            .map_err(MaxError::Database)?;
        }

        Ok(())
    }

    pub fn get_auth_token(&self) -> MaxResult<Option<String>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare("SELECT token FROM auth LIMIT 1")
            .map_err(MaxError::Database)?;

        let token: Option<Option<String>> = stmt
            .query_row([], |row| row.get(0))
            .optional()
            .map_err(MaxError::Database)?;

        Ok(token.flatten())
    }

    pub fn get_device_id(&self) -> MaxResult<Uuid> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare("SELECT device_id FROM auth LIMIT 1")
            .map_err(MaxError::Database)?;

        let id_str: Option<String> = stmt
            .query_row([], |row| row.get(0))
            .optional()
            .map_err(MaxError::Database)?;

        match id_str {
            Some(s) => Uuid::parse_str(&s)
                .map_err(|e| MaxError::Other(format!("Invalid UUID in db: {}", e))),
            None => {
                let new_id = Uuid::new_v4();
                conn.execute(
                    "INSERT OR REPLACE INTO auth (device_id, token) VALUES (?1, NULL)",
                    params![new_id.to_string()],
                )
                .map_err(MaxError::Database)?;
                Ok(new_id)
            }
        }
    }

    pub fn update_auth_token(&self, device_id: &Uuid, token: &str) -> MaxResult<()> {
        let conn = self.connect()?;
        let device_id_str = device_id.to_string();

        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM auth WHERE device_id = ?1",
                params![device_id_str],
                |row| row.get::<_, i64>(0),
            )
            .map_err(MaxError::Database)?
            > 0;

        if exists {
            conn.execute(
                "UPDATE auth SET token = ?1 WHERE device_id = ?2",
                params![token, device_id_str],
            )
            .map_err(MaxError::Database)?;
        } else {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM auth", [], |row| row.get(0))
                .map_err(MaxError::Database)?;

            if count > 0 {
                conn.execute(
                    "UPDATE auth SET device_id = ?1, token = ?2",
                    params![device_id_str, token],
                )
                .map_err(MaxError::Database)?;
            } else {
                conn.execute(
                    "INSERT INTO auth (device_id, token) VALUES (?1, ?2)",
                    params![device_id_str, token],
                )
                .map_err(MaxError::Database)?;
            }
        }

        Ok(())
    }
}

trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
