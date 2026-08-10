# LemonCraft WASM Migration Guide

> Status: draft, 2026-08-10
> Companion: [`wasm-port-report.md`](wasm-port-report.md) (feasibility analysis)
> Goal: compile LemonCraft as a **singleplayer-only browser build** —
> no multiplayer, no server-cli, no native persistence.

---

## 0. Overview

| | |
|---|---|
| Target | `wasm32-unknown-unknown`, WebGPU backend, singleplayer only |
| Entry | `voxygen` as a wasm-bindgen app, served over HTTP |
| Assets | 415MB `assets/` served by a static host, loaded via a preloaded WebSource |
| Persistence | in-memory for the first port; IndexedDB later |
| Toolchain | `nightly-2026-06-13` (rust-toolchain), `wasm32-unknown-unknown` target, `wasm-pack` + `trunk` (or plain wasm-bindgen) |

The singleplayer path already uses **process-internal tokio Mpsc channels**
(no sockets). The migration is therefore mostly *compile-time gating* +
*contained runtime rework*, not a protocol rewrite.

---

## 1. Prerequisites

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack        # or: trunk
```

Nightly ships the wasm std library; no special toolchain needed.

---

## 2. Stage A — Compile-time gating (blockers first)

Goal: `cargo build --target wasm32-unknown-unknown` for each crate succeeds
(even if nothing runs yet).

### A1. network: drop quic/socket2 from the wasm build

`client/Cargo.toml`, `server/Cargo.toml`:

```toml
network = { ..., features = ["compression", "metrics"] }   # remove "quic"
```

`network/Cargo.toml`:

```toml
quic = ["dep:quinn"]   # keep the feature, but it must be OFF for wasm
```

Make `quinn` / `socket2` / `rustls` optional in `network`, `client`, `server`
(each currently declares them non-optional). The singleplayer path never
touches `ConnectAddr::Quic`/`ListenAddr::Quic`, so the runtime is unaffected.

### A2. hickory-resolver (DNS)

`client/Cargo.toml`: make `hickory-resolver` optional, gate behind a
`dns`/`networking` feature that the wasm build disables.

### A3. shaderc

`voxygen/Cargo.toml`:

```toml
shaderc = { version = "0.10", optional = true }
shaderc-from-source = ["shaderc/build-from-source", "dep:shaderc"]
```

Browser builds omit it; naga is already the default shader compiler
(`render/mod.rs` `enable_naga`).

### A4. rusqlite (server persistence)

`server/Cargo.toml`: `rusqlite` becomes optional behind `persistence`/
`sqlite` feature. The wasm build compiles `server` with persistence **off**
(see Stage E for the replacement).

### A5. window_clipboard

`voxygen/Cargo.toml`: gate `window_clipboard` behind a non-wasm feature.
Wasm uses `navigator.clipboard` (a new small module in `ui/ice`).

### A6. mumble-link / native-dialog / discord-sdk / steamworks

All become optional; wasm build disables them:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
mumble-link = ...
```

### A7. tokio single-thread, rayon fallback, std::thread

- `voxygen/src/main.rs` runtime: `Builder::new_current_thread()` under wasm.
- `rayon` (common, world, server, systems): add a `wasm` cfg that uses
  `rayon::iter::for_each` on single-thread (rayon compiles on wasm; the
  pool creation must be avoided — gate `ThreadPool`-dependent code or use
  the global pool which falls back to in-order execution).
- `std::thread::spawn` sites: `voxygen/src/singleplayer/mod.rs` (server
  thread), slowjob pools — see Stage F.

### A8. userdata / directories

`common/base/src/userdata_dir.rs`: add a wasm branch returning a
localStorage-backed virtual path (or a fixed in-memory dir). Keep the native
code untouched.

---

## 3. Stage B — Renderer: WebGPU

File: `voxygen/src/render/renderer/mod.rs` (`Renderer::new`, lines ~212-345).

```rust
#[cfg(target_arch = "wasm32")]
let backends = wgpu::Backends::BROWSER_WEBGPU;
#[cfg(not(target_arch = "wasm32"))]
let backends = wgpu::Backends::PRIMARY | wgpu::Backends::SECONDARY;
```

1. Add the `BROWSER_WEBGPU` backend selection (wasm only).
2. Replace `runtime.block_on(request_adapter/request_device)` with async
   awaits driven from the event loop (wasm main thread must not block).
   Create the renderer inside the winit `run` callback instead of before it.
3. Feature/limits: WebGPU does **not** support `PUSH_CONSTANTS` or
   `max_push_constant_size`; drop them under wasm:
   ```rust
   #[cfg(not(target_arch = "wasm32"))]
   { required_features.insert(wgpu::Features::PUSH_CONSTANTS); ... }
   ```
4. Shader compiler: ensure `WgpuCompiler` (naga) does not spin its own tokio
   runtime under wasm (`render/renderer/pipeline_creation.rs` / `compiler.rs`).

---

## 4. Stage C — Window & event loop

File: `voxygen/src/window.rs` (`Window::new`), `voxygen/src/run.rs`.

- winit web requires window creation **inside** the event-loop callback
  (async context). Move `Window::new` + `Renderer::new` into the `run`
  closure or a `spawn_local` bootstrap.
- `ControlFlow::Poll` maps to `requestAnimationFrame` on web — semantics
  differ (frame-driven); keep it, but verify input/timers.
- Gate exclusive fullscreen / `VideoModeHandle` / monitor APIs on
  `cfg(not(target_arch = "wasm32"))`.
- Cursor grab on web uses Pointer Lock (winit web supports it).

---

## 5. Stage D — Assets

Files: `common/assets/src/fs.rs`, `common/assets/src/lib.rs`
(`ASSETS` static init, `ASSETS_PATH`, canary check).

