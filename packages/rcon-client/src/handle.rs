use std::sync::Arc;
use std::time::Duration;

use secrecy::ExposeSecret;
use tokio::net::TcpStream;
use tokio::sync::{watch, Mutex as AsyncMutex, Notify};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::{ConnectionState, RconConfig, RconError};

const BACKOFF_INITIAL: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

/// A live (or trying-to-be-live) connection to a single server.
///
/// Created via [`ConnectionHandle::open`]. The handle owns a background
/// "maintain" task that keeps a `rcon::Connection` warm with reconnect
/// backoff; calling [`ConnectionHandle::close`] cancels the task and
/// releases the socket. Dropping the handle without `close` leaks the
/// reconnect task — always go through `close`.
pub struct ConnectionHandle {
    state_tx: Arc<watch::Sender<ConnectionState>>,
    state_rx: watch::Receiver<ConnectionState>,
    connection: Arc<AsyncMutex<Option<rcon::Connection<TcpStream>>>>,
    reconnect_signal: Arc<Notify>,
    cancel: CancellationToken,
    config: Arc<RconConfig>,
    maintain_task: Option<JoinHandle<()>>,
}

impl ConnectionHandle {
    /// Open a connection and start the maintain task.
    ///
    /// Returns immediately with the handle in [`ConnectionState::Connecting`].
    /// The maintain task transitions to `Open` once the initial handshake
    /// succeeds, or to `Failed` if authentication is rejected.
    ///
    /// To know whether the connection has come up, watch [`Self::subscribe`]
    /// or call [`Self::wait_until_settled`].
    pub fn open(config: RconConfig) -> Self {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Connecting);
        let state_tx = Arc::new(state_tx);
        let connection = Arc::new(AsyncMutex::new(None));
        let reconnect_signal = Arc::new(Notify::new());
        let cancel = CancellationToken::new();
        let config = Arc::new(config);

        let task = tokio::spawn(maintain(
            state_tx.clone(),
            connection.clone(),
            reconnect_signal.clone(),
            cancel.clone(),
            config.clone(),
        ));

        Self {
            state_tx,
            state_rx,
            connection,
            reconnect_signal,
            cancel,
            config,
            maintain_task: Some(task),
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

    /// Block until the maintain task settles into [`ConnectionState::Open`]
    /// or [`ConnectionState::Failed`]. Useful in tests and for "Test
    /// connection" flows that need a synchronous outcome.
    pub async fn wait_until_settled(&self) -> ConnectionState {
        let mut rx = self.state_rx.clone();
        loop {
            let s = *rx.borrow_and_update();
            if matches!(s, ConnectionState::Open | ConnectionState::Failed) {
                return s;
            }
            if rx.changed().await.is_err() {
                // Sender dropped (handle closed mid-wait).
                return *rx.borrow();
            }
        }
    }

    /// Send an RCON command and return the assembled response.
    ///
    /// Honours [`RconConfig::command_timeout`]. On transport error, clears
    /// the live connection and signals the maintain task to reconnect, so
    /// the next call after the backoff will land on a fresh socket.
    pub async fn send(&self, cmd: &str) -> Result<String, RconError> {
        if *self.state_rx.borrow() != ConnectionState::Open {
            return Err(RconError::NotConnected);
        }

        let mut guard = self.connection.lock().await;
        let Some(conn) = guard.as_mut() else {
            // Race: state was Open but maintain task cleared the slot.
            return Err(RconError::NotConnected);
        };

        match timeout(self.config.command_timeout, conn.cmd(cmd)).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(rcon_err)) => {
                warn!(error = %rcon_err, "rcon send failed; triggering reconnect");
                *guard = None;
                drop(guard);
                self.state_tx.send_replace(ConnectionState::Reconnecting);
                self.reconnect_signal.notify_one();
                Err(rcon_err.into())
            }
            Err(_elapsed) => {
                warn!("rcon send timed out; triggering reconnect");
                *guard = None;
                drop(guard);
                self.state_tx.send_replace(ConnectionState::Reconnecting);
                self.reconnect_signal.notify_one();
                Err(RconError::Timeout)
            }
        }
    }

    /// Cancel the maintain task and release the socket.
    ///
    /// Safe to call multiple times; subsequent calls are no-ops.
    pub async fn close(mut self) {
        self.cancel.cancel();
        if let Some(task) = self.maintain_task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        // Best-effort cleanup if the caller didn't await close().
        // Cancels the loop; the spawned task will exit on its next select.
        self.cancel.cancel();
    }
}

async fn maintain(
    state_tx: Arc<watch::Sender<ConnectionState>>,
    connection: Arc<AsyncMutex<Option<rcon::Connection<TcpStream>>>>,
    reconnect_signal: Arc<Notify>,
    cancel: CancellationToken,
    config: Arc<RconConfig>,
) {
    let mut backoff = BACKOFF_INITIAL;

    'outer: loop {
        if cancel.is_cancelled() {
            break;
        }

        debug!(host = %config.host, port = config.port, "connecting");

        let connect_result = tokio::select! {
            _ = cancel.cancelled() => break 'outer,
            r = timeout(config.connect_timeout, open_connection(&config)) => r,
        };

        match connect_result {
            Ok(Ok(conn)) => {
                info!(host = %config.host, port = config.port, "rcon connected");
                backoff = BACKOFF_INITIAL;
                *connection.lock().await = Some(conn);
                state_tx.send_replace(ConnectionState::Open);

                // Park until something asks us to reconnect or cancels us.
                tokio::select! {
                    _ = cancel.cancelled() => break 'outer,
                    _ = reconnect_signal.notified() => {
                        debug!("reconnect signal received");
                    }
                }
            }
            Ok(Err(RconError::AuthFailed)) => {
                warn!(host = %config.host, "authentication failed; not retrying");
                // Auth failure is terminal. Clear the connection slot and
                // leave state as Failed — bypass the cleanup at the bottom
                // so we don't overwrite Failed with Disconnected.
                *connection.lock().await = None;
                state_tx.send_replace(ConnectionState::Failed);
                debug!("maintain task exited (auth failed)");
                return;
            }
            Ok(Err(err)) => {
                warn!(error = %err, backoff_ms = backoff.as_millis() as u64, "connect failed; backing off");
                state_tx.send_replace(ConnectionState::Reconnecting);
                if !sleep_or_cancel(backoff, &cancel).await {
                    break 'outer;
                }
                backoff = (backoff * 2).min(BACKOFF_MAX);
            }
            Err(_elapsed) => {
                warn!(
                    backoff_ms = backoff.as_millis() as u64,
                    "connect timed out; backing off"
                );
                state_tx.send_replace(ConnectionState::Reconnecting);
                if !sleep_or_cancel(backoff, &cancel).await {
                    break 'outer;
                }
                backoff = (backoff * 2).min(BACKOFF_MAX);
            }
        }
    }

    *connection.lock().await = None;
    state_tx.send_replace(ConnectionState::Disconnected);
    debug!("maintain task exited");
}

async fn open_connection(config: &RconConfig) -> Result<rcon::Connection<TcpStream>, RconError> {
    rcon::Connection::<TcpStream>::builder()
        .enable_minecraft_quirks(false)
        .enable_factorio_quirks(false)
        .connect(
            (config.host.as_str(), config.port),
            config.password.expose_secret(),
        )
        .await
        .map_err(Into::into)
}

/// Sleep for `dur`, but return early (false) if `cancel` fires.
async fn sleep_or_cancel(dur: Duration, cancel: &CancellationToken) -> bool {
    tokio::select! {
        _ = cancel.cancelled() => false,
        _ = sleep(dur) => true,
    }
}
