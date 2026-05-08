# Conan Exiles Enhanced — `sql` RCON command schema discovery

The Conan Exiles RCON `sql <query>` command runs raw SQL against the
server's SQLite database. Captured against a live CESM Dev Server on
**2026-05-08** to scope out features for v1.x.

## Why this matters

`sql` exposes the entire game-state DB. Potential v1.x features powered
purely by RCON (no SFTP needed):

- **Player history view** — last-online timestamps, guild membership,
  rank, alive/dead state, killer.
- **Base inspector** — building owners, damage state, destruction history.
- **Purge tracker** — `purgescores` table.
- **Server stats over time** — `serverPopulationRecordings`.
- **Diplomacy panel** — `diplomacy` table for inter-guild relationships.

## Tables available (verified)

```sql
sql SELECT name FROM sqlite_master WHERE type='table' ORDER BY name
```

Returns 25+ tables including:

- `characters`              — player records
- `character_stats`         — per-character stat values
- `character_buffs`         — active buffs/debuffs
- `character_id_reservation`
- `actor_position`          — player + thrall positions
- `actor_bounding_box`
- `buildings`               — base structures
- `building_instances`      — individual placeables
- `buildable_health`        — durability state
- `static_buildables`       — fixed-place world objects
- `destruction_history`     — log of building destruction
- `purgescores`             — purge progression per clan
- `guilds`                  — clan records
- `diplomacy`               — clan relationships
- `events` / `game_events`  — gameplay event log
- `follower_markers`
- `item_inventory`
- `item_properties`
- `properties`
- `mod_controllers`
- `serverPopulationRecordings`
- `used_smart_objects`
- `dw_settings`             — Dreamworld (Conan engine layer)
- `sqlite_sequence`, `sqlite_stat1` — SQLite internals

## `characters` schema

```
cid | name                 | type    | notnull | pk
 0  | playerId             | TEXT    |  0      |  0
 1  | id                   | BIGINT  |  1      |  1   (primary key)
 2  | char_name            | TEXT    |  1      |  0
 3  | level                | INTEGER |  0      |  0
 4  | rank                 | INTEGER |  0      |  0
 5  | guild                | BIGINT  |  0      |  0
 6  | isAlive              | BOOLEAN |  0      |  0
 7  | killerName           | TEXT    |  0      |  0
 8  | lastTimeOnline       | INTEGER |  0      |  0
 9  | killerId             | TEXT    |  0      |  0
10  | lastServerTimeOnline | REAL    |  0      |  0
```

Note: column is `char_name`, not `name` — querying `name` returns
`Successfully executed:` (no error, just empty result; a Conan-RCON
quirk worth handling).

## `sql` response format

Empty result:

```
Successfully executed: sql <query>
```

Non-empty result:

```
   col1 |   col2 |
#0  v1  |   v2   |
#1  v3  |   v4   |
```

- First line: column names, right-padded, pipe-separated.
- Subsequent lines: `#<rowidx> <values…> |`.
- Pipe-separated columns; values may contain spaces.

## Caveats

- `sql .tables` returns "Successfully executed:" — only standard SQL
  works, not SQLite shell dot-commands.
- Some table sizes balloon fast (e.g. `actor_position` has rows for
  every active actor — thousands at scale). Always include `LIMIT`.
- Read-only queries are safe; UPDATE/DELETE work too — be careful in
  production. **A future "SQL console" feature should default to a
  read-only confirmation prompt.**
- Schema may shift between Conan patches — anything we surface in the
  UI should tolerate column drift gracefully.

## Suggested v1.x feature implementations

```sql
-- Player history view (sortable, paged)
SELECT id, char_name, level, rank, guild, isAlive,
       lastTimeOnline, killerName
FROM characters
ORDER BY lastTimeOnline DESC
LIMIT 50;

-- Active guilds
SELECT g.id, g.name, COUNT(c.id) as members, MAX(c.lastTimeOnline) as last_active
FROM guilds g LEFT JOIN characters c ON c.guild = g.id
GROUP BY g.id;

-- Recent base destructions
SELECT * FROM destruction_history
ORDER BY rowid DESC LIMIT 20;

-- Purge state per clan
SELECT * FROM purgescores ORDER BY purgeScore DESC;
```

## Risk: `sql` exposure

The `sql` RCON command is a footgun. Anyone with RCON credentials can
run arbitrary SQL — including table drops. CESM's UI should:

1. Never auto-run user-typed SQL without explicit confirmation.
2. Surface results read-only by default; a "destructive query" toggle
   gates UPDATE / DELETE / DROP / ALTER.
3. Log every executed `sql` query into the connection-level event feed
   so admins have an audit trail.
