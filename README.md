# Conan Exiles | Server Manager Enhanced

[![Licence: MIT](https://img.shields.io/badge/licence-MIT-blue.svg)](./LICENSE)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-orange.svg)](#status)
[![Build](https://img.shields.io/github/actions/workflow/status/brandonrbridges/conan-exiles-server-manager/ci.yml?branch=main)](https://github.com/brandonrbridges/conan-exiles-server-manager/actions)
[![Latest release](https://img.shields.io/github/v/release/brandonrbridges/conan-exiles-server-manager?include_prereleases)](https://github.com/brandonrbridges/conan-exiles-server-manager/releases)
[![Downloads](https://img.shields.io/github/downloads/brandonrbridges/conan-exiles-server-manager/total)](https://github.com/brandonrbridges/conan-exiles-server-manager/releases)

A free, open-source desktop control panel for **Conan Exiles Enhanced** (UE5) dedicated servers.

> **The first server manager built specifically for Conan Exiles Enhanced.** Cross-platform, no telemetry, no accounts, no subscription required. Your credentials never leave your machine.

## Status

Pre-alpha. v0 is in active design — see [`docs/plans/2026-05-08-v0-design.md`](./docs/plans/2026-05-08-v0-design.md) for the full specification, and the [Roadmap project](https://github.com/users/brandonrbridges/projects) for live progress.

**ETA for v0 release:** ~3 weeks from 2026-05-08. [Watch the repo](https://github.com/brandonrbridges/conan-exiles-server-manager) to be notified when it ships.

## What it does (v0)

A modern desktop app (Windows + macOS) that connects to your Conan Exiles Enhanced dedicated server over RCON and gives you a clean, live interface for everyday admin work:

- **Multi-server connections** — manage every server you run from one app.
- **Live player list** — see who's online, in real time, with kick/ban/promote actions.
- **Bans manager** — cached list of banned players with one-click unban.
- **Broadcast composer** — send a message to everyone on the server.
- **Server controls** — save world, restart, status at a glance.
- **Raw RCON console** — power-user escape hatch for any command.
- **Beta channel** — opt in to early builds and help shape the product.

## What's coming

- **v1.0** — log tail, live chat feed, log search, crash detection (SFTP integration).
- **v1.1** — config editor with validation for `ServerSettings.ini` / `Engine.ini` / `Game.ini`.
- **v1.2** — mods panel: Workshop browser, mod sync, version pinning.
- **v2.0** — optional paid Pro tier: Discord webhook alerts, scheduled restarts and backups, cross-device sync.

See [`docs/plans/2026-05-08-v0-design.md`](./docs/plans/2026-05-08-v0-design.md) for the full roadmap.

## Why this exists

Conan Exiles Enhanced shipped on 2026-05-05 — Funcom's free UE5 upgrade. Existing server-admin tooling is Windows-only, FTP-based, and was written for the UE4 build. This project is a clean-slate, modern, open-source alternative — built for Enhanced from day one.

## Stack

- **[Tauri 2](https://tauri.app/)** — Rust core + native webview UI. Single ~10 MB binary.
- **Vite + React + TypeScript** — UI shell.
- **[shadcn/ui](https://ui.shadcn.com/)** — component layer.
- **SQLite** for local persistence; passwords stay in the OS keychain (macOS Keychain, Windows Credential Manager).

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the full architecture.

## Update channels

- **Stable** (default) — solid, tested releases.
- **Beta** — early access to upcoming features. Some bugs expected; bug reports actively welcomed.

Switch channels in **Settings → Update channel** once installed.

## Privacy

- No telemetry. No phone-home. No analytics.
- No accounts. No login.
- Credentials stored in your OS keychain only — never written to disk, never sent over the network except to your own server.

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for dev setup, branching, and submission guidelines. Beta testers, bug reporters, and Linux testers especially welcome.

For security issues, see [`SECURITY.md`](./SECURITY.md) — please don't open public issues for vulnerabilities.

## Licence

[MIT](./LICENSE).

## Acknowledgements

- **Funcom** for Conan Exiles and the Enhanced upgrade.
- The Conan Exiles community for years of self-built tooling that informed this design.
