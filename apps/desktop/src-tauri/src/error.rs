//! Application-wide error type.
//!
//! Every Tauri command returns `Result<T, AppError>`. The variants serialize
//! to a discriminated union so the UI can `switch` on `kind` for typed
//! handling rather than parsing free-text error messages.

use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Error, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppError {
    #[error("server not found")]
    ServerNotFound,

    #[error("authentication failed")]
    AuthFailed,

    #[error("not connected")]
    NotConnected,

    #[error("operation timed out")]
    Timeout,

    #[error("storage error: {message}")]
    Storage { message: String },

    #[error("keychain error: {message}")]
    Keychain { message: String },

    #[error("rcon error: {message}")]
    Rcon { message: String },

    #[error("invalid input: {message}")]
    Invalid { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Storage {
            message: err.to_string(),
        }
    }
}

impl From<rusqlite_migration::Error> for AppError {
    fn from(err: rusqlite_migration::Error) -> Self {
        Self::Storage {
            message: err.to_string(),
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(err: keyring::Error) -> Self {
        Self::Keychain {
            message: err.to_string(),
        }
    }
}

impl From<rcon_client::RconError> for AppError {
    fn from(err: rcon_client::RconError) -> Self {
        match err {
            rcon_client::RconError::AuthFailed => Self::AuthFailed,
            rcon_client::RconError::NotConnected => Self::NotConnected,
            rcon_client::RconError::Timeout => Self::Timeout,
            rcon_client::RconError::UnknownServer => Self::ServerNotFound,
            rcon_client::RconError::Transport(e) => Self::Rcon {
                message: e.to_string(),
            },
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal {
            message: err.to_string(),
        }
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        Self::Internal {
            message: err.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
