//! OS keychain integration for RCON credentials.
//!
//! Two entries per server:
//! - `cesm:{server_id}:rcon`  — the RCON password (always present once a
//!   server is saved).
//! - `cesm:{server_id}:admin` — the optional in-game admin password used
//!   for the `/MakeAdmin` prompt flow.
//!
//! Backed by `keyring` v3, which uses macOS Keychain on darwin and Windows
//! Credential Manager on win32. Linux Secret Service support arrives with
//! the v0.1 Linux build.

use keyring::Entry;
use secrecy::{ExposeSecret, SecretString};

use crate::error::AppResult;

/// Service identifier used for every keychain entry. Namespaced so we can
/// be deleted cleanly via OS tooling if a user uninstalls.
const SERVICE: &str = "dev.brandonbridges.cesm";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Rcon,
    Admin,
}

impl Slot {
    fn suffix(self) -> &'static str {
        match self {
            Slot::Rcon => "rcon",
            Slot::Admin => "admin",
        }
    }
}

fn account_for(server_id: &str, slot: Slot) -> String {
    format!("cesm:{server_id}:{}", slot.suffix())
}

fn entry(server_id: &str, slot: Slot) -> AppResult<Entry> {
    Ok(Entry::new(SERVICE, &account_for(server_id, slot))?)
}

pub fn store(server_id: &str, slot: Slot, value: &SecretString) -> AppResult<()> {
    entry(server_id, slot)?.set_password(value.expose_secret())?;
    Ok(())
}

pub fn load(server_id: &str, slot: Slot) -> AppResult<Option<SecretString>> {
    match entry(server_id, slot)?.get_password() {
        Ok(s) => Ok(Some(SecretString::from(s))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete(server_id: &str, slot: Slot) -> AppResult<()> {
    match entry(server_id, slot)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Delete every keychain entry for `server_id`. Called on server delete.
pub fn delete_all(server_id: &str) -> AppResult<()> {
    delete(server_id, Slot::Rcon)?;
    delete(server_id, Slot::Admin)?;
    Ok(())
}
