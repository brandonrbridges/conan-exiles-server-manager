//! Parsers for Conan Exiles Enhanced RCON command responses.
//!
//! Every parser here was written against captured real-server output,
//! documented in `docs/rcon-commands.md`. When you add a new one, capture
//! a fresh sample first via `examples/probe-raw` and add it as a test case.
//!
//! Parsers are pure functions — no I/O, no state — so they unit-test
//! cleanly without a live server.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Single online player as returned by `listplayers`.
///
/// Captured header (verified 2026-05-08):
/// ```text
/// Idx | Char name | Player name | User ID | Platform ID | Platform Name
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Player {
    pub idx: u32,
    pub char_name: String,
    pub player_name: String,
    pub user_id: String,
    pub platform_id: String,
    pub platform_name: String,
}

/// Parse the response of `listplayers` into a vector of players.
///
/// Empty server returns just the header row, so an empty Vec is the
/// expected outcome there. Returns `Err` if the header row is missing or
/// malformed (which would indicate a Funcom format change worth a bug
/// report rather than silent best-effort parsing).
pub fn parse_listplayers(body: &str) -> Result<Vec<Player>, ParseError> {
    let mut lines = body.lines().filter(|l| !l.trim().is_empty());

    let header = lines.next().ok_or(ParseError::Empty)?;
    if !is_listplayers_header(header) {
        return Err(ParseError::UnexpectedHeader(header.to_string()));
    }

    let mut players = Vec::new();
    for line in lines {
        let cells = split_pipe_row(line);
        if cells.len() < 6 {
            // Malformed row; skip rather than fail the whole parse — Conan
            // logs warnings about characters with corrupted state and we
            // shouldn't silently lose the rest of the list.
            continue;
        }
        let idx: u32 = cells[0]
            .parse()
            .map_err(|_| ParseError::BadIndex(cells[0].clone()))?;
        players.push(Player {
            idx,
            char_name: cells[1].clone(),
            player_name: cells[2].clone(),
            user_id: cells[3].clone(),
            platform_id: cells[4].clone(),
            platform_name: cells[5].clone(),
        });
    }

    Ok(players)
}

fn is_listplayers_header(line: &str) -> bool {
    let cells = split_pipe_row(line);
    cells.len() >= 6
        && cells[0].eq_ignore_ascii_case("Idx")
        && cells[1].eq_ignore_ascii_case("Char name")
        && cells[2].eq_ignore_ascii_case("Player name")
        && cells[3].eq_ignore_ascii_case("User ID")
}

fn split_pipe_row(line: &str) -> Vec<String> {
    line.split('|').map(|s| s.trim().to_string()).collect()
}

/// One row from `listbans`.
///
/// **Note**: as of 2026-05-08 we don't have a confirmed sample of the
/// populated `listbans` output — only the empty-state response
/// `Successfully executed: listbans`. The fields below are best-guess
/// based on the parallel structure with `listplayers`. Update once we
/// can capture real ban data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BannedPlayer {
    pub user_id: String,
    pub platform_id: String,
    pub player_name: Option<String>,
    pub reason: Option<String>,
}

/// Parse a `listbans` response. Empty list returns Ok(vec![]).
///
/// Returns `Err(ParseError::NoData)` when the server returned the generic
/// "Successfully executed:" line, which means "command worked but produced
/// no rows" — not strictly an error.
pub fn parse_listbans(body: &str) -> Result<Vec<BannedPlayer>, ParseError> {
    let trimmed = body.trim();
    if trimmed.starts_with("Successfully executed:") {
        return Ok(vec![]);
    }

    // Format unknown until we capture a real sample. Return empty for now
    // and let an integration test surface the issue when we have ban data.
    let mut bans = Vec::new();
    for line in trimmed.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cells = split_pipe_row(line);
        if cells.len() >= 2 {
            bans.push(BannedPlayer {
                user_id: cells[0].clone(),
                platform_id: cells[1].clone(),
                player_name: cells.get(2).cloned(),
                reason: cells.get(3).cloned(),
            });
        }
    }
    Ok(bans)
}

/// Parse a `GetServerSetting <key>` response.
///
/// - `Key=Value` → `Ok(Some(value))`. Empty values (`Key=`) yield `Some("")`.
/// - `Server setting 'X' not found.` → `Ok(None)`.
/// - Anything else → `Err(ParseError::UnexpectedFormat)`.
pub fn parse_get_server_setting(body: &str, key: &str) -> Result<Option<String>, ParseError> {
    let trimmed = body.trim();
    let not_found_marker = format!("Server setting '{}' not found.", key);
    if trimmed == not_found_marker {
        return Ok(None);
    }
    if let Some((k, v)) = trimmed.split_once('=') {
        if k.eq_ignore_ascii_case(key) {
            return Ok(Some(v.to_string()));
        }
    }
    Err(ParseError::UnexpectedFormat(trimmed.to_string()))
}

/// Parse the result of `broadcast <message>`. Returns `true` for the
/// known success line, `false` otherwise. The UI surfaces a toast either
/// way; this is mostly a sanity check.
pub fn parse_broadcast_ack(body: &str) -> bool {
    body.trim() == "Message has been broadcast."
}

