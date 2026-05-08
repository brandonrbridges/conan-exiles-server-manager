# Conan Exiles Enhanced — RCON command reference

Verified against a live Conan Exiles Enhanced (UE5) Linux dedicated
server on **2026-05-08** (game build released 2026-05-05). All output
samples are real, captured via `packages/rcon-client/examples/probe-raw`.

This document is the source of truth for the parsers in
`packages/rcon-client/src/parsers/` and the design of the live-admin
features in CESM. Anything in `docs/plans/2026-05-08-v0-design.md`
that disagrees with this file is wrong (the design predated live-server
verification).

## Wire-protocol caveats

Two Conan-specific quirks the upstream `rcon` crate doesn't handle out of
the box:

1. **Single-packet responses only.** Conan ignores the empty-marker
   packet that Source RCON's multi-packet idiom uses — sending one hangs
   the client until timeout. We pin `Builder::enable_factorio_quirks(true)`
   despite the misleading name; it just means "single-packet mode".
2. **Server closes the TCP connection after every command.** The session
   is one connection per command: connect → auth → send cmd → read
   response → server closes. Our wrapper has to reconnect+re-auth between
   commands, not reuse the socket.

## Rate limiting (`RconMaxKarma`)

Conan's RCON has a karma-based rate limiter. Default `RconMaxKarma=60`
in `Game.ini` under `[RconPlugin]`. Hitting the limit returns:

```
Too many commands, try again later.
```

The dev server runs with `RconMaxKarma=5000` so probing isn't blocked.
Production users should leave it at default for security.

## Available commands (full)

Captured from `help` against a fresh Enhanced server. The `Usage:` lines
shown are the server's own help text.

| Command | Usage | Notes |
| --- | --- | --- |
| `help` | `help "Optional command name filter". lists all available rcon commands.` | Filter is a substring match. |
| `listplayers` | `listplayers` | Online players table. |
| `listbans` | `listbans` | Banned players table. |
| `broadcast` | `broadcast <message>` | Server-wide chat message. |
| `KickPlayer` | `kickplayer (index|name|userid|platformid|player) <identification> <message>` | Identifier kind is required. |
| `BanPlayer` | `banplayer (index|name|userid|platformid|player) <identification> <message>` | Same selector pattern. |
| `UnbanPlayer` | `unbanplayer <userid|platformid>` | Only by id, not name. |
| `WhitelistPlayer` | `whitelistplayer <userid|platformid>` | Only by id. |
| `UnWhitelistPlayer` | `unwhitelistplayer <userid|platformid>` | Only by id. |
| `Shutdown` | `shutdown` | Graceful server stop. |
| `restart` | `restart` | Restart in place. |
| `GetServerSetting` | `GetServerSetting <Setting Name>` | Read runtime ini value. |
| `SetServerSetting` | `SetServerSetting <Setting Name> <Setting Value>` | Write runtime ini value. |
| `GetLandOwner` | `getlandowner <x> <y>` | Map-coord land ownership query. |
| `buildingquery` | `buildingquery (list|destroy) <owner id> <filters>` | Lists or destroys buildings by owner. |
| `BuildingDestroy` | `buildingdestroy <building ids>` | By IDs from `buildingquery`. |
| `BuildingContribution` | `buildingcontribution <building id>` | Per-base contribution stats. |
| `con` | `con <id> <command> <args>` | Run an in-game console command as a connected player. |
| `exec` | `exec (args)` | Generic exec — args TBD. |
| `sql` | `sql <query>` | Raw SQL against the server's SQLite-equivalent DB. **Powerful and dangerous.** |
| `netprofile` | `netprofile (enable|disable)` | Net profiler toggle. |
| `dumpticks` | `dumpticks` | Per-tick performance dump. |
| `memreport` | `memreport` | Memory diagnostics. |
| `memreportsilentevent` | `memreportsilentevent (key)` | Tagged memory dump. |
| `validateallbuildings` | `validateallbuildings` | Building integrity audit. |

**Commands that DO NOT exist** (the v0 design doc was wrong):
`serverinfo`, `banlist` (use `listbans`), `version`, `saveworld`.

## Output samples

### `listplayers` (empty server)

```
Idx | Char name | Player name | User ID | Platform ID | Platform Name
```

Header-only line. Rows follow the same six-column pipe-separated
shape when players are connected. **TODO**: capture sample with a
live player to lock the row format.

### `listbans` (no bans)

```
Successfully executed: listbans
```

The "no rows" case returns the success line instead of an empty
table. **TODO**: capture sample with an actual ban to lock format.

