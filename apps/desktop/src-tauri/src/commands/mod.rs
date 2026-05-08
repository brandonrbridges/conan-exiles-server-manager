//! Tauri command surface — the only `pub` Rust the UI can reach.
//!
//! Each command:
//! - Takes typed input (deserialised from JSON by Tauri).
//! - Acquires managed state (`Db`, `ConnectionRegistry`) via `tauri::State`.
//! - Returns `Result<T, AppError>`. `T` and `AppError` are both ts-rs
//!   exported, so the UI gets typed responses and discriminated errors.

pub mod servers;
pub mod settings;
