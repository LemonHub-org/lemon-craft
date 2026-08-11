# LemonCraft WASM Browser Port — Technical Roadmap & TODO

> Status: working plan, 2026-08-11
> Based on: [`wasm-migration-guide.md`](wasm-migration-guide.md) (Stages A-G)
> Companion: [`wasm-port-report.md`](wasm-port-report.md) (feasibility)
> Scope: singleplayer-only browser build (WebGPU, OPFS persistence, no
> multiplayer/server-cli). Every phase must leave native builds unchanged.

## 1. Execution plan overview

```
P0 launcher ──> P1 portability gates ──> P2 WebGPU shell ──> P3 assets/title
   └──────────────────────────────────────────────────────────────┘
                                                                    │
P3 ──> P4 simulation ──> P5 storage ──> P6 hardening
```

Each phase has a concrete exit check: a browser artifact that can be exercised,
not just `cargo check`.

## 2. TODO checklist by phase

### P0 — Decisions and launcher (guide §9)

Bootstrap the browser target and a blank canvas.

- [ ] 001 Create `wasm/` crate skeleton: `wasm/index.html`, `wasm/Trunk.toml`,
      `wasm/src/lib.rs` (`crate-type = ["cdylib"]`), `wasm/Cargo.toml`
- [ ] 002 `rustup target add wasm32-unknown-unknown`; `cargo install trunk`
      (record exact trunk version in the launcher README)
- [ ] 003 Add `#[wasm_bindgen(start)] fn start()` that mounts a canvas with a
      version label; keep browser-only JS limited to bootstrap concerns
- [ ] 004 Wire the launcher crate into the workspace (`wasm/` member) without
      touching native member builds
- [ ] 005 Verify: `trunk serve wasm/index.html` shows the version canvas with
      no console errors

Exit: blank browser canvas with version label, reproducible via trunk.

### P1 — Portability gates (guide §2, Stage A)

Make common/world/client/voxygen compile for `wasm32-unknown-unknown` with a
clean feature graph.

- [ ] 101 A0: add `wasm-singleplayer` feature owned by the browser target;
      enable singleplayer + browser adapters, exclude sockets/CLI/sqlite/
      shaderc/integrations/hot-reload
- [ ] 102 A0: audit the feature graph with
      `cargo tree --target wasm32-unknown-unknown -e features` and
      `cargo metadata --format-version 1 --no-deps`; record results in
      `docs/wasm-feature-audit.md`
- [ ] 103 A1: make `quinn`, `socket2`, QUIC-only TLS optional; keep the `quic`
      feature but disable for WASM (`network/`)
- [ ] 104 A1: make `hickory-resolver` optional behind a native
      `dns`/`networking` feature
- [ ] 105 A1: gate server query/listen paths independently from the in-process
      MPSC connection path; browser build must not construct socket addresses
      from default settings
- [ ] 106 A2: move `mumble-link`, `native-dialog`, `discord-sdk`,
      `steamworks`, `window_clipboard` to
      `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`
- [ ] 107 A2: clipboard browser adapter (`navigator.clipboard` with
      secure-context/permission fallback or disabled state)
- [ ] 108 A3: make `shaderc` + `shaderc-from-source` native-only; verify naga
      compiler path needs no runtime/blocking file API
- [ ] 109 A4: make `rusqlite` + server persistence migrations optional behind a
      native `sqlite`/`persistence` feature; compile a small singleplayer
      server surface instead of the full server
- [ ] 110 A5: introduce project-local parallel-iteration abstraction
      (`cfg(not(wasm32))` → rayon prelude, `cfg(wasm32)` → sequential);
      inventory every `std::thread::spawn`, blocking channel wait, fs watcher,
      slow-job pool in `docs/wasm-platform-api-inventory.md`
- [ ] 111 A6: replace `userdata_dir`/path-based writes with an async storage
      boundary (load/save operations, not paths); native behavior unchanged
- [ ] 112 First compile checkpoints, in order (all `--locked`):
      `cargo check --target wasm32-unknown-unknown` for `lemoncraft-common`,
      `lemoncraft-world`, then `lemoncraft-client` and `lemoncraft-voxygen`
      with `--no-default-features --features wasm-singleplayer`

Exit: four crates compile for the target with no native socket, filesystem,
thread-pool, or integration dependency in the graph.

### P2 — WebGPU shell (guide §4, §5, Stages C-D)

Asynchronous renderer bootstrap plus a working winit web event loop.

- [ ] 201 Select `wgpu::Backends::BROWSER_WEBGPU` under `cfg(target_arch =
      "wasm32")`; confirm exact wgpu version/backend constants from
      `Cargo.lock`
- [ ] 202 Make `request_adapter`/`request_device` async (no `block_on` on the
      browser main thread); drive from the launcher state machine
- [ ] 203 Gate native-only features: `PUSH_CONSTANTS`,
      `max_push_constant_size` behind `cfg(not(target_arch = "wasm32"))`
- [ ] 204 Clear-color/triangle checkpoint before integrating the full renderer
      (`voxygen/src/render/renderer/mod.rs`)
- [ ] 205 Create the winit web canvas/window inside the event-loop bootstrap;
      renderer created only after the window exists (`voxygen/src/window.rs`,
      `voxygen/src/run.rs`)
- [ ] 206 Verify timers, input, focus, resize on the web loop; clamp frame `dt`
      after tab suspension (no one enormous catch-up step)
- [ ] 207 Gate exclusive fullscreen, monitor enumeration,
      `VideoModeHandle` on native cfg; cursor grab = optional Pointer Lock,
      released on focus loss
- [ ] 208 Unsupported-WebGPU error screen (feature detection + fatal state)

Exit: clear-color/triangle renders in browser; resize, focus, input work.

