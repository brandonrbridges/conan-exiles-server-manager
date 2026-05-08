use thiserror::Error;

/// Errors surfaced by the RCON wrapper.
///
/// `Display` implementations are user-facing — they end up in toast messages
/// and the connection-state UI, so they should be terse and free of jargon.
#[derive(Debug, Error)]
pub enum RconError {
    /// Authentication was rejected by the server.
    ///
    /// Source RCON signals this with a packet whose `id` is `-1`. Always
    /// terminal — never retried automatically, since the password is wrong.
    #[error("authentication failed")]
    AuthFailed,

    /// Underlying TCP transport failed (connect refused, reset, etc.).
    #[error("transport error: {0}")]
    Transport(#[source] std::io::Error),

    /// `send` exceeded its per-call timeout.
    #[error("command timed out")]
    Timeout,

    /// Caller invoked `send` while the connection wasn't `Open`.
    #[error("not connected")]
    NotConnected,

    /// Registry lookup miss — `send`/`close` against an unknown `ServerId`.
    #[error("unknown server")]
    UnknownServer,

    /// Conan's RCON karma rate limiter rejected the connection. Bumping
    /// `RconMaxKarma` in `Game.ini` raises the ceiling; the default of 60
    /// is conservative for production but cramped for development.
    #[error("rate limited")]
    RateLimited,
}

impl From<rcon::Error> for RconError {
    fn from(err: rcon::Error) -> Self {
        match err {
            rcon::Error::Auth => Self::AuthFailed,
            rcon::Error::Io(e) => Self::Transport(e),
            // `CommandTooLong` is impossible because we disable Minecraft
            // quirks; if upstream changes their mind, surface as transport.
            rcon::Error::CommandTooLong => Self::Transport(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "command exceeds upstream length limit",
            )),
        }
    }
}

/// Conan signals rate-limit denial as the *body* of a command response,
/// not as a transport error. Detect it on the way out of `send`.
pub(crate) fn is_rate_limit_response(body: &str) -> bool {
    body.trim() == "Too many commands, try again later."
}
