//! Source RCON client wrapper for Conan Exiles | Server Manager Enhanced.
//!
//! Wraps the upstream [`rcon`] crate (Source RCON over TCP) with the bits a
//! desktop control panel needs:
//!
//! - per-server [`ConnectionHandle`]s, owned by a process-wide [`ConnectionRegistry`]
//! - cooperative reconnect with exponential backoff (1s → 2s → 4s → 8s → 30s cap)
//! - per-call timeouts on `send`
//! - cancellation via [`tokio_util::sync::CancellationToken`] so closing a
//!   server in the UI tears the reconnect loop down promptly
//! - secrets boxed in [`secrecy::SecretString`] so passwords are zeroed on drop
//!
//! Conan uses Source RCON despite the BattlEye anti-cheat — the two are
//! unrelated. Plain Source RCON: no Minecraft 1413-byte command cap, no
//! Factorio single-packet quirk. Both upstream quirk flags are forced off
//! when we open a connection.

mod config;
mod error;
mod handle;
pub mod parsers;
mod registry;
mod state;
mod test_connection;

pub use config::{RconConfig, ServerId};
pub use error::RconError;
pub use handle::ConnectionHandle;
pub use registry::ConnectionRegistry;
pub use state::ConnectionState;
pub use test_connection::test_connection;
