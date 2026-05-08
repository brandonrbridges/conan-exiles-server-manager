//! Server overview command — aggregates the per-server hero stats the
//! dashboard shows above the player table.
//!
//! Conan's RCON has no single `serverinfo`-style command. The hero strip
//! is a handful of `GetServerSetting` calls plus the player count from
//! `listplayers`. Bundling them in one Tauri call keeps the UI from
//! managing N parallel queries and avoids burning karma on chatter.

use rcon_client::parsers::{parse_get_server_setting, parse_listplayers};
use rcon_client::ConnectionRegistry;
use serde::Serialize;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Keys the dashboard reads via `GetServerSetting`. This list is the
/// **verified-working** subset captured 2026-05-08 — others either
/// return `not found` or live in different ini files (ServerName,
/// MaxPlayers, RconMaxKarma — none of which `GetServerSetting` exposes).
const OVERVIEW_KEYS: &[&str] = &[
    "PVPEnabled",
    "ServerRegion",
    "HarvestAmountMultiplier",
    "ItemSpoilRateScale",
    "ResourceRespawnSpeedMultiplier",
    "StaminaCostMultiplier",
    "ClanMaxSize",
    "ChatMaxMessageLength",
    "IsBattlEyeEnabled",
];

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ServerOverview {
    /// Number of players currently connected, derived from `listplayers`.
    pub player_count: u32,
    /// Verified `GetServerSetting` keys → values. Missing keys are
    /// omitted rather than rendered as "not found".
    pub settings: Vec<ServerSetting>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ServerSetting {
    pub key: String,
    pub value: String,
}

#[tauri::command]
pub async fn server_overview(
    server_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> AppResult<ServerOverview> {
    let id = Uuid::parse_str(&server_id)
        .map(rcon_client::ServerId)
        .map_err(|_| AppError::Invalid {
            message: "server id must be a uuid".into(),
        })?;

    let players_body = registry.send(id, "listplayers").await?;
    let players = parse_listplayers(&players_body).map_err(|e| AppError::Rcon {
        message: format!("could not parse listplayers: {e}"),
    })?;

    let mut settings = Vec::with_capacity(OVERVIEW_KEYS.len());
    for key in OVERVIEW_KEYS {
        // One `GetServerSetting` call per key. Conan's RCON has no
        // batch-read so we serialise these. Each open() is karma-cheap
        // at the configured rate; the registry handles connect-per-call.
        let body = registry
            .send(id, &format!("GetServerSetting {key}"))
            .await?;
        match parse_get_server_setting(&body, key) {
            Ok(Some(value)) => {
                if !value.is_empty() {
                    settings.push(ServerSetting {
                        key: (*key).into(),
                        value,
                    });
                }
            }
            Ok(None) => {} // silently absent — server doesn't expose this key
            Err(_) => {
                // Don't fail the whole overview because one key returned
                // something weird; surface what we did parse.
            }
        }
    }

    Ok(ServerOverview {
        player_count: players.len() as u32,
        settings,
    })
}
