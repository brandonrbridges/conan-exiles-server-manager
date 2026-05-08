use serde::{Deserialize, Serialize};

/// Lifecycle state of a single server's connection.
///
/// Read by the UI's status indicator. The transitions are:
///
/// ```text
///   Disconnected ──open()──▶ Connecting ──auth ok──▶ Open
///                                 │                    │
///                                 │                    │ transport error
///                                 ▼                    ▼
///                             Failed ◀───────── Reconnecting
///                                                     │
///                                                     │ auth ok
///                                                     ▼
///                                                    Open
/// ```
///
/// `Failed` is only entered when the configured retry budget is exhausted
/// (currently never — backoff is unbounded by design until `close()` is
/// called) or when authentication fails. It's reserved for the case where
/// the registry has decided the user must intervene.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Open,
    Reconnecting,
    Failed,
}
