use std::sync::Arc;

use secrecy::ExposeSecret;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::error::is_rate_limit_response;
use crate::{ConnectionState, RconConfig, RconError};

/// A logical RCON connection to a single server.
///
/// Conan's RCON server **closes the TCP connection after every command**
/// and rate-limits new connections via `RconMaxKarma`. So unlike a typical
/// Source RCON service we cannot keep one socket warm — every `send` opens
/// a fresh connection, authenticates, sends one command, reads the
/// single-packet response, and drops the socket.
///
/// `ConnectionHandle` therefore tracks credential-validity rather than
/// socket-liveness:
///
/// - `Connecting` — initial auth probe in flight after `open()`.
/// - `Open` — credentials are good. Sends will likely succeed.
/// - `Reconnecting` — last operation hit a transport failure; subsequent
///   sends may recover transparently (each is a fresh connect).
/// - `Failed` — terminal: server rejected the password. Caller must
///   close + re-open with corrected creds.
/// - `Disconnected` — `close()` was called.
pub struct ConnectionHandle {
    state_tx: Arc<watch::Sender<ConnectionState>>,
    state_rx: watch::Receiver<ConnectionState>,
    cancel: CancellationToken,
    config: Arc<RconConfig>,
    probe_task: Option<JoinHandle<()>>,
}

impl ConnectionHandle {
    /// Starts a one-shot auth probe in the background. The handle becomes
    /// usable for `send` once it transitions to [`ConnectionState::Open`]
    /// — see [`Self::wait_until_settled`] for a synchronous outcome.
    pub fn open(config: RconConfig) -> Self {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Connecting);
        let state_tx = Arc::new(state_tx);
        let cancel = CancellationToken::new();
        let config = Arc::new(config);

        let probe_task = tokio::spawn(probe(state_tx.clone(), cancel.clone(), config.clone()));

        Self {
            state_tx,
            state_rx,
            cancel,
            config,
            probe_task: Some(probe_task),
        }
    }

    /// Current connection state. Cheap, non-blocking.
    pub fn state(&self) -> ConnectionState {
        *self.state_rx.borrow()
    }

    /// Subscribe to state changes. UI bindings poll the receiver or call
    /// `changed().await` to react.
    pub fn subscribe(&self) -> watch::Receiver<ConnectionState> {
        self.state_rx.clone()
    }

    /// Block until the auth probe settles into [`ConnectionState::Open`]
    /// or [`ConnectionState::Failed`].
    pub async fn wait_until_settled(&self) -> ConnectionState {
        let mut rx = self.state_rx.clone();
        loop {
            let s = *rx.borrow_and_update();
            if matches!(s, ConnectionState::Open | ConnectionState::Failed) {
                return s;
            }
            if rx.changed().await.is_err() {
                return *rx.borrow();
            }
        }
    }

    /// Send an RCON command and return the assembled response.
    ///
    /// Each call opens a fresh TCP connection, authenticates, sends, reads
    /// the single-packet response, and drops the socket. Honours
    /// [`RconConfig::command_timeout`].
    pub async fn send(&self, cmd: &str) -> Result<String, RconError> {
        let state = *self.state_rx.borrow();
        if state == ConnectionState::Failed {
            return Err(RconError::AuthFailed);
        }
        if state == ConnectionState::Disconnected {
            return Err(RconError::NotConnected);
        }

        match send_one(&self.config, cmd).await {
            Ok(response) => {
                if state != ConnectionState::Open {
                    self.state_tx.send_replace(ConnectionState::Open);
                }
                if is_rate_limit_response(&response) {
                    warn!("rcon rate-limited; raise RconMaxKarma in Game.ini");
                    return Err(RconError::RateLimited);
                }
                Ok(response)
            }
            Err(RconError::AuthFailed) => {
                warn!("rcon auth rejected mid-session — credentials changed?");
                self.state_tx.send_replace(ConnectionState::Failed);
                Err(RconError::AuthFailed)
            }
            Err(err) => {
                // Transient transport hiccup: park in Reconnecting so the
                // UI can show the indicator. Next send retries on a fresh
                // socket, which usually recovers automatically.
                self.state_tx.send_replace(ConnectionState::Reconnecting);
                Err(err)
            }
        }
    }

    /// Cancel the probe task and mark the handle disconnected.
    pub async fn close(mut self) {
        self.cancel.cancel();
        if let Some(task) = self.probe_task.take() {
            let _ = task.await;
        }
        self.state_tx.send_replace(ConnectionState::Disconnected);
    }
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// One-shot auth probe — connect, authenticate, drop. Used to flip the
/// state from `Connecting` to `Open` (or `Failed`) once after `open()`.
async fn probe(
    state_tx: Arc<watch::Sender<ConnectionState>>,
    cancel: CancellationToken,
    config: Arc<RconConfig>,
) {
    let probe_fut = open_socket(&config);
    tokio::select! {
        _ = cancel.cancelled() => {
            debug!("probe cancelled");
        }
        result = timeout(config.connect_timeout, probe_fut) => {
            match result {
                Ok(Ok(_conn)) => {
                    info!(host = %config.host, port = config.port, "rcon credentials verified");
                    state_tx.send_replace(ConnectionState::Open);
                }
                Ok(Err(RconError::AuthFailed)) => {
                    warn!(host = %config.host, "rcon authentication failed");
                    state_tx.send_replace(ConnectionState::Failed);
                }
                Ok(Err(err)) => {
                    warn!(error = %err, "rcon probe transport error");
                    state_tx.send_replace(ConnectionState::Reconnecting);
                }
                Err(_) => {
                    warn!("rcon probe timed out");
                    state_tx.send_replace(ConnectionState::Reconnecting);
                }
            }
        }
    }
}

/// Run a single command end-to-end: connect, auth, send, read, drop.
async fn send_one(config: &RconConfig, cmd: &str) -> Result<String, RconError> {
    let send_fut = async {
        let mut conn = open_socket(config).await?;
        let response = conn.cmd(cmd).await?;
        Ok::<_, RconError>(response)
    };

    timeout(config.command_timeout, send_fut)
        .await
        .map_err(|_| RconError::Timeout)?
}

/// Open and authenticate a fresh `rcon::Connection`. Conan-specific quirks
/// applied: Minecraft mode off (no command-length cap, no inter-packet
/// sleep), Factorio mode on (single-packet responses — Conan ignores the
/// empty-marker packet that multi-packet mode relies on).
async fn open_socket(config: &RconConfig) -> Result<rcon::Connection<TcpStream>, RconError> {
    rcon::Connection::<TcpStream>::builder()
        .enable_minecraft_quirks(false)
        .enable_factorio_quirks(true)
        .connect(
            (config.host.as_str(), config.port),
            config.password.expose_secret(),
        )
        .await
        .map_err(Into::into)
}
