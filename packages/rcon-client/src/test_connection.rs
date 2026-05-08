use secrecy::ExposeSecret;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::{RconConfig, RconError};

/// One-shot connection test: open + auth + drop.
///
/// Used by the UI's "Test connection" button. Returns `Ok(())` if the
/// configured host accepts the password, otherwise the typed reason.
/// Honours [`RconConfig::connect_timeout`].
pub async fn test_connection(config: &RconConfig) -> Result<(), RconError> {
    let connect_fut = rcon::Connection::<TcpStream>::builder()
        .enable_minecraft_quirks(false)
        .enable_factorio_quirks(false)
        .connect(
            (config.host.as_str(), config.port),
            config.password.expose_secret(),
        );

    match timeout(config.connect_timeout, connect_fut).await {
        Ok(Ok(_conn)) => Ok(()),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err(RconError::Timeout),
    }
}
