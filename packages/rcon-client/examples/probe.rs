//! Tiny CLI for sending arbitrary RCON commands to a Conan Exiles Enhanced
//! server. Used during development to capture real command output that the
//! parsers in this crate are written against.
//!
//! Usage:
//! ```text
//! RCON_HOST=127.0.0.1 RCON_PORT=25575 RCON_PASSWORD=secret \
//!   cargo run --example probe -- "serverinfo"
//! ```

use std::env;

use rcon_client::{ConnectionHandle, ConnectionState, RconConfig};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("RCON_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("RCON_PORT")
        .unwrap_or_else(|_| "25575".to_string())
        .parse()?;
    let password = env::var("RCON_PASSWORD")?;

    let cmd = env::args().nth(1).ok_or("usage: probe <command>")?;

    let mut cfg = RconConfig::new(host, port, SecretString::from(password));
    cfg.command_timeout = std::time::Duration::from_secs(20);
    cfg.connect_timeout = std::time::Duration::from_secs(15);
    let handle = ConnectionHandle::open(cfg);

    match handle.wait_until_settled().await {
        ConnectionState::Open => {}
        other => {
            eprintln!("connection settled to {:?}", other);
            return Err("could not establish connection".into());
        }
    }

    let response = handle.send(&cmd).await?;
    print!("{}", response);

    handle.close().await;
    Ok(())
}