The `Source` trait is synchronous, so HTTP cannot be lazy:

**Option 1 (recommended first port): preloaded WebSource**

```rust
// common/assets/src/web.rs
pub struct WebSource { files: HashMap<String, Vec<u8>> }
impl Source for WebSource { /* read/read_dir/exists from the map */ }
```

At startup, fetch the asset manifest (`assets/manifest.json`) over HTTP,
then download all listed files into the map (or lazily fetch each
`read()` — but `read` is sync, so preload is simpler). 415MB may take a
while; a manifest-based subset (core assets first) is a better UX.

**Option 2: `assets_manager::source::Embedded`** (`embed!` macro) — assets
compiled into the wasm binary. Only practical for a curated subset
(fonts, UI textures); not the full 415MB.

Changes:
- `lib.rs`: `#[cfg(target_arch = "wasm32")]` branch for `ASSETS` using
  `WebSource`; skip the `ASSETS_PATH` search and canary check on wasm.
- `fs.rs`: gate `FileSystem` (and hot-reloading) behind non-wasm.
- Serve `assets/` with any static host (no CORS issue if same-origin).

---

## 6. Stage E — Persistence & config

Files: `server/src/persistence/` (rusqlite models), `voxygen` settings
(`settings.ron`, `profile.ron` via `userdata_dir`).

- First port: **in-memory persistence**. Gate `run_migrations` +
  `persistence` systems behind the `sqlite` feature; provide a no-op
  `PersistenceManager` for wasm singleplayer. Characters/saves live in
  memory for the session.
- Config: `settings.ron`/`profile.ron` read/write via a wasm branch of
  `userdata_dir` that stubs to defaults (or localStorage-backed serialization
  later).
- Later: IndexedDB adapter implementing the same persistence interface.

---

## 7. Stage F — Singleplayer in-process wiring

Files: `voxygen/src/singleplayer/mod.rs`, `voxygen/src/main.rs`,
`server/src/lib.rs` (`Server::new`), `client/src/lib.rs`.

1. **Settings**: `Settings::singleplayer` → `gameserver_protocols = vec![]`,
   `query_address = None` (no TCP/UDP sockets on wasm).
2. **Server thread → async task**:
   ```rust
   // before: std::thread::spawn(|| run_server(...))
   // after:  runtime.spawn(async move { run_server_async(...) })
   ```
   `run_server` (blocking Clock loop) becomes an async loop yielding
   periodically (e.g. `tokio::time::sleep` per tick).
3. **Shared runtime**: `main.rs` single-thread runtime is shared by client
   and server — already the case; keep it.
4. **Mpsc path**: the existing `ConnectionArgs::Mpsc` + in-process channels
   work without sockets. bincode serialization over an in-memory channel is
   cheap; do not rewrite to direct ECS calls in the first port.
5. Gate `server`'s network-only paths (`Protocol::Tcp/Quic`, query server)
   behind the `networking` feature.

---

## 8. Build & serve

Create a `wasm/` directory at the workspace root:

```
wasm/
├── index.html
├── wasm-pack.toml        # or trunk config
└── js/
    └── bootstrap.js      # wasm-bindgen init, fetch assets, start()
```

Build:

```sh
cargo build -p lemoncraft-voxygen --target wasm32-unknown-unknown \
  --no-default-features --features "wasm-singleplayer" \
  --profile release

wasm-bindgen --target web --out-dir wasm/pkg \
  target/wasm32-unknown-unknown/release/lemoncraft_voxygen.wasm
```

Recommended feature set for the wasm build (add to `voxygen/Cargo.toml`):

```toml
[features]
wasm-singleplayer = [
  "singleplayer",
  # deliberately excludes: egui-ui? (verify), shaderc, discord, hot-reloading
]
```

Serve with any static server (`python -m http.server`, nginx, GitHub Pages)
with correct `Content-Type` for `.wasm`.

---

## 9. Verification checklist per stage

| Stage | Check |
|---|---|
| A | `cargo build --target wasm32-unknown-unknown -p lemoncraft-common -p lemoncraft-world` succeeds; then voxygen compiles to a wasm artifact |
| B | Browser shows a rendered frame (clear color / UI), no adapter panic |
| C | Input (mouse/keyboard) works; window resize works |
| D | Title screen fonts/images load; no asset NotFoun d |
| E | Singleplayer world generates and saves/loads within the session |
| F | Full gameplay loop (spawn → walk → mine → combat) in-browser |

---

## 10. Risks & mitigations

| Risk | Mitigation |
|---|---|
| World generation slow on a single wasm thread | Async chunk scheduling; `no_overflow` profile; later: wasm threads (SharedArrayBuffer + COOP/COEP headers) |
| 415MB asset load time | Manifest-based subset loading; cache in IndexedDB; lazy fetch of world/map assets |
| WebGPU feature gaps (push constants) | Already gated in Stage B; fall back to uniform buffers |
| tokio/rayon single-thread regressions | Gate and profile; worldgen can run in chunks across ticks |
| winit web API drift | Pin winit 0.30 (already pinned); test on Chrome/Edge/Firefox |

---

## 11. Execution plan (recommended order)

1. **S1** — target add; build `common` + `world` for wasm (zero code change, validates the pure-Rust layer).
2. **S2** — Stage A gating; get `voxygen` to produce a wasm binary (link errors resolved).
3. **S3** — Stage B+C shell: winit web window + WebGPU clear-color ("hello triangle").
4. **S4** — Stage D: WebSource + title screen.
5. **S5** — Stage E+F: in-process singleplayer loop, persistence stubbed.

Each stage is independently verifiable and reversible (all gating is
cfg/feature-based; native builds remain untouched).
