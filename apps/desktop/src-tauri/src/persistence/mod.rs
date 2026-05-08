//! SQLite-backed persistence layer.
//!
//! - `connection` — opens the DB at `$APP_DATA/cesm/db.sqlite`, runs
//!   migrations, exposes a thread-safe wrapper.
//! - `servers` — CRUD against the `servers` table.
//! - `settings` — key/value access for the `app_settings` table.
//!
//! No business logic lives here. `commands` is the only caller; the DB
//! types map 1:1 to rows.

mod connection;
mod migrations;
pub mod servers;
pub mod settings;

pub use connection::Db;