### P3 — Assets and title screen (guide §6, Stage E)

Staged async asset delivery; never fetch from a synchronous `Source::read`.

- [ ] 301 Manifest generator: `assets/manifest.json` with path, byte length,
      content hash, manifest version (packaging script, e.g. under
      `tools/` or the launcher crate)
- [ ] 302 Core manifest subset: fonts, UI, title background, shaders, minimal
      client data
- [ ] 303 Async preload of the core set with hash validation; build in-memory
      `WebSource` from completed bytes (`common/assets/src/fs.rs`,
      `common/assets/src/lib.rs`)
- [ ] 304 Initialize the global asset manager only after the source is ready;
      no async work in a static asset singleton during module init
- [ ] 305 Later phases for world/audio/optional assets; send completed groups
      to the Storage Worker for OPFS caching (cache key: manifest version +
      content hash)
- [ ] 306 Hosting docs: MIME types, cache headers, same-origin/CORS policy,
      never publish Git LFS pointer files
- [ ] 307 Title screen renders from the core set (no full 415MB preload)

Exit: title screen works from the core manifest without downloading the full
asset tree.

### P4 — Singleplayer in-process loop (guide §8, Stage G)

Bounded fixed-tick simulation in the browser.

- [ ] 401 Extract `SingleplayerRuntime { server, client, accumulator }` with a
      `tick(&mut self, frame_dt)` that clamps, accumulates, and runs a bounded
      number of fixed ticks (`voxygen/src/singleplayer/mod.rs`)
- [ ] 402 Disable `gameserver_protocols`, query addresses, socket setup for
      WASM; keep `ConnectionArgs::Mpsc` in-process serialization
- [ ] 403 Drive server/client ticks from the browser frame loop or bounded
      `spawn_local` scheduler; yield between expensive world-generation/chunk
      batches (no multi-second synchronous worldgen)
- [ ] 404 Use `tokio::sync` only where known to compile for the target;
      `spawn_local` for non-`Send` tasks; single-thread runtime on WASM
- [ ] 405 Keep the native server thread/clock loop unchanged behind native cfgs
- [ ] 406 Smoke test: spawn → walk → mine → combat → return-to-title; runs
      several minutes without panic/adapter error/unbounded memory

Exit: responsive fixed-tick loop; basic gameplay in browser; native server
untouched.

### P5 — Storage (guide §7, Stage F)

OPFS persistence through a dedicated Storage Worker.

- [ ] 501 Define async `UserStorage` trait (`load`, `save_atomic`, `remove`);
      native impl wraps sync fs calls; browser impl talks to the worker
- [ ] 502 Main-thread capability check + `navigator.storage.persist()` request;
      record grant result; surface quota/availability status on title screen
- [ ] 503 Dedicated Storage Worker as sole owner of OPFS handles; typed
      commands `Load`/`SaveAtomic`/`Remove`/`List`/`Flush` via
      `wasm-bindgen` adapter; `createSyncAccessHandle()` worker-only for hot
      files (always flush + close)
- [ ] 504 Versioned OPFS layout `/lemoncraft/v1/` (schema, settings, profiles,
      worlds, saves, assets, journal, tmp); writes: payload to `tmp/`,
      validate, publish committed generation manifest
- [ ] 505 SQLite migrations/persistence systems disabled for WASM; settings,
      profiles, world/character saves on OPFS
- [ ] 506 Memory-only mode as an explicit non-persistent fallback, clearly
      marked in the UI
- [ ] 507 Session save/load test: reload within session never touches native
      paths

Exit: save/load round-trips through OPFS; quota errors visible; explicit
memory-only state.

### P6 — Hardening (guide §10, §11)

Browser matrix, profiling, reproducible hosting.

- [ ] 601 `wasm-bindgen-test` unit tests for pure browser adapters
      (clipboard, OPFS adapter, storage commands)
- [ ] 602 Headless Chrome smoke tests: launcher, input, resize, title screen
- [ ] 603 Manual WebGPU checks on Chrome/Edge + documented Firefox result
- [ ] 604 Memory and frame-time sampling during world generation; artifact-size
      report for `trunk build --release`
- [ ] 605 Error screens: asset download failure, unsupported WebGPU, rejected
      device, malformed manifest
- [ ] 606 Hosting runbook: HTTPS, stable origin (OPFS is origin-private),
      documented headers; reproducible static output

Exit: reproducible release artifact with documented hosting; full guide §10
verification table satisfied.

## 3. Cross-phase dependencies

| Task | Blocked by |
|---|---|
| P2 shell | P0 launcher, P1 compile gates (A2 clipboard optional) |
| P3 title | P2 WebGPU shell |
| P4 simulation | P2 (frame loop), P3 (assets for in-game content) |
| P5 storage | P1 A6 (storage boundary), P3 305 (asset caching) |
| P6 hardening | P2-P5 complete |
| WASM CI job (later) | P6 release pipeline |

## 4. Recurring verification (every phase)

- [ ] `cargo clippy` per guide §10 table and `.gitlab/scripts/code-quality.sh`
      order still passes for native builds
- [ ] Native singleplayer/server unchanged (`cargo run --bin
      lemoncraft-voxygen` unaffected)
- [ ] `cargo tree --target wasm32-unknown-unknown -e features` shows no
      unexpected native dependency
- [ ] Browser console clean in the exercised artifact

## 5. Risks carried forward (guide §11 summary)

- World generation freezes the tab → fixed-tick batches + cooperative yield
- 415MB assets → core manifest + staged loading + hash validation
- WebGPU feature/driver gaps → detection + fatal error screen
- OPFS quota/partial writes → worker ownership + generation manifests
- Tab suspension jumps → clamp `dt`, cap catch-up work
