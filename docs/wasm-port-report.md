# WASM Port Feasibility Report — Browser (Singleplayer-only)

> Status: pre-research, 2026-08-10
> Scope: compile LemonCraft as a singleplayer-only browser build
> (multiplayer dropped: no QUIC/WebTransport, no server-cli)

## 1. Conclusion

**Feasible.** The singleplayer architecture is already ~80% process-internal
(tokio Mpsc channels, shared runtime, no socket I/O in the singleplayer
path), and the render stack (wgpu 27 / winit 0.30) officially supports
`wasm32-unknown-unknown`. The work splits into:

- **Compile-time gating** (quinn/rustls/hickory/shaderc/rusqlite/window_clipboard…): mostly mechanical
- **Runtime rework** (async renderer init, single-thread tokio, asset WebSource, persistence backend): contained
- **No** network-protocol rewrite: `common-net` (serde/bincode) is quinn-independent

Estimated effort: weeks to ~2 months for a playable singleplayer browser
build, assuming a focused team.

## 2. Singleplayer architecture (good news)

| Fact | Evidence |
|---|---|
| Singleplayer client↔server uses **Mpsc channels** (no socket) | `client/src/addr.rs` `ConnectionArgs::Mpsc`; `network/src/channel.rs` `with_mpsc_*`; `voxygen/src/singleplayer/mod.rs` |
| Server runs as **in-process std::thread** on the shared tokio runtime | `singleplayer/mod.rs:137-170`; `main.rs:126-137` |
| `Server::new` network listens can be **emptied** for singleplayer | `settings/mod.rs:291-315` (`gameserver_protocols`, `query_address`) |
| Protocol layer (`common-net`) is **independent of quinn** | pure serde/bincode; WebTransport could reuse it as-is |
| `network-protocol` is trait-based (`tcp.rs`/`quic.rs`/`mpsc.rs`) | a future `wt.rs` slots in cleanly |

## 3. Compile blockers (wasm32-unknown-unknown)

| Dependency | Why | Fix |
|---|---|---|
| `quinn` + `socket2` + `rustls` | QUIC/UDP, no wasm | feature-gate off for browser build (singleplayer path never calls it) |
| `hickory-resolver` | DNS | feature-gate off |
| `shaderc 0.10` | C++ SPIR-V tools | make optional; wasm forces the naga path (already default) |
| `rusqlite` (bundled) | C library, server persistence | replace with in-memory/IndexedDB backend for browser singleplayer |
| `window_clipboard 0.5` | no wasm backend | replace with `navigator.clipboard` |
| `mumble-link` | native VoIP | `cfg`-exclude on wasm |
| `tokio rt-multi-thread` | `main.rs` + client/server | single-thread runtime on wasm |
| `std::thread` (singleplayer server, slowjobs) | no threads on wasm | `tokio::spawn` / async loop |
| `rayon` | thread pools | single-thread fallback (feature or cfg) |
| `directories-next` / `userdata_dir` | filesystem | localStorage-backed config on wasm |
| `native-dialog`, `discord-sdk`, `steamworks` | desktop-only | feature-gate off |

## 4. Runtime rework (high value, contained)

1. **Renderer init** (`voxygen/src/render/renderer/mod.rs:212-345`):
   - add `wgpu::Backends::BROWSER_WEBGPU` under `cfg(target_arch="wasm32")`
   - `request_adapter`/`request_device` must become async (no `block_on` on main thread)
   - drop `PUSH_CONSTANTS` + `max_push_constant_size` for WebGPU (unsupported)
2. **Window/event loop** (`window.rs:263-299`, `run.rs:35`): create window/renderer inside the event-loop callback; web uses `requestAnimationFrame` semantics
3. **Assets** (`common/assets/src/fs.rs`, `lib.rs:284-347`): replace `FileSystem` static init + `ASSETS_PATH`/canary with a preloaded `Source` (assets_manager 0.13 supports custom Sources and `Embedded`); serve assets over HTTP + cache
4. **Persistence**: `server/src/persistence/` (rusqlite) → memory/IndexedDB for the browser build
5. **Audio**: kira/cpal have wasm32 support (verify backend wiring)
6. **Entry point**: `main.rs` → `#[wasm_bindgen(start)]` async bootstrap

## 5. Recommended spike plan

| Step | Goal | Verify |
|---|---|---|
| **S1** | `rustup target add wasm32-unknown-unknown`; `cargo build --target wasm32-unknown-unknown -p lemoncraft-common -p lemoncraft-world` | common/world compile clean (pure Rust) |
| **S2** | Gate quinn/shaderc/rusqlite/window_clipboard/mumble behind features; build `lemoncraft-voxygen --target wasm32-unknown-unknown` with them off | voxygen reaches "wasm linker needs entry" stage |
| **S3** | Minimal browser shell: wasm-bindgen start + winit web window + `BROWSER_WEBGPU` adapter + clear-color render | "hello triangle" in browser |
| **S4** | Preloaded WebSource for assets; load title screen + fonts | title screen renders in browser |
| **S5** | Singleplayer in-process wiring (Mpsc→direct calls), single-thread runtime, async server loop | world generation + gameplay loop in browser |

S1-S3 de-risk the hard unknowns (compile gating + WebGPU init). S4/S5 are
feature work once the shell stands.

## 6. Risks

- **World generation on a single wasm thread** is the main runtime risk
  (currently CPU-heavy); mitigation: async chunk scheduling + `no_overflow`
  profile semantics.
- **Asset size** (415MB) needs an HTTP static host + lazy loading strategy.
- Mpsc/bincode serialization between client and in-process server is cheap
  enough; skip a full "direct ECS call" rewrite unless profiling demands it.
