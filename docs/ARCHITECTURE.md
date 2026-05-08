# Architecture

High-level system map for Conan Exiles | Server Manager Enhanced. For the v0 specification, see [`plans/2026-05-08-v0-design.md`](./plans/2026-05-08-v0-design.md).

## Overview

```
┌──────────────────────────────────────────────────────────────────┐
│ Desktop app (Tauri 2)                                            │
│                                                                  │
│   ┌──────────────────────┐        ┌────────────────────────┐    │
│   │ Webview UI           │ invoke │ Rust core              │    │
│   │ Vite + React + TS    │ ─────▶ │ src-tauri/             │    │
│   │ shadcn/ui            │ ◀───── │  - RCON client pool    │    │
│   │                      │ events │  - SQLite persistence  │    │
│   └──────────────────────┘        │  - Keychain            │    │
│                                   │  - Polling scheduler   │    │
│                                   └────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
                                              │
                                              │ TCP / Source RCON
                                              ▼
                              ┌──────────────────────────────┐
                              │ Conan Exiles Enhanced server │
                              │ (user's dedicated server)    │
                              └──────────────────────────────┘
```

## Layers

### Webview UI (`apps/desktop/src/`)

- React + TypeScript single-page app.
- Communicates with Rust via Tauri's `invoke()` (request/response) and `listen()` (events).
- No direct network or filesystem access — everything routed through the Rust core.

### Rust core (`apps/desktop/src-tauri/`)

- Owns all I/O: TCP RCON connections, SQLite, OS keychain, file system.
- Hosts the polling scheduler — wakes up every N seconds while a server is the active view, runs `listplayers`, diffs against the last snapshot, emits events.
- Exposes a small surface of Tauri commands to the UI: `connect`, `disconnect`, `send_command`, `list_servers`, `save_server`, `delete_server`, etc.

### Shared types (`packages/rcon-client/`)

- Reusable Rust crate wrapping the upstream `rcon` crate (Source RCON over TCP) with a per-server connection registry, reconnect-with-backoff, per-call timeouts, and cooperative cancellation.
- TypeScript types generated via `ts-rs` so the UI sees the same shapes as the Rust core. Build step regenerates `apps/desktop/src/types/generated.ts`.

## Data flow: a typical "kick player" action

1. User clicks **Kick** on a row in the Players view.
2. Modal collects an optional reason; user confirms.
3. UI calls `invoke('kick_player', { server_id, player_name, reason })`.
4. Rust core looks up the active RCON connection for `server_id`, sends `kickplayer "<name>" "<reason>"`.
5. Server responds; Rust returns the response string to the UI.
6. UI optimistically removes the player from the list and shows a toast; next polling tick confirms.

## Persistence

- **SQLite** at `$APP_DATA/cesm/db.sqlite` — connection metadata only (server name, host, port, timestamps).
- **OS keychain** — RCON password and optional Admin password per server. Two entries: `cesm:{server_id}:rcon` and `cesm:{server_id}:admin`.
- **No remote sync** in v0–v1. Cross-device sync is a v2.0 Pro feature.

## RCON connection lifecycle

- Lazy: a connection opens only when its server is the active view.
- Single connection per server (RCON is single-session by protocol).
- Reconnect with exponential backoff: 1s, 2s, 4s, 8s, max 30s.
- Closes on view switch or app blur. Polling pauses on blur.

## Build and release pipeline

- **CI** runs on every PR and push to `main`. Matrix: `macos-latest` + `windows-latest`. Lint, typecheck, test, smoke build.
- **release-please** opens auto-updating release PRs based on Conventional Commits.
- **release.yml** triggers on `vX.Y.Z` tags: cross-platform Tauri build, code-sign macOS, attach signed installers + checksums to GitHub Release.
- **beta.yml** triggers on `vX.Y.Z-beta.N` tags.
- **Tauri updater** reads `updater/stable.json` or `updater/beta.json` from GitHub Releases based on the user's chosen channel.

## Future layers (post-v0)

- **SFTP adapter** (v1.0) — adds a second connection type alongside RCON. Used for log tail, chat feed, log search, config editing.
- **Mod manager** (v1.2) — Workshop browser, mod sync, conflict detection.
- **Pro hosted sidecar** (v2.0) — Cloudflare Workers + D1 + R2. Talks to the user's RCON/SFTP from the cloud on a schedule. Stripe billing. Discord webhooks. Cross-device sync.

## Non-goals

- Web/SaaS dashboard. The desktop client is the product. Pro is a paid sidecar to the desktop app, not a replacement for it.
- In-app modding tools (model editing, asset packaging). The official Mod Dev Kit owns that space.
- A custom RCON wire protocol. We use what Conan ships.
