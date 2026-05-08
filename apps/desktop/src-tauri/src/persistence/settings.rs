//! Key/value access to the `app_settings` table.
//!
//! Used for theme, polling interval, update channel, etc. — anything app-
//! level that doesn't deserve its own table.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::Db;

    #[test]
    fn missing_key_returns_none() {
        let db = Db::in_memory().unwrap();
        let value = db.with(|c| get(c, "theme")).unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn set_then_get_roundtrip() {
        let db = Db::in_memory().unwrap();
        db.with(|c| set(c, "theme", "dark")).unwrap();
        assert_eq!(
            db.with(|c| get(c, "theme")).unwrap(),
            Some("dark".to_string())
        );
    }

    #[test]
    fn set_overwrites_existing_value() {
        let db = Db::in_memory().unwrap();
        db.with(|c| set(c, "theme", "dark")).unwrap();
        db.with(|c| set(c, "theme", "light")).unwrap();
        assert_eq!(
            db.with(|c| get(c, "theme")).unwrap(),
            Some("light".to_string())
        );
    }
}
