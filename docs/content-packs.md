# LemonCraft Content Pack Mechanism Design

> Status: design draft v0.1
> Scope: asset loading layer for the client and singleplayer server

## 1. Background & Goals

LemonCraft plans to streamline gameplay: full content (sites, creatures, recipes,
music, etc.) is preserved as **optional content packs**, while the core loads a
streamlined experience by default. This mechanism is also the foundation for a
future mod ecosystem — a content pack is the smallest unit of a mod.

Goals:

1. Content can be enabled/disabled per pack; multiple packs can stack
2. Shipped packs (official "full" pack) and player-installed packs (mods) coexist
3. Integrate with the existing asset system with minimal changes
4. Fail loudly on broken/conflicting packs — never degrade silently

Non-goals (this phase):

- In-game UI for pack management (future)
- Digital signatures / source verification
- Transitive dependency resolution

## 2. Current State & Constraints

Existing asset loading (`common/assets/src/fs.rs`):

- `FileSystem` implements two-layer sources: default (`assets/`) + override
  (`VELOREN_ASSETS_OVERRIDE` env)
- `read()`: override wins when present, otherwise falls back to default
  (**per-file override**)
- `read_dir()`: both directories are **merged** with file-level dedup
  (**directory appending** already works, well tested)
- Hot reloading: FsWatcher on both directories
- `userdata_dir()` (`common/base/src/userdata_dir.rs`): next to the executable /
  workspace directory
- voxygen config dir: `<userdata>/voxygen` (overridable via `VOXYGEN_CONFIG`),
  stores `settings.ron` / `profile.ron`
- Repository assets use Git LFS (`.gitattributes`)

Constraints:

- A single override layer cannot express "multiple packs stacked in order"
- Override is controlled only by an environment variable, no config file
- Asset id references (RON referencing item/creature ids) only fail at load time;
  there is no startup-time dangling-reference check

## 3. Core Concepts

- **Content Pack**: a standalone directory containing an asset tree plus a
  `pack.ron` manifest. When enabled, it merges with the core assets:
  same-path files override core, new files are appended.
- **Pack sources**:
  - Shipped with the game: `assets/packs/<pack_id>/` (e.g. `assets/packs/full/`)
  - Player-installed: `<userdata>/packs/<pack_id>/`
- **Enablement config**: `<config_dir>/packs.ron` — an ordered enablement list.
- **Compatibility**: `VELOREN_ASSETS_OVERRIDE` is kept, treated as a single
  highest-priority pack appended at the end (no manifest required).

## 4. Pack Format

```
pack_id/
├── pack.ron          # manifest (required)
└── <asset tree>      # same directory structure as assets/
```

`pack.ron`:

```ron
(
    id: "lemoncraft.full",
    name: "LemonCraft Full Content",
    version: "0.18.0",
    // Optional: pack ids this pack depends on (order validation; warning only this phase)
    // deps: ["lemoncraft.core"],
    // Optional: target game version (warning if mismatched)
    // game_compat: "0.18.0",
)
```

Rules:

- `id` is the canonical form of the directory name (lowercase, `.`/`_`/`-`).
  A directory whose name does not match its manifest `id` is rejected.
- A directory without `pack.ron` is not a pack (silently skipped when not
  enabled; fail-loud when enabled).
- Packs impose no directory conventions beyond `pack.ron` — the asset tree is
  a plain overlay layer.

## 5. Loading Mechanism Design

### 5.1 Source Composition (core `common/assets` change)

Refactor `FileSystem`:

```rust
pub struct FileSystem {
    default: RawFs,            // assets/ (core)
    packs: Vec<RawFs>,         // enabled packs; order = priority, later wins
}
```

- `read(id, ext)`: iterate `packs` in reverse (highest priority first); fall
  back to `default` only if no pack matches
- `read_dir(id, f)`: merge all sources (core first, lower-priority packs next),
  file-level dedup, higher priority wins
- `exists`: true if any source matches
- `configure_hot_reloading`: watch all source directories

