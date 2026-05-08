//! Queries against the `servers` table.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::error::AppResult;

/// Row in the `servers` table — what the DB stores.
///
/// Mirrors the schema 1:1. Passwords are NOT stored here; they live in
/// the OS keychain keyed by `id` (see `secrets.rs`). `has_admin_pw` is the
/// only flag the DB keeps about credentials, so the UI can decide whether
/// to surface "Promote to admin" affordances.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub rcon_port: u16,
    pub has_admin_pw: bool,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

impl Server {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            host: row.get(2)?,
            rcon_port: row.get::<_, i64>(3)? as u16,
            has_admin_pw: row.get::<_, i64>(4)? != 0,
            created_at: row.get(5)?,
            last_used_at: row.get(6)?,
        })
    }
}

/// Input shape from the UI — everything needed to create or update a
/// server. The password fields are taken by value here and forwarded to
/// the keychain by the command layer; they never reach the DB row.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
pub struct ServerInput {
    /// `None` for creates, `Some(id)` for updates.
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub rcon_port: u16,
    /// Always required on save — the keychain entry is rewritten each time.
    pub rcon_password: String,
    /// `None` clears any existing admin password.
    pub admin_password: Option<String>,
}

pub fn list(conn: &Connection) -> AppResult<Vec<Server>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, host, rcon_port, has_admin_pw, created_at, last_used_at \
         FROM servers ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], Server::from_row)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Server>> {
    let result = conn
        .query_row(
            "SELECT id, name, host, rcon_port, has_admin_pw, created_at, last_used_at \
             FROM servers WHERE id = ?1",
            params![id],
            Server::from_row,
        )
        .optional()?;
    Ok(result)
}

/// Insert a brand-new server row. The `id` and `created_at` are filled in.
/// Returns the persisted [`Server`].
pub fn insert(
    conn: &Connection,
    name: &str,
    host: &str,
    rcon_port: u16,
    has_admin_pw: bool,
    now: i64,
) -> AppResult<Server> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO servers (id, name, host, rcon_port, has_admin_pw, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, name, host, rcon_port as i64, has_admin_pw as i64, now],
    )?;
    Ok(Server {
        id,
        name: name.into(),
        host: host.into(),
        rcon_port,
        has_admin_pw,
        created_at: now,
        last_used_at: None,
    })
}

/// Update an existing server. Returns the updated row, or `None` if `id`
/// doesn't exist.
pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    host: &str,
    rcon_port: u16,
    has_admin_pw: bool,
) -> AppResult<Option<Server>> {
    let rows = conn.execute(
        "UPDATE servers SET name = ?1, host = ?2, rcon_port = ?3, has_admin_pw = ?4 \
         WHERE id = ?5",
        params![name, host, rcon_port as i64, has_admin_pw as i64, id],
    )?;
    if rows == 0 {
        return Ok(None);
    }
    get(conn, id)
}

/// Bump `last_used_at` to `now` for `id`. Best-effort — silent no-op if
/// the row doesn't exist.
pub fn touch(conn: &Connection, id: &str, now: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE servers SET last_used_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Delete a server. Returns whether a row was removed.
pub fn delete(conn: &Connection, id: &str) -> AppResult<bool> {
    let rows = conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::Db;

    fn now() -> i64 {
        1_700_000_000
    }

    #[test]
    fn empty_list_on_fresh_db() {
        let db = Db::in_memory().unwrap();
        let servers = db.with(|c| list(c)).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn insert_get_update_delete_roundtrip() {
        let db = Db::in_memory().unwrap();

        let inserted = db
            .with(|c| insert(c, "Test", "1.2.3.4", 7779, false, now()))
            .unwrap();
        assert_eq!(inserted.name, "Test");
        assert_eq!(inserted.host, "1.2.3.4");

        let fetched = db.with(|c| get(c, &inserted.id)).unwrap().unwrap();
        assert_eq!(fetched, inserted);

        let updated = db
            .with(|c| update(c, &inserted.id, "Renamed", "5.6.7.8", 25575, true))
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.host, "5.6.7.8");
        assert_eq!(updated.rcon_port, 25575);
        assert!(updated.has_admin_pw);

        let removed = db.with(|c| delete(c, &inserted.id)).unwrap();
        assert!(removed);

        assert!(db.with(|c| get(c, &inserted.id)).unwrap().is_none());
    }

    #[test]
    fn list_orders_by_name_case_insensitive() {
        let db = Db::in_memory().unwrap();
        db.with(|c| {
            insert(c, "zebra", "h", 1, false, now())?;
            insert(c, "Apple", "h", 1, false, now())?;
            insert(c, "banana", "h", 1, false, now())?;
            Ok(())
        })
        .unwrap();
        let names: Vec<_> = db
            .with(|c| list(c))
            .unwrap()
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, vec!["Apple", "banana", "zebra"]);
    }

    #[test]
    fn delete_unknown_returns_false() {
        let db = Db::in_memory().unwrap();
        let removed = db.with(|c| delete(c, "no-such-id")).unwrap();
        assert!(!removed);
    }

    #[test]
    fn update_unknown_returns_none() {
        let db = Db::in_memory().unwrap();
        let result = db
            .with(|c| update(c, "no-such-id", "n", "h", 1, false))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn touch_updates_last_used_at() {
        let db = Db::in_memory().unwrap();
        let s = db.with(|c| insert(c, "t", "h", 1, false, now())).unwrap();
        assert!(s.last_used_at.is_none());

        db.with(|c| touch(c, &s.id, now() + 100)).unwrap();
        let after = db.with(|c| get(c, &s.id)).unwrap().unwrap();
        assert_eq!(after.last_used_at, Some(now() + 100));
    }
}