### `GetServerSetting <known-key>`

```
PVPEnabled=false
HarvestAmountMultiplier=1.0
ClanMaxSize=30
ChatMaxMessageLength=512
ServerRegion=EU
AdminPassword=admin_dev_cesm
IsBattlEyeEnabled=false
ItemSpoilRateScale=1.0
StaminaCostMultiplier=1.0
ResourceRespawnSpeedMultiplier=1.0
```

`Key=Value`, one line. Empty value is possible: `NPCRespawnMultiplier=`.

### `GetServerSetting <unknown-key>`

```
Server setting 'XPMultiplier' not found.
```

### Verified known-good `GetServerSetting` keys

| Key | Type | Sample |
| --- | --- | --- |
| `PVPEnabled` | bool | `false` |
| `ServerRegion` | string | `EU` |
| `HarvestAmountMultiplier` | float | `1.0` |
| `ItemSpoilRateScale` | float | `1.0` |
| `ResourceRespawnSpeedMultiplier` | float | `1.0` |
| `StaminaCostMultiplier` | float | `1.0` |
| `ClanMaxSize` | int | `30` |
| `ChatMaxMessageLength` | int | `512` |
| `AdminPassword` | string | (echoes the configured value) |
| `IsBattlEyeEnabled` | bool | `false` |

Settings NOT exposed via `GetServerSetting` (silently "not found") even
though they exist in `ServerSettings.ini`/`Engine.ini`:
- `ServerName`, `MaxPlayers`, `ServerCommunity`, `RconMaxKarma`
- These live in different ini files; the RCON `GetServerSetting` API
  appears to only target `ServerSettings.ini`'s `[ServerSettings]`
  block (or a subset of it).

### `broadcast <message>`

```
Message has been broadcast.
```

### `KickPlayer` / `BanPlayer` (no args)

```
Syntax error, see help for usage.
```

### `BanPlayer name <unknown-name> <message>`

```
No player with name FakePlayerForTesting.
```

### `UnbanPlayer <unbanned-id>`

```
Failed to ban player FakePlayerForTesting, was not banned.
```

(Yes, the error message says "ban" instead of "unban" — Funcom typo.)

### `GetLandOwner 0 0`

```
The location ( 0 , 0 ) is unclaimed
```

### `buildingquery list` (missing args)

```
Did not specify enough args. Syntax should be 'BuildingQuery (list|destroy) <owner id> <filters>'
```

### `sql SELECT 1`

```
   1 |
#0 1 |
```

First line: column header (right-padded). Second+ lines: `#<rowidx> value |`. Multiple columns are pipe-separated.

### `sql <query-with-empty-result>`

```
Successfully executed: sql SELECT name FROM characters LIMIT 5
```

When the query returns zero rows, RCON falls back to the generic
"successful no-data" response.

### Generic patterns

| Response shape | Meaning |
| --- | --- |
| `Successfully executed: <command>` | Command ran, no payload to return. |
| `Couldn't find the command: X. Try "help"` | Unknown RCON command. |
| `Syntax error, see help for usage.` | Argument parsing failed. |
| `Server setting 'X' not found.` | `GetServerSetting` for unknown key. |
| `Too many commands, try again later.` | RconMaxKarma exhausted. |

## Implementation notes for the rcon-client crate

1. **Default to single-packet mode** for new connections. Multi-packet
   mode hangs forever against Conan.
2. **Reconnect per command.** Don't try to reuse a socket — Conan closes
   it. Make `send` a self-contained connect+auth+send+drop. Caller-facing
   API stays the same; the registry's per-server `RconConfig` is what's
   reused.
3. **Detect rate-limit responses.** If the response body matches `Too many
   commands, try again later.`, surface a typed `RconError::RateLimited`
   variant rather than a generic transport error.
4. **No keep-alive needed.** Source RCON has no native keep-alive and
   Conan doesn't expect one — the connection is short-lived by design.
5. **`Successfully executed: <cmd>` is success-with-no-data.** Parsers
   should treat it as "operation succeeded, output empty" rather than as
   the actual data.

## What's still TODO before live-admin features ship

- Capture `listplayers` with a connected player.
- Capture `listbans` with a real banned account.
- Verify `BanPlayer` success response shape (need a real online player).
- Verify `KickPlayer` success response shape.
- Try `con <pid> <command>` to see what in-game console commands can be
  triggered remotely (potentially huge surface for v1.x features).
- Map the `sql` schema — what tables exist, what columns. Could power
  a "player history" / "base inspector" feature in v1.x.
