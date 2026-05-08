//! Stub for the RCON client wrapper.
//!
//! Real implementation lands in a follow-up PR alongside the BattlEye RCON
//! spike against a live Conan Exiles Enhanced server.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RconError {
    #[error("not yet implemented")]
    NotImplemented,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_error_renders() {
        assert_eq!(RconError::NotImplemented.to_string(), "not yet implemented");
    }
}
