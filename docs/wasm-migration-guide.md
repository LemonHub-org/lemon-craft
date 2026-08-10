# LemonCraft WASM Migration Guide

> Status: revised draft, 2026-08-10
> Companion: [`wasm-port-report.md`](wasm-port-report.md)
> Goal: compile LemonCraft as a **singleplayer-only browser build** with WebGPU.
> Non-goals for the first port: multiplayer, server CLI, native OS filesystem
> persistence, browser extensions, and Web Worker simulation parallelism. Browser-side
> persistence uses OPFS from the first persistence milestone.

## 0. Scope and architectural decisions

The browser build is a separate product target. Hiding multiplayer controls is
not sufficient: the WASM feature graph must exclude sockets, QUIC, server CLI,
native persistence, and platform integrations at compile time.

The first port uses these decisions:

| Area | First-port decision |
|---|---|
| Target | `wasm32-unknown-unknown`, WebGPU only, with a clear unsupported-browser screen |
| Entry | A dedicated `wasm` launcher crate or `voxygen` library target with `#[wasm_bindgen(start)]`; do not treat the native binary as the browser entry |
| Build tool | Trunk is the recommended orchestrator; use standalone `wasm-bindgen` only if Trunk cannot express the required bootstrap |
| Runtime | Browser main thread plus `spawn_local`; no `block_on`, native thread pool, or assumption that `tokio::spawn` accepts non-`Send` tasks |
| Assets | Asynchronous manifest/bootstrap followed by a synchronous in-memory source; never fetch from a synchronous `Source::read` call |
| Persistence | OPFS is the only browser persistence backend; memory-only mode is used only when OPFS is unavailable |
| Parallelism | Simulation remains sequential first; one dedicated Storage Worker is required for OPFS I/O |
| Multiplayer | Kept as dormant native code where practical, but excluded from the `wasm-singleplayer` feature graph |

The existing singleplayer path uses process-internal MPSC channels rather than
network sockets. Keep that boundary for the first port, but do not assume that
the current server clock loop can simply be moved into an async task.

### First-port acceptance criteria

The first playable milestone is complete only when a browser can:

1. load the title screen and core fonts/UI assets;
2. create or enter one in-memory singleplayer world;
3. render a frame through WebGPU and accept mouse/keyboard input;
4. run the loop for at least several minutes without a panic, adapter error, or
   unbounded memory growth; and
5. return to the title screen without attempting a socket, filesystem, or
   native-platform operation.

## 1. Prerequisites and dependency inventory

Install the target and one build orchestrator:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

If Trunk is not used, install a pinned `wasm-bindgen-cli` version matching the
workspace and document that alternative separately. Do not maintain two
independent browser build pipelines during the first port.

Before changing dependencies, record the actual feature graph:

```sh
cargo tree --target wasm32-unknown-unknown -e features
cargo metadata --format-version 1 --no-deps > target/cargo-metadata.json
```

The inventory must include `std::fs`, `std::thread`, `std::process`, native
directories, sockets, DNS, clipboard, platform window APIs, `Instant`/timer
usage, and build scripts—not only direct Cargo dependencies.

## 2. Stage A — compile-time portability gates

Goal: make the pure Rust layer and then the browser feature graph compile
without linking any native-only capability.

### A0. Define the feature boundary first

Add a `wasm-singleplayer` feature owned by the browser target. It should enable
singleplayer code and browser adapters while excluding:

- QUIC/TCP/socket/query-server paths;
- server CLI and native SQLite persistence;
- shaderc build-from-source;
- Discord, Steam, Mumble, native dialogs, and native clipboard;
- hot-reloading and native filesystem asset sources.

The feature graph should be checked with `cargo tree`; a dependency being
optional in `Cargo.toml` is not enough if another feature re-enables it.

### A1. Network and DNS

