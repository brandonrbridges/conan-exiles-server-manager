//! Embedded migrations applied on every app startup.
//!
//! Ordering is by filename — `0001_*` before `0002_*` etc. Adding a new
//! migration is a one-line append in the `migrations()` constructor; the
//! crate handles tracking applied versions in a `__migrations` table.

use rusqlite_migration::{Migrations, M};
use std::sync::OnceLock;

static MIGRATIONS: OnceLock<Migrations<'static>> = OnceLock::new();

pub fn all() -> &'static Migrations<'static> {
    MIGRATIONS
        .get_or_init(|| Migrations::new(vec![M::up(include_str!("migrations/0001_init.sql"))]))
}