/// What kind of acknowledgement a generic command returned.
#[derive(Debug, PartialEq, Eq)]
pub enum GenericAck<'a> {
    /// `Successfully executed: <cmd>` — operation succeeded, no payload.
    Success { command: &'a str },
    /// `Couldn't find the command: X. Try "help"` — caller mistyped.
    UnknownCommand { command: &'a str },
    /// `Syntax error, see help for usage.`
    SyntaxError,
    /// Anything else — treat as command-specific output.
    Other,
}

pub fn classify_ack(body: &str) -> GenericAck<'_> {
    let trimmed = body.trim();
    if let Some(rest) = trimmed.strip_prefix("Successfully executed: ") {
        return GenericAck::Success { command: rest };
    }
    if let Some(rest) = trimmed.strip_prefix("Couldn't find the command: ") {
        let cmd = rest.split('.').next().unwrap_or(rest);
        return GenericAck::UnknownCommand { command: cmd };
    }
    if trimmed == "Syntax error, see help for usage." {
        return GenericAck::SyntaxError;
    }
    GenericAck::Other
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("response body was empty")]
    Empty,

    #[error("unexpected header: {0:?}")]
    UnexpectedHeader(String),

    #[error("first column was not a numeric index: {0:?}")]
    BadIndex(String),

    #[error("server returned a no-data acknowledgement")]
    NoData,

    #[error("unexpected response format: {0:?}")]
    UnexpectedFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- listplayers ----

    #[test]
    fn listplayers_empty_server_returns_no_players() {
        // Captured 2026-05-08 against a fresh CESM Dev Server with no
        // players connected.
        let body = "Idx | Char name | Player name | User ID | Platform ID | Platform Name\n";
        let players = parse_listplayers(body).unwrap();
        assert!(players.is_empty());
    }

    #[test]
    fn listplayers_parses_synthetic_row() {
        // Synthetic — no connected-player capture available yet. Format
        // assumes pipe-separated rows mirroring the header columns.
        let body = "Idx | Char name | Player name | User ID | Platform ID | Platform Name\n\
                    0 | Brennorr | brandonrbridges | 12345 | 76561198000000000 | Steam\n\
                    1 | Eve | EvePlayer | 67890 | 76561198000000001 | Steam\n";
        let players = parse_listplayers(body).unwrap();
        assert_eq!(players.len(), 2);
        assert_eq!(players[0].idx, 0);
        assert_eq!(players[0].char_name, "Brennorr");
        assert_eq!(players[0].platform_name, "Steam");
        assert_eq!(players[1].user_id, "67890");
    }

    #[test]
    fn listplayers_rejects_unknown_header() {
        let body = "Some | Other | Header\n0 | foo | bar\n";
        let err = parse_listplayers(body).unwrap_err();
        assert!(matches!(err, ParseError::UnexpectedHeader(_)));
    }

    #[test]
    fn listplayers_skips_malformed_rows_but_keeps_good_ones() {
        let body = "Idx | Char name | Player name | User ID | Platform ID | Platform Name\n\
                    not-enough-cells\n\
                    0 | A | a | 1 | sid1 | Steam\n";
        let players = parse_listplayers(body).unwrap();
        assert_eq!(players.len(), 1);
        assert_eq!(players[0].char_name, "A");
    }

    // ---- listbans ----

    #[test]
    fn listbans_empty_returns_empty_vec() {
        // Captured 2026-05-08 with no bans.
        let body = "Successfully executed: listbans";
        let bans = parse_listbans(body).unwrap();
        assert!(bans.is_empty());
    }

    // ---- GetServerSetting ----

    #[test]
    fn get_server_setting_known_key() {
        // Captured 2026-05-08.
        assert_eq!(
            parse_get_server_setting("PVPEnabled=false", "PVPEnabled").unwrap(),
            Some("false".into())
        );
        assert_eq!(
            parse_get_server_setting("HarvestAmountMultiplier=1.0", "HarvestAmountMultiplier")
                .unwrap(),
            Some("1.0".into())
        );
        assert_eq!(
            parse_get_server_setting("ServerRegion=EU", "ServerRegion").unwrap(),
            Some("EU".into())
        );
    }

    #[test]
    fn get_server_setting_empty_value() {
        // `NPCRespawnMultiplier=` was captured as a real response.
        assert_eq!(
            parse_get_server_setting("NPCRespawnMultiplier=", "NPCRespawnMultiplier").unwrap(),
            Some("".into())
        );
    }

    #[test]
    fn get_server_setting_not_found() {
        // Captured 2026-05-08.
        let body = "Server setting 'XPMultiplier' not found.";
        assert_eq!(
            parse_get_server_setting(body, "XPMultiplier").unwrap(),
            None
        );
    }

    #[test]
    fn get_server_setting_rejects_unrelated_response() {
        let err = parse_get_server_setting("totally garbage", "X").unwrap_err();
        assert!(matches!(err, ParseError::UnexpectedFormat(_)));
    }

    // ---- broadcast ----

    #[test]
    fn broadcast_ack_recognises_canonical_ok() {
        assert!(parse_broadcast_ack("Message has been broadcast."));
        assert!(parse_broadcast_ack("Message has been broadcast.\n"));
    }

    #[test]
    fn broadcast_ack_rejects_anything_else() {
        assert!(!parse_broadcast_ack("Something unexpected"));
    }

    // ---- generic ack ----

    #[test]
    fn classify_ack_success() {
        assert_eq!(
            classify_ack("Successfully executed: listbans"),
            GenericAck::Success {
                command: "listbans"
            },
        );
    }

    #[test]
    fn classify_ack_unknown_command() {
        // Captured 2026-05-08: `Couldn't find the command: serverinfo. Try "help"`.
        assert_eq!(
            classify_ack(r#"Couldn't find the command: serverinfo. Try "help""#),
            GenericAck::UnknownCommand {
                command: "serverinfo"
            },
        );
    }

    #[test]
    fn classify_ack_syntax_error() {
        assert_eq!(
            classify_ack("Syntax error, see help for usage."),
            GenericAck::SyntaxError
        );
    }

    #[test]
    fn classify_ack_other_pass_through() {
        assert_eq!(classify_ack("PVPEnabled=false"), GenericAck::Other);
    }
}