Keep the `quic` feature in `network`, but disable it for WASM. Make QUIC,
`socket2`, and any QUIC-only TLS dependencies optional or target-specific.
Make `hickory-resolver` optional behind a native `dns`/`networking` feature.

Gate server query and listen paths independently from the in-process MPSC
connection path. The browser build must not construct a TCP/UDP address merely
because a default server address exists in settings.

### A2. Native integrations

Use target-specific dependencies where possible:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
mumble-link = ...
native-dialog = ...
discord-sdk = ...
steamworks = ...
window_clipboard = ...
```

The browser adapter for clipboard must account for secure-context and
permission failures and provide a visible fallback or a disabled state.

### A3. Shader compilation

Make `shaderc` and `shaderc-from-source` native-only. Confirm that the browser
path uses the existing naga compiler and that shader/pipeline creation does not
create a tokio runtime or call a blocking file API.

### A4. Persistence dependencies

Make `rusqlite` and server persistence migrations optional behind a native
`sqlite`/`persistence` feature. Prefer compiling a small singleplayer server
surface instead of compiling the entire server feature set and hoping unused
network code is removed by the linker.

### A5. Concurrency and blocking work

Do not assume rayon automatically becomes a correct single-thread fallback.
Introduce a small project-local parallel iteration abstraction, for example:

```rust
#[cfg(not(target_arch = "wasm32"))]
pub use rayon::prelude::*;

#[cfg(target_arch = "wasm32")]
pub use sequential_iter::*;
```

The exact abstraction should cover only operations LemonCraft actually uses.
Avoid creating a rayon global pool on WASM. Inventory every
`std::thread::spawn`, blocking channel wait, filesystem watcher, and slow-job
pool; each must become either a synchronous bounded step or a `spawn_local`
task.

### A6. Configuration and platform paths

Do not return a fake filesystem path from `userdata_dir` and then let callers
write to it. Introduce an asynchronous storage boundary so settings/profile
code asks for load/save operations rather than paths. Native filesystem
behavior remains unchanged; the browser adapter talks to a dedicated Storage
Worker over an RPC/message boundary and may fall back to memory-only mode when
the origin-private filesystem is unavailable.

### A7. First compile checkpoints

Run these in order:

```sh
cargo check --target wasm32-unknown-unknown -p lemoncraft-common --locked
cargo check --target wasm32-unknown-unknown -p lemoncraft-world --locked
cargo check --target wasm32-unknown-unknown -p lemoncraft-client --locked \
  --no-default-features --features wasm-singleplayer
cargo check --target wasm32-unknown-unknown -p lemoncraft-voxygen --locked \
  --no-default-features --features wasm-singleplayer
```

Use `cargo build` only after `cargo check` has a clean feature graph; `check`
alone does not validate the final browser artifact or bindgen exports.

## 3. Stage B — browser launcher and asynchronous bootstrap

The current `voxygen` native binary should not be passed directly to
`wasm-bindgen` as the long-term architecture. Add one of:

- a dedicated `wasm/` launcher crate with `crate-type = ["cdylib"]`; or
- a library target in `voxygen` exposing a browser-only start function.

The launcher owns this state machine:

```text
Loading manifest/assets
        ↓
Creating winit canvas and requesting WebGPU adapter/device
        ↓
Constructing Iced/UI and renderer
        ↓
Running frame loop
        ↓
