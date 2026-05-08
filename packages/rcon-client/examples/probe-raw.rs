//! Raw probe that bypasses our wrapper to test against the upstream `rcon`
//! crate directly with `factorio_quirks` (single-packet response mode).
//! Conan's RCON appears to ignore the empty-marker packet our default
//! multi-packet handling sends, hanging command responses.

use std::env;

use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("RCON_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("RCON_PORT")
        .unwrap_or_else(|_| "25575".to_string())
        .parse()?;
    let password = env::var("RCON_PASSWORD")?;

    let cmd = env::args().nth(1).ok_or("usage: probe-raw <command>")?;

    let mut conn = rcon::Connection::<TcpStream>::builder()
        .enable_minecraft_quirks(false)
        .enable_factorio_quirks(true) // single-packet mode
        .connect((host.as_str(), port), password.as_str())
        .await?;

    let response = conn.cmd(&cmd).await?;
    print!("{}", response);

    Ok(())
}
