use std::time::Duration;

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Newtype wrapping a UUID identifying a saved server.
///
/// Stored as TEXT in SQLite. Use a newtype rather than raw `String`/`Uuid`
/// so the compiler stops us passing a player name where a server id was
/// expected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServerId(pub Uuid);

impl ServerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ServerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Configuration for opening a connection to a single server.
///
/// `password` is wrapped in [`SecretString`] so it's zeroed on drop and
/// won't leak through `Debug` or accidental logs. The keychain layer
/// (PR #3) is the only place that touches the underlying string.
#[derive(Clone, Debug, Deserialize)]
pub struct RconConfig {
    pub host: String,
    pub port: u16,
    pub password: SecretString,
    /// Timeout for the initial TCP connect + auth handshake.
    /// Default: 10s.
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: Duration,
    /// Timeout per `send` call.
    /// Default: 5s.
    #[serde(default = "default_command_timeout")]
    pub command_timeout: Duration,
}

impl RconConfig {
    pub fn new(host: impl Into<String>, port: u16, password: SecretString) -> Self {
        Self {
            host: host.into(),
            port,
            password,
            connect_timeout: default_connect_timeout(),
            command_timeout: default_command_timeout(),
        }
    }
}

const fn default_connect_timeout() -> Duration {
    Duration::from_secs(10)
}

const fn default_command_timeout() -> Duration {
    Duration::from_secs(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_id_displays_as_uuid() {
        let id = ServerId::new();
        let s = id.to_string();
        assert_eq!(s.len(), 36);
        assert_eq!(s.matches('-').count(), 4);
    }

    #[test]
    fn server_ids_are_unique() {
        let a = ServerId::new();
        let b = ServerId::new();
        assert_ne!(a, b);
    }
}
