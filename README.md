# Conan Exiles | Server Manager Enhanced

A modern, free, open-source desktop control panel for Conan Exiles Enhanced (UE5) dedicated servers.

> Status: **early — v0 in design**. See [`docs/plans/2026-05-08-v0-design.md`](./docs/plans/2026-05-08-v0-design.md) for the v0 specification.

## What this is

A cross-platform desktop app (Windows + macOS) that connects to your Conan Exiles Enhanced dedicated server over RCON and gives you a clean, live interface for the things you do every day: kicking trolls, banning grief-builders, broadcasting messages, promoting admins, watching player counts.

Built for self-hosters and rented-host customers alike. No telemetry. No accounts. Your credentials never leave your machine.

## Roadmap

- **v0** — Live admin via RCON: multi-server connections, player list, kick/ban/unban, broadcast, admin promotion flow, raw RCON console.
- **v1** — SFTP layer: live log tail, chat feed, log search, crash detection. Config editor. Mods panel with Workshop browser.
- **v2** — Optional paid hosted "Pro" sidecar: Discord webhook alerts, scheduled restarts/backups, cross-device sync.

See the [GitHub Project](#) for the live roadmap.

## Stack

- **Tauri 2** (Rust core + webview UI) — single ~10 MB binary, OS-native keychain, no Electron bloat.
- **Vite + React + TypeScript** — UI shell.
- **shadcn/ui** — component layer.
- **SQLite** — local persistence (connection metadata only; passwords stay in OS keychain).

## Licence

MIT. See [`LICENSE`](./LICENSE).

## Contributing

Coming soon — contribution guide and code of conduct land with the v0 scaffolding pass.