Complexity: generalize the two-layer logic to N layers (`Option<RawFs>` →
`Vec<RawFs>`), ~30 lines, reusing the existing test scaffolding
(`FileSystem::scope` extended to multi-directory mocks).

### 5.2 Pack Resolution & Assembly (new module `common/assets/src/packs.rs`)

```rust
pub struct PackResolver {
    shipped: Vec<RawFs>,   // scanned assets/packs/<id>/, manifests parsed
    installed: Vec<RawFs>, // scanned <userdata>/packs/<id>/, manifests parsed
}

impl PackResolver {
    // Scan both source trees, parse each pack.ron, assemble Vec<RawFs>
    // according to the enablement list.
    pub fn resolve(config_dir: &Path) -> Result<Vec<RawFs>, PackError>;
}
```

Assembly rules:

1. Read the enablement list from `<config_dir>/packs.ron` (ordered `Vec<String>` of ids)
2. Scan shipped + installed pack dirs, build an id → path index; on id conflict,
   installed wins
3. Assemble in enablement-list order; non-enabled packs are not loaded
4. Enabled but missing → hard error (fail-loud), listing available pack ids
5. Manifest parse failure → hard error naming the pack path
6. No `packs.ron` → core only (default streamlined experience)

`packs.ron` format:

```ron
(
    // Order matters: later entries have higher priority
    enabled: ["lemoncraft.full"],
)
```

### 5.3 Startup Integration

- Extend `FileSystem::new()` signature: `new(config_dir: Option<&Path>)`
  (client and server entry points pass their own config dir)
- `VELOREN_ASSETS_OVERRIDE` compatibility: when set, appended to `packs` as the
  highest-priority entry (no manifest validation)
- Singleplayer server and client share the same asset layer (already the case;
  no new work)

## 6. Priority & Conflict Rules

| Scenario | Rule |
|---|---|
| Same-path file | Higher-priority pack overrides lower, packs override core |
| Directory append | Merged across sources, file-level dedup (generalized existing semantics) |
| Same id in both sources (shipped vs installed) | Installed wins (player installs override official distribution) |
| Dependency order | Manifest warning only this phase; no transitive resolution |
| Dangling asset references | Fails at load time (current behavior); startup-time check is phase 2 |

## 7. Integrity Guarantees

- Enabled-but-missing / broken manifest → startup failure with pack id and path
  (fail-loud, per engineering standards)
- Hot reloading: pack directory changes trigger asset reload (obtained for free
  from the generalized FsWatcher)
- Phase 2 (optional): `cargo pack-check` tool (mirroring `cargo img-export`),
  scanning pack RON references against core for dangling ids

## 8. Implementation Phases

| Phase | Scope | Verification |
|---|---|---|
| **P0** | Generalize `FileSystem` to N sources (Vec + merge semantics) | Generalize existing tests + new multi-pack tests |
| **P1** | `packs.rs` (scan/manifest/assembly) + `packs.ron` parsing | Unit tests for assembly rules; manual: core+full dual-pack run |
| **P2** | Official "full content" pack lands (`assets/packs/full/`) + core streamlining | Full build + dual-mode runtime comparison |
| **P3** | Documentation (player pack installation guide) | — |

## 9. Verification Plan

- Unit tests: multi-pack override/append/missing/broken-manifest/installed-wins
  cases (reusing the `FileSystem::scope` pattern)
- Integration: `cargo test -p lemoncraft-common-assets`
- Runtime:
  1. Default (no `packs.ron`): streamlined experience, no asset load errors
  2. `packs.ron` enabling the full pack: content regression (site/creature/recipe counts)
  3. Hot reload: editing pack assets hot-updates
- Full: `cargo check --workspace --locked` + the four CI checks

## 10. Relationship to the Mod Ecosystem

- A content pack is the release form of a mod: one directory + manifest,
  installed by copying or `git clone`
- No code SDK required: RON data + assets express items/recipes/creatures/sites/music
- Future evolution: in-game pack management UI → multi-pack dependency
  resolution → (only if real demand) a scripting layer
