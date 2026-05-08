# Contributing

Thanks for thinking about contributing to Conan Exiles | Server Manager Enhanced. This document covers what to expect, how the workflow runs, and how to land changes cleanly.

## Code of Conduct

This project follows the [Contributor Covenant](./CODE_OF_CONDUCT.md). Be kind, give people the benefit of the doubt, and assume good faith.

## Ways to help

- **Report bugs** via [GitHub Issues](https://github.com/brandonrbridges/conan-exiles-server-manager/issues/new/choose).
- **Suggest features** via Issues, but please skim the [Roadmap project](https://github.com/brandonrbridges/conan-exiles-server-manager/projects) first — your idea may already be planned.
- **Run the beta channel** and report what breaks. See [README → Update channels](./README.md#update-channels).
- **Help with Linux testing** — Linux support lands in v0.1; we need testers on common distros (Ubuntu, Fedora, Arch).
- **Improve docs** — typos, broken links, unclear sections. PRs welcome with no prior discussion.
- **Code contributions** — see "Submitting code" below.

## Dev setup

### Prerequisites

- **Node.js 22+** and **pnpm 9+**
- **Rust 1.80+** (install via [rustup](https://rustup.rs/))
- **Tauri prerequisites** — see [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/) for OS-specific requirements (Xcode CLI tools on macOS, MSVC + WebView2 on Windows).
- A **Conan Exiles Enhanced dedicated server** to test against. Test against your own server, not someone else's, even if you have RCON creds.

### Bootstrap

```bash
git clone https://github.com/brandonrbridges/conan-exiles-server-manager.git
cd conan-exiles-server-manager
pnpm install
pnpm dev
```

`pnpm dev` boots the Tauri shell with hot reload on the React side and `cargo watch` on the Rust side.

### Common commands

```bash
pnpm dev           # Tauri dev shell with hot reload
pnpm build         # Production build
pnpm lint          # Biome + cargo clippy
pnpm format        # Biome + cargo fmt --write
pnpm test          # vitest + cargo test
pnpm typecheck     # tsc --noEmit
```

## Submitting code

### Branching

- Branch from `main` for everything. Name branches descriptively: `feat/player-list-search`, `fix/rcon-reconnect-loop`, `docs/contributing-update`.
- Keep PRs small and focused. One logical change per PR.

### Conventional Commits

PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/) — they become the squash-merge commit message and feed `release-please` for automated changelogs and version bumps.

```
feat: add player ban duration field
fix: prevent RCON reconnect loop on auth failure
docs: clarify Windows code-signing setup
chore: bump tauri to 2.1.4
refactor: extract polling scheduler to its own module
test: cover ban_cache cascade delete
```

Use `feat!:` or include `BREAKING CHANGE:` in the body for breaking changes.

### Pre-commit hooks

`lefthook` is installed automatically by `pnpm install` and runs:

- **Pre-commit** — `cargo fmt`, `biome format --write`, `biome check`
- **Pre-push** — `cargo clippy`, `tsc --noEmit`

If you need to bypass them in an emergency, use `git commit --no-verify` and explain in the PR description why. Don't make this a habit.

### CI must pass

Every PR runs CI on macOS and Windows. CI must be green before merge. If it's red, fix it — don't merge through a flake without investigating.

### Reviews

This project is small enough that review is a sanity check, not a gate. Self-merging is fine for clear, low-risk changes. Anything touching auth, secret handling, or RCON wire protocol gets a second pair of eyes — even if those eyes are yours after a tea break.

### Tests

- Add tests for any new logic in the Rust core (RCON parsing, polling diff, etc.).
- UI tests are not required for v0 — manual smoke testing during dev is fine.
- Don't write tests that mock RCON itself; mock the transport. The wire protocol is the bit most likely to drift.

## Reporting security issues

**Don't open a public issue for security bugs.** See [SECURITY.md](./SECURITY.md) for the responsible disclosure process.

## Licence

By contributing, you agree your contributions are licensed under the MIT licence (see [LICENSE](./LICENSE)).
