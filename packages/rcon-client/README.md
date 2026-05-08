# `rcon-client`

Internal Rust crate that wraps the upstream [`rcon`](https://crates.io/crates/rcon) (Source RCON over TCP) with the bits a desktop control panel needs:

- **Per-server connection registry** keyed by `ServerId` (a UUID newtype). At most one live connection per server — Source RCON is single-session.
- **Cooperative reconnect** with exponential backoff (1s → 2s → 4s → 8s → 30s cap). Cancellable via [`tokio_util::sync::CancellationToken`] so closing a server in the UI tears the loop down promptly.
- **Per-call `send` timeout** (configurable on `RconConfig`, default 5s). Conan freezes briefly during world-save; without a timeout, a `kick` issued mid-save would hang.
- **`secrecy::SecretString`** for the password — zeroed on drop, doesn't leak through `Debug`.
- **`tokio::sync::watch` state stream** so the UI can react to `Connecting` → `Open` → `Reconnecting` → `Failed` transitions without polling.

## Why Source RCON

Conan Exiles uses **Source RCON over TCP** despite shipping with the BattlEye anti-cheat. The two are unrelated. Default RCON port is 7779 (game is 7777, Steam query is 7778).

When constructing a connection we explicitly disable both Minecraft quirks (1413-byte command-length cap + 3ms send/recv sleep) and Factorio quirks (single-packet response mode) on the upstream `rcon::Builder`. Conan needs neither.

## Public API at a glance

```rust
use rcon_client::{ConnectionRegistry, RconConfig, ServerId};
use secrecy::SecretString;

let registry = ConnectionRegistry::new();
let id = ServerId::new();
let cfg = RconConfig::new("conan.example.com", 7779, SecretString::from("password"));

registry.open(id, cfg).await;
let players = registry.send(id, "listplayers").await?;
registry.close(id).await;
```

## Tests

Integration tests bring up a tiny in-process Source RCON server (`tests/mock_server.rs`) on `127.0.0.1:0`, drive auth + the multi-packet response idiom, and exercise reconnect and cancellation paths. No live Conan server required for CI.

End-to-end smoke testing against an actual Conan Exiles Enhanced server is a manual gate before each release; once a community-maintained Conan headless container exists we'll add it as a CI job.

## Deferred (tracked as v1.0 issues)

- Tauri-side wiring (`#[tauri::command]` and event emission) — lands in PR #3 alongside SQLite + keychain.
- Live-server integration test in CI (Conan dedicated server in headless docker).
- Privacy review of `tracing` debug logs — RCON command bodies currently log at debug level and may include player IDs.

## Licence

MIT — same as the parent project.
