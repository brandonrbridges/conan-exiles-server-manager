//! Player-management Tauri commands — list, kick, ban, unban, broadcast.
//!
//! Each command opens an RCON session via the registered handle for the
//! given server, runs one Conan-format command, and parses the response
//! through `rcon_client::parsers`. UI sees typed shapes (`Player`,
//! `BannedPlayer`) rather than raw text.

use rcon_client::parsers::{
    parse_broadcast_ack, parse_listbans, parse_listplayers, BannedPlayer, GenericAck, Player,
};
use rcon_client::{parsers, ConnectionRegistry};
use serde::Deserialize;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

fn parse_server_id(id: &str) -> AppResult<rcon_client::ServerId> {
    Uuid::parse_str(id)
        .map(rcon_client::ServerId)
        .map_err(|_| AppError::Invalid {
            message: "server id must be a uuid".into(),
        })
}

async fn send_command(
    registry: &ConnectionRegistry,
    server_id: rcon_client::ServerId,
    cmd: &str,
) -> AppResult<String> {
    Ok(registry.send(server_id, cmd).await?)
}

#[tauri::command]
pub async fn list_players(
    server_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<Vec<Player>> {
    let id = parse_server_id(&server_id)?;
    let body = send_command(&registry, id, "listplayers").await?;
    parse_listplayers(&body).map_err(|e| AppError::Rcon {
        message: format!("could not parse listplayers response: {e}"),
    })
}

#[tauri::command]
pub async fn list_bans(
    server_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<Vec<BannedPlayer>> {
    let id = parse_server_id(&server_id)?;
    let body = send_command(&registry, id, "listbans").await?;
    parse_listbans(&body).map_err(|e| AppError::Rcon {
        message: format!("could not parse listbans response: {e}"),
    })
}

/// How to identify a player for `KickPlayer` / `BanPlayer`. Conan's
/// `kickplayer` and `banplayer` commands expect this as the first arg.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum PlayerIdentifierKind {
    Index,
    Name,
    UserId,
    PlatformId,
    Player,
}

impl PlayerIdentifierKind {
    fn as_arg(self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::Name => "name",
            Self::UserId => "userid",
            Self::PlatformId => "platformid",
            Self::Player => "player",
        }
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct KickInput {
    pub kind: PlayerIdentifierKind,
    pub identifier: String,
    pub message: String,
}

#[tauri::command]
pub async fn kick_player(
    server_id: String,
    input: KickInput,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<String> {
    let id = parse_server_id(&server_id)?;
    let cmd = format!(
        "KickPlayer {} {} {}",
        input.kind.as_arg(),
        quote_arg(&input.identifier),
        quote_arg(&input.message),
    );
    let body = send_command(&registry, id, &cmd).await?;
    Ok(body)
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct BanInput {
    pub kind: PlayerIdentifierKind,
    pub identifier: String,
    pub message: String,
}

#[tauri::command]
pub async fn ban_player(
    server_id: String,
    input: BanInput,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<String> {
    let id = parse_server_id(&server_id)?;
    let cmd = format!(
        "BanPlayer {} {} {}",
        input.kind.as_arg(),
        quote_arg(&input.identifier),
        quote_arg(&input.message),
    );
    let body = send_command(&registry, id, &cmd).await?;
    Ok(body)
}

#[tauri::command]
pub async fn unban_player(
    server_id: String,
    user_or_platform_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<String> {
    let id = parse_server_id(&server_id)?;
    let cmd = format!("UnbanPlayer {}", quote_arg(&user_or_platform_id));
    let body = send_command(&registry, id, &cmd).await?;
    Ok(body)
}

#[tauri::command]
pub async fn broadcast(
    server_id: String,
    message: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<()> {
    let id = parse_server_id(&server_id)?;
    let body = send_command(&registry, id, &format!("broadcast {message}")).await?;
    if !parse_broadcast_ack(&body) {
        // Conan still echoes "Message has been broadcast." reliably;
        // anything else is a server-side anomaly worth surfacing.
        if let GenericAck::SyntaxError = parsers::classify_ack(&body) {
            return Err(AppError::Invalid {
                message: "broadcast text was rejected by the server".into(),
            });
        }
        return Err(AppError::Rcon {
            message: format!("unexpected broadcast response: {body}"),
        });
    }
    Ok(())
}

/// Wrap an RCON command argument in double quotes if it contains
/// whitespace. Conan's command parser is space-delimited; quoted
/// arguments survive the split.
fn quote_arg(value: &str) -> String {
    if value.is_empty() {
        return String::from("\"\"");
    }
    if value.contains(char::is_whitespace) || value.contains('"') {
        let escaped = value.replace('"', "\\\"");
        return format!("\"{escaped}\"");
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_arg_passes_simple_values_through() {
        assert_eq!(quote_arg("brandonrbridges"), "brandonrbridges");
        assert_eq!(quote_arg("12345"), "12345");
    }

    #[test]
    fn quote_arg_quotes_whitespace() {
        assert_eq!(quote_arg("a b c"), "\"a b c\"");
    }

    #[test]
    fn quote_arg_escapes_inner_quotes() {
        assert_eq!(quote_arg(r#"say "hi""#), r#""say \"hi\"""#);
    }

    #[test]
    fn quote_arg_handles_empty() {
        assert_eq!(quote_arg(""), "\"\"");
    }

    #[test]
    fn identifier_kind_serialises_to_lowercase() {
        assert_eq!(PlayerIdentifierKind::UserId.as_arg(), "userid");
        assert_eq!(PlayerIdentifierKind::PlatformId.as_arg(), "platformid");
    }
}