Fatal browser error screen
```

Use `wasm-bindgen-futures::spawn_local` or the chosen browser executor for
bootstrap. Do not call `runtime.block_on` on the browser main thread.

The bootstrap must expose a useful error screen for failed asset downloads,
unsupported WebGPU, rejected device requests, and malformed manifests.

## 4. Stage C — renderer: WebGPU

File: `voxygen/src/render/renderer/mod.rs` and pipeline creation modules.

1. Select the browser backend only under `cfg(target_arch = "wasm32")` and
   verify the exact `wgpu` version/API in `Cargo.lock`; do not assume a backend
   constant exists without compiling the target.
2. Request the adapter/device asynchronously as part of the launcher state
   machine.
3. Gate native-only features such as push constants and
   `max_push_constant_size`:

   ```rust
   #[cfg(not(target_arch = "wasm32"))]
   {
       required_features.insert(wgpu::Features::PUSH_CONSTANTS);
       // native-only limits
   }
   ```

4. Add a minimal clear-color/triangle checkpoint before integrating the full
   renderer. This isolates browser adapter problems from asset and ECS bugs.
5. Add a graceful unsupported-WebGPU page. WebGPU support and feature limits
   vary by browser, driver, and privacy policy.

## 5. Stage D — window and event loop

Files: `voxygen/src/window.rs`, `voxygen/src/run.rs`.

- Create the web canvas/window inside the winit web event-loop bootstrap.
- Keep renderer creation after the window exists; do not construct either from
  a synchronous native-style `main` before the browser loop starts.
- Use `requestAnimationFrame` semantics through the winit web loop and verify
  that timers, input, focus, and resize events remain correct.
- Gate exclusive fullscreen, monitor enumeration, `VideoModeHandle`, and other
  native display APIs on `cfg(not(target_arch = "wasm32"))`.
- Treat cursor grab as optional Pointer Lock. Losing focus must release it.
- Clamp frame `dt` after tab suspension so the simulation does not attempt one
  enormous catch-up step when the tab becomes visible again.

## 6. Stage E — assets and browser delivery

Files: `common/assets/src/fs.rs`, `common/assets/src/lib.rs`, and a new browser
asset bootstrap module.

The existing `Source` trait is synchronous. A synchronous `read()` cannot
perform an HTTP fetch, so a browser source must be installed only after an
async preload phase.

### Recommended first implementation

1. Generate `assets/manifest.json` during packaging. Include path, byte length,
   content hash, and a manifest version.
2. Load a small core manifest first: fonts, UI, title background, shaders, and
   minimal client data.
3. Download the core set asynchronously, validate hashes, and construct an
   in-memory `WebSource` from the completed bytes.
4. Initialize the global asset manager only after that source is ready. Do not
   make a static asset singleton perform asynchronous work during module init.
5. Add later phases for world, audio, and optional assets. Send completed
   groups to the Storage Worker, which caches them in an OPFS directory keyed
   by manifest version and content hash.

Do not preload the full 415MB asset tree for the title screen. It creates a
poor first-load experience and can multiply memory usage through network,
Rust, decompression, and GPU copies. The static host must also serve the
manifest and assets with correct MIME types, cache headers, and same-origin or
explicit CORS policy. Git LFS pointer files must never be published as runtime
assets.

`Embedded` remains useful for a tiny emergency shell, but embedding the full
asset tree in the WASM binary is not a viable first-port strategy.

## 7. Stage F — OPFS persistence and configuration

Files: `server/src/persistence/`, `voxygen` settings/profile code, a new
storage abstraction, and a browser Storage Worker implementation.

Define a narrow asynchronous storage interface before adding browser-specific
code. Native implementations may wrap synchronous filesystem calls, but the
interface must be awaitable so OPFS does not get hidden behind fake paths:

```rust
trait UserStorage {
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    async fn save_atomic(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError>;
    async fn remove(&self, key: &str) -> Result<(), StorageError>;
}
```

First port:

- settings/profile and world/character saves use OPFS;
- memory-only mode is an explicit non-persistent fallback, not another storage
  backend;
- SQLite migrations and persistence systems: disabled for WASM.

### OPFS worker architecture

Use one dedicated Storage Worker as the sole owner of OPFS file handles:

```text
UI / renderer / simulation
          │ async commands and responses
          ▼
Dedicated Storage Worker
  ├── navigator.storage.getDirectory()
  ├── metadata and save files
  ├── asset cache files
  └── FileSystemSyncAccessHandle for hot files
```

The main thread performs the capability check and requests durable storage;
the worker performs the actual file operations. The main thread must never
perform large synchronous file work. `createSyncAccessHandle()` is a worker
only optimization, not the general OPFS API.

### OPFS layout and write rules

Use `navigator.storage.getDirectory()` through a small `wasm-bindgen` adapter
inside the Storage Worker. The main thread communicates with the worker through
typed commands such as `Load`, `SaveAtomic`, `Remove`, `List`, and `Flush`.
Keep the layout versioned and separate temporary data from committed data:

```text
/lemoncraft/v1/
├── schema.json
├── settings.ron
├── profiles/
├── worlds/<world-id>/
├── saves/<save-id>/
├── assets/<manifest-version>/<content-hash>/
├── journal/
└── tmp/
```

- Serialize all writes through the Storage Worker.
- Write payloads to `tmp/` first, validate the byte length/hash, then publish a
  small committed generation manifest pointing at the new data.
- Never treat a partially written file as a valid save after a tab crash.
- Keep schema version and migration metadata in the committed manifest.
- Use asynchronous directory/file-handle setup in the worker, then use
  `FileSystemSyncAccessHandle` only for high-frequency files that benefit from
  in-place access. Open handles are exclusive and must always be flushed and
  closed by the worker.
- Keep metadata and low-frequency settings on the worker's asynchronous file
  API; do not force every small write through a synchronous handle.
- Check `navigator.storage.estimate()` and write failures explicitly. Surface a
  “storage unavailable” state instead of silently reporting a successful save.
- Keep the memory-only fallback clearly marked in the UI so users know that
  progress will not survive a reload.

OPFS is origin-private storage, so production hosting must use a stable origin
and HTTPS. During main-thread bootstrap, call `navigator.storage.persist()`
when appropriate and record whether the browser granted durable storage; the
request can be denied and must not be treated as guaranteed. Expose OPFS
availability, quota, and persistence status to the title screen before
starting world generation.

## 8. Stage G — singleplayer in-process loop

Files: `voxygen/src/singleplayer/mod.rs`, `voxygen/src/main.rs`,
`server/src/lib.rs`, and `client/src/lib.rs`.

The first implementation should extract a bounded simulation tick instead of
rewriting the whole server loop as async:

```rust
struct SingleplayerRuntime {
    server: Server,
    client: Client,
    accumulator: Duration,
}

impl SingleplayerRuntime {
    fn tick(&mut self, frame_dt: Duration) {
        // clamp frame_dt, accumulate, and run a bounded number of fixed ticks
    }
}
```

Rules:

1. Disable `gameserver_protocols`, query addresses, and socket setup for WASM.
2. Keep `ConnectionArgs::Mpsc` and the existing in-process serialization path
   for the first port; do not replace it with direct ECS calls yet.
3. Drive server/client ticks from the browser frame loop or a bounded
   `spawn_local` scheduler.
4. Use `tokio::sync` only where it is known to compile for the target. Use
   `spawn_local` for non-`Send` browser tasks; do not assume the native
   shared-runtime model transfers unchanged.
5. Yield between expensive world-generation/chunk batches. A synchronous
   world-generation call that runs for seconds will freeze the browser tab.
6. Keep the native server thread and clock loop unchanged behind native cfgs
   until the browser tick path is independently verified.

## 9. Browser project and build

Create a browser launcher directory or crate with an explicit Trunk entry:

```text
wasm/
├── index.html
├── Trunk.toml
└── src/
    └── lib.rs
```

The launcher should export a small `start()` function and keep browser-only
JavaScript limited to canvas/bootstrap concerns.

Suggested development commands:

```sh
cargo check --target wasm32-unknown-unknown -p lemoncraft-voxygen --locked \
  --no-default-features --features wasm-singleplayer
trunk serve wasm/index.html
```

Suggested release command:

```sh
trunk build wasm/index.html --release
```

If standalone bindgen is selected instead, pin the CLI version, add the
`cdylib` target explicitly, and document the exact `wasm-bindgen` command next
to the launcher. Do not mix an unpinned `wasm-pack` command with a Trunk
pipeline.

## 10. Verification and exit criteria

| Stage | Required verification | Exit condition |
|---|---|---|
| A | `cargo tree` feature audit; target `cargo check` for common/world/client/voxygen | No native socket, filesystem, thread-pool, or integration dependency enters the WASM graph unexpectedly |
| B | Browser launcher loads and reports bootstrap errors | `start()` runs without a synchronous block or panic |
| C | Clear-color/triangle plus resize/focus/input smoke test | WebGPU adapter/device and winit loop work in a browser |
| D | Core manifest load, hash check, font/UI render | Title screen works without downloading the full asset tree |
| E | In-memory settings and session save/load test | Reload within the session does not access native paths |
| F | Spawn → walk → mine → combat → return-to-title smoke test | Fixed-tick singleplayer loop remains responsive for several minutes |
| Release | `trunk build --release`, browser console check, artifact-size report | Reproducible static output with documented hosting headers |

Add browser tests where feasible:

- `wasm-bindgen-test` for pure browser adapters;
- headless Chrome smoke tests for launcher, input, resize, and title screen;
- manual WebGPU checks on Chrome/Edge and an explicitly documented Firefox
  result, rather than assuming identical browser support;
- memory and frame-time sampling during world generation.

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| World generation blocks the tab | Fixed-tick batches, cooperative yielding, and a progress state |
| 415MB assets cause slow startup or memory pressure | Core manifest, staged loading, hash validation, browser cache, and an asset budget |
| WebGPU feature/driver gaps | Feature detection, native-only feature gates, and a fatal error screen |
| Native runtime assumptions leak into WASM | Target feature graph plus source-level platform API inventory |
| OPFS availability/quota or partial writes | Main-thread capability check, Storage Worker ownership, generation manifests, quota handling, and an explicit memory-only state |
| Single-thread regressions | Sequential parallel abstraction and deterministic tests before Web Workers |
| Tab suspension causes simulation jumps | Clamp `dt` and cap fixed-step catch-up work |
| Browser API permissions/CORS fail | Same-origin dev server, HTTPS production hosting, explicit headers, and visible fallback states |

## 12. Execution plan

1. **P0 — decisions and launcher:** choose Trunk, add the WASM launcher, and
   produce a blank browser canvas with a version label.
2. **P1 — portability inventory:** add `wasm-singleplayer`, audit the feature
   graph and platform APIs, and make common/world/client compile.
3. **P2 — WebGPU shell:** implement asynchronous adapter/device bootstrap,
   clear-color/triangle rendering, resize, focus, and input.
4. **P3 — assets/title:** generate the manifest, implement the async
   `WebSource` bootstrap, load the core subset, and render the title UI.
5. **P4 — simulation:** extract the bounded singleplayer tick path and prove
   world generation plus basic gameplay without native threads or sockets.
6. **P5 — storage:** add the Storage Worker, OPFS adapter, schema/version
   manifests, generation writes, quota errors, durable-storage request, and the
   explicit memory-only fallback.
7. **P6 — hardening:** browser matrix, artifact-size report, memory/frame-time
   profiling, error screens, and reproducible static hosting.

Each phase must leave native builds unchanged and have a concrete exit check.
Do not start the next phase when the previous one only passes `cargo check` but
has not produced or exercised the corresponding browser artifact.

## 13. Browser API references

- [MDN: Origin Private File System](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system)
- [MDN: FileSystemSyncAccessHandle](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemSyncAccessHandle)
- [MDN: StorageManager.persist()](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/persist)
- [web.dev: The origin private file system](https://web.dev/articles/origin-private-file-system?hl=en)
- [Chrome for Developers: SQLite Wasm backed by OPFS](https://developer.chrome.com/blog/sqlite-wasm-in-the-browser-backed-by-the-origin-private-file-system?hl=en)
