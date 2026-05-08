//! Tiny in-process Source RCON server, used by the rcon-client integration
//! tests. Implements just enough of the protocol to drive auth + the
//! multi-packet response idiom the upstream `rcon` crate uses.
//!
//! Wire format reference: <https://developer.valvesoftware.com/wiki/Source_RCON_Protocol>

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[allow(dead_code)]
const TYPE_AUTH: i32 = 3;
#[allow(dead_code)]
const TYPE_AUTH_RESPONSE: i32 = 2;
const TYPE_EXEC_COMMAND: i32 = 2;
const TYPE_RESPONSE_VALUE: i32 = 0;

pub struct MockRconServer {
    pub addr: std::net::SocketAddr,
    pub handle: JoinHandle<()>,
    pub stop: oneshot::Sender<()>,
}

#[derive(Clone)]
pub struct MockBehaviour {
    /// Password required for successful auth. Wrong passwords produce id=-1.
    pub password: Arc<String>,
    /// If `Some`, every accepted connection drops without a single response
    /// after this many commands have been received. Used to simulate mid-flight
    /// failure.
    pub drop_after: Option<usize>,
    /// If true, return `pong` for any single command.
    pub echo_pong: bool,
    /// If `Some`, return this exact body for any command.
    pub canned_response: Option<String>,
}

impl Default for MockBehaviour {
    fn default() -> Self {
        Self {
            password: Arc::new("correct-horse".into()),
            drop_after: None,
            echo_pong: true,
            canned_response: None,
        }
    }
}

impl MockRconServer {
    pub async fn start(behaviour: MockBehaviour) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let (stop_tx, mut stop_rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop_rx => return,
                    accept = listener.accept() => {
                        if let Ok((stream, _peer)) = accept {
                            let b = behaviour.clone();
                            tokio::spawn(async move {
                                let _ = handle_client(stream, b).await;
                            });
                        }
                    }
                }
            }
        });

        Ok(Self {
            addr,
            handle,
            stop: stop_tx,
        })
    }

    pub async fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.handle.await;
    }
}

async fn handle_client(mut stream: TcpStream, b: MockBehaviour) -> std::io::Result<()> {
    // Auth
    let (auth_id, auth_type, auth_body) = read_packet(&mut stream).await?;
    if auth_type != TYPE_AUTH {
        return Ok(());
    }
    let password_ok = auth_body == *b.password;
    let response_id = if password_ok { auth_id } else { -1 };

    // Source-protocol quirk: server sends a SERVERDATA_RESPONSE_VALUE first,
    // then the SERVERDATA_AUTH_RESPONSE. The upstream `rcon` crate filters by
    // type so the order is forgiving, but mirroring real servers is cheap.
    write_packet(&mut stream, auth_id, TYPE_RESPONSE_VALUE, "").await?;
    write_packet(&mut stream, response_id, TYPE_AUTH_RESPONSE, "").await?;

    if !password_ok {
        return Ok(());
    }

    // Command loop
    let mut commands_seen = 0usize;
    loop {
        let (cmd_id, cmd_type, cmd_body) = match read_packet(&mut stream).await {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };
        if cmd_type != TYPE_EXEC_COMMAND {
            continue;
        }

        // Only count real commands; the empty marker is the second leg of
        // the multi-packet response idiom and must always get a response.
        // Otherwise we'd break the very command we're "letting through".
        if let Some(drop_at) = b.drop_after {
            if !cmd_body.is_empty() && commands_seen >= drop_at {
                drop(stream);
                return Ok(());
            }
        }

        // The upstream client uses the multi-packet response idiom: real
        // command, then an empty packet whose response signals end-of-output.
        let response_body = if cmd_body.is_empty() {
            // Empty marker — answer with empty body so the client sees end-of-output.
            String::new()
        } else if let Some(canned) = b.canned_response.as_ref() {
            canned.clone()
        } else if b.echo_pong {
            "pong".to_string()
        } else {
            String::new()
        };

        write_packet(&mut stream, cmd_id, TYPE_RESPONSE_VALUE, &response_body).await?;

        if !cmd_body.is_empty() {
            commands_seen += 1;
        }
    }
}

async fn read_packet(stream: &mut TcpStream) -> std::io::Result<(i32, i32, String)> {
    let size = stream.read_i32_le().await?;
    let id = stream.read_i32_le().await?;
    let ptype = stream.read_i32_le().await?;
    let remaining = (size as usize).saturating_sub(8);
    let mut buf = vec![0u8; remaining];
    stream.read_exact(&mut buf).await?;
    let body = match buf.iter().position(|&b| b == 0) {
        Some(end) => String::from_utf8_lossy(&buf[..end]).to_string(),
        None => String::from_utf8_lossy(&buf).to_string(),
    };
    Ok((id, ptype, body))
}

async fn write_packet(
    stream: &mut TcpStream,
    id: i32,
    ptype: i32,
    body: &str,
) -> std::io::Result<()> {
    let body_bytes = body.as_bytes();
    let size = (10 + body_bytes.len()) as i32;
    stream.write_i32_le(size).await?;
    stream.write_i32_le(id).await?;
    stream.write_i32_le(ptype).await?;
    stream.write_all(body_bytes).await?;
    stream.write_all(&[0u8, 0u8]).await?;
    stream.flush().await?;
    Ok(())
}
