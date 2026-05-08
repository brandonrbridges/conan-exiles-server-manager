use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{ConnectionHandle, ConnectionState, RconConfig, RconError, ServerId};

/// Process-wide registry of live RCON connections, keyed by [`ServerId`].
///
/// At most one [`ConnectionHandle`] per server — Source RCON is single-session,
/// so opening a second connection to the same server while the first is alive
/// is a bug. The registry enforces this by replacing-and-closing.
///
/// Cheap to clone; the inner state is `Arc`-wrapped.
#[derive(Clone, Default)]
pub struct ConnectionRegistry {
    inner: Arc<RwLock<HashMap<ServerId, Arc<ConnectionHandle>>>>,
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open (or replace) the handle for `id`.
    ///
    /// If a handle already exists for `id`, it is closed before the new one
    /// takes its place. Returns immediately — the handle starts in
    /// `Connecting`. Use [`ConnectionHandle::wait_until_settled`] via
    /// [`Self::handle`] if you need a synchronous outcome.
    pub async fn open(&self, id: ServerId, config: RconConfig) {
        let new_handle = Arc::new(ConnectionHandle::open(config));
        let prev = {
            let mut map = self.inner.write().await;
            map.insert(id, new_handle)
        };
        if let Some(old) = prev {
            // We hold the only Arc here once it's removed from the map, but a
            // caller might still have a clone from `handle()`. Best-effort:
            // try to unwrap, fall back to firing cancel via Drop.
            if let Ok(handle) = Arc::try_unwrap(old) {
                handle.close().await;
            }
        }
    }

    /// Get a clone of the handle for `id`, if open.
    pub async fn handle(&self, id: ServerId) -> Option<Arc<ConnectionHandle>> {
        self.inner.read().await.get(&id).cloned()
    }

    /// Send a command to the server identified by `id`.
    pub async fn send(&self, id: ServerId, cmd: &str) -> Result<String, RconError> {
        let handle = self.handle(id).await.ok_or(RconError::UnknownServer)?;
        handle.send(cmd).await
    }

    /// Current state for `id`, or `None` if unknown.
    pub async fn state(&self, id: ServerId) -> Option<ConnectionState> {
        self.handle(id).await.map(|h| h.state())
    }

    /// Close the handle for `id` if it exists.
    pub async fn close(&self, id: ServerId) {
        let removed = self.inner.write().await.remove(&id);
        if let Some(handle) = removed {
            if let Ok(h) = Arc::try_unwrap(handle) {
                h.close().await;
            }
        }
    }

    /// Close every open handle. Used on app shutdown.
    pub async fn close_all(&self) {
        let drained: Vec<_> = self.inner.write().await.drain().collect();
        for (_, handle) in drained {
            if let Ok(h) = Arc::try_unwrap(handle) {
                h.close().await;
            }
        }
    }
}
