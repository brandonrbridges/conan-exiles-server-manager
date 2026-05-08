//! Process-wide SQLite connection wrapper.
//!
//! Holds a single `rusqlite::Connection` behind a `Mutex`. Desktop apps
//! see negligible concurrency on the DB so a connection pool would just be
//! ceremony — the mutex serialises the single-user-with-multiple-windows
//! case fine.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};
use crate::persistence::migrations;

pub struct Db {
    inner: Mutex<Connection>,
}

impl Db {
    /// Open the SQLite database at `path`, creating it (and parent dirs)
    /// if missing, and run any pending migrations.
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut conn = Connection::open(path)?;

        // Foreign keys aren't on by default in rusqlite — required for the
        // `ban_cache(server_id) -> servers(id) ON DELETE CASCADE` to actually
        // cascade.
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        migrations::all().to_latest(&mut conn)?;

        Ok(Self {
            inner: Mutex::new(conn),
        })
    }

    /// Open an in-memory database — for tests only.
    #[cfg(test)]
    pub fn in_memory() -> AppResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrations::all().to_latest(&mut conn)?;
        Ok(Self {
            inner: Mutex::new(conn),
        })
    }

    /// Run a closure with the underlying connection. The closure gets `&mut
    /// Connection` for transaction support; it must not hold the lock across
    /// `.await` points.
    pub fn with<R>(&self, f: impl FnOnce(&mut Connection) -> AppResult<R>) -> AppResult<R> {
        let mut guard = self.inner.lock().map_err(|_| AppError::Internal {
            message: "db mutex poisoned".into(),
        })?;
        f(&mut guard)
    }
}
