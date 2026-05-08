use std::time::{SystemTime, UNIX_EPOCH};

use rcon_client::{ConnectionRegistry, ConnectionState, RconConfig};
use secrecy::SecretString;
use serde::Deserialize;
use tauri::State;
use ts_rs::TS;

use crate::error::{AppError, AppResult};
use crate::persistence::servers::{self as servers_repo, Server, ServerInput};
use crate::persistence::Db;
use crate::secrets::{self, Slot};

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn parse_server_id(id: &str) -> AppResult<rcon_client::ServerId> {
    uuid::Uuid::parse_str(id)
        .map(rcon_client::ServerId)
        .map_err(|_| AppError::Invalid {
            message: "server id must be a uuid".into(),
        })
}

/// Validate a server input. Returns `AppError::Invalid` for empty fields,
/// out-of-range ports, etc.
fn validate(input: &ServerInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Invalid {
            message: "name must not be empty".into(),
        });
    }
    if input.host.trim().is_empty() {
        return Err(AppError::Invalid {
            message: "host must not be empty".into(),
        });
    }
    if input.rcon_port == 0 {
        return Err(AppError::Invalid {
            message: "rcon port must be 1..=65535".into(),
        });
    }
    if input.rcon_password.is_empty() {
        return Err(AppError::Invalid {
            message: "rcon password must not be empty".into(),
        });
    }
    Ok(())
}

#[tauri::command]
pub fn list_servers(db: State<'_, Db>) -> AppResult<Vec<Server>> {
    db.with(|c| servers_repo::list(c))
}

#[tauri::command]
pub fn save_server(input: ServerInput, db: State<'_, Db>) -> AppResult<Server> {
    validate(&input)?;

    let has_admin_pw = input
        .admin_password
        .as_ref()
        .map(|p| !p.is_empty())
        .unwrap_or(false);

    let server = match input.id.clone() {
        None => db.with(|c| {
            servers_repo::insert(
                c,
                input.name.trim(),
                input.host.trim(),
                input.rcon_port,
                has_admin_pw,
                now_secs(),
            )
        })?,
        Some(id) => {
            let updated = db.with(|c| {
                servers_repo::update(
                    c,
                    &id,
                    input.name.trim(),
                    input.host.trim(),
                    input.rcon_port,
                    has_admin_pw,
                )
            })?;
            updated.ok_or(AppError::ServerNotFound)?
        }
    };

    // Always rewrite the keychain entries on save so users can correct a
    // bad password by re-saving.
    secrets::store(
        &server.id,
        Slot::Rcon,
        &SecretString::from(input.rcon_password),
    )?;
    match input.admin_password {
        Some(p) if !p.is_empty() => {
            secrets::store(&server.id, Slot::Admin, &SecretString::from(p))?;
        }
        _ => secrets::delete(&server.id, Slot::Admin)?,
    }

    Ok(server)
}

#[tauri::command]
pub async fn delete_server(
    id: String,
    db: State<'_, Db>,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<()> {
    // Tear down any live connection first.
    if let Ok(server_id) = parse_server_id(&id) {
        registry.close(server_id).await;
    }
    let removed = db.with(|c| servers_repo::delete(c, &id))?;
    if !removed {
        return Err(AppError::ServerNotFound);
    }
    secrets::delete_all(&id)?;
    Ok(())
}

/// Subset of `ServerInput` for "Test connection" — no DB write, no
/// keychain mutation. Lets the UI verify credentials before save.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct TestConnectionInput {
    pub host: String,
    pub rcon_port: u16,
    pub rcon_password: String,
}

#[tauri::command]
pub async fn test_connection(input: TestConnectionInput) -> AppResult<()> {
    let cfg = RconConfig::new(
        input.host,
        input.rcon_port,
        SecretString::from(input.rcon_password),
    );
    rcon_client::test_connection(&cfg).await?;
    Ok(())
}

#[tauri::command]
pub async fn open_connection(
    server_id: String,
    db: State<'_, Db>,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<()> {
    let id = parse_server_id(&server_id)?;
    let server = db
        .with(|c| servers_repo::get(c, &server_id))?
        .ok_or(AppError::ServerNotFound)?;
    let password = secrets::load(&server_id, Slot::Rcon)?.ok_or(AppError::Keychain {
        message: "rcon password missing from keychain".into(),
    })?;

    let cfg = RconConfig::new(server.host, server.rcon_port, password);
    registry.open(id, cfg).await;
    db.with(|c| servers_repo::touch(c, &server_id, now_secs()))?;
    Ok(())
}

#[tauri::command]
pub async fn close_connection(
    server_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<()> {
    let id = parse_server_id(&server_id)?;
    registry.close(id).await;
    Ok(())
}

#[tauri::command]
pub async fn connection_state(
    server_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<ConnectionState> {
    let id = parse_server_id(&server_id)?;
    Ok(registry
        .state(id)
        .await
        .unwrap_or(ConnectionState::Disconnected))
}

#[tauri::command]
pub async fn send_command(
    server_id: String,
    command: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<String> {
    let id = parse_server_id(&server_id)?;
    Ok(registry.send(id, &command).await?)
}
