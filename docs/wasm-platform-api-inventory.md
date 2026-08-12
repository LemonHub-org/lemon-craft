# WASM Platform API Inventory — P1 Result

> Status: recorded 2026-08-12, after Stage A (P1) landed
> Goal: catalogue every native-only platform API the browser build must not
> touch, and how each was handled at compile time. Runtime handling (storage
> worker, WebGPU bootstrap) is staged in P2/P5.

## 1. Threads and concurrency

| Location | API | P1 handling |
|---|---|---|
| `common/src/clock.rs` | `thread_priority` in `Clock::tick` | `#[cfg(not(target_arch = "wasm32"))]` |
| `common/src/slowjob.rs` | thread priority in slow-job pool | `#[cfg(not(target_arch = "wasm32"))]` block |
| `common/state/src/state.rs` | rayon pool `set_priority` closure | cfg-gated closure + spawn handler |
| `common/state/src/state.rs` | `rayon::ThreadPoolBuilder` | compiles on wasm (stub pool); real single-thread fallback is a P4 concern |
| `voxygen/src/main.rs` | `tokio::runtime::Builder::new_multi_thread()` | wasm branch uses `new_current_thread()` |
| `client/src/lib.rs` | `tokio::task::block_in_place` in `Drop` | `#[cfg(not(target_arch = "wasm32"))]` |
| `voxygen/src/menu/main/client_init.rs` | `tokio::task::block_in_place` (runtime drop) | cfg branch; wasm drops directly |
| `common/frontend/src/lib.rs` | `tracing_appender::non_blocking` flusher thread | wasm writes directly to the terminal writer (no guard thread) |

## 2. Sockets / DNS / TLS

| Location | API | P1 handling |
|---|---|---|
| `network/src/channel.rs` | `tokio::net::TcpStream`/`TcpListener`, socket2, TCP protocols | whole TCP transport behind `networking` feature |
| `network/src/api.rs` | `ConnectAddr::Tcp/Udp`, `ListenAddr::Tcp/Udp` | behind `networking` |
| `network/src/scheduler.rs` | TCP listen/connect dispatch | behind `networking` |
| `network/src/metrics.rs` | protocol name mapping | cfg per variant |
| `network/src/participant.rs` | `best_protocol` TCP/QUIC selection | cfg per branch |
| `client/src/addr.rs` | `tokio::net::lookup_host` resolution | behind `quic` feature |
| `client/src/lib.rs` | hickory resolver, quinn, rustls verifier, QUIC/TCP/SRV connect | behind `quic` feature |
| `client/src/error.rs` | `rustls::Error` variant | behind `quic` |
| `voxygen/src/menu/main/mod.rs` | `Error::RustlsErr` match arm | behind `client/quic` |
| `getrandom` | wasm JS backend | `--cfg getrandom_backend="wasm_js"` rustflags + `js`/`wasm_js` features |

## 3. Native integrations

| Location | API | P1 handling |
|---|---|---|
| `voxygen/src/session/mod.rs` | `mumble_link::SharedLink` | cfg `not(macos)` AND `not(wasm32)` (6 sites) |
| `voxygen/src/window.rs` | `winit::platform::wayland` window name | cfg excludes wasm32 |
| `voxygen/src/ui/ice/winit.rs` | `window_clipboard::Clipboard` | native field + connection; wasm no-op stub (browser adapter is a follow-up) |
| `voxygen/src/cmd.rs` | `open::that_detached` (wiki command) | cfg branch; wasm returns an error message |
| `voxygen/src/main.rs` | `winres` logo in `build.rs` | build.rs checks `CARGO_CFG_TARGET_ARCH != wasm32` (host cfg trap) |

## 4. Renderer

| Location | API | P1 handling |
|---|---|---|
| `voxygen/src/render/renderer/compiler.rs` | `shaderc::Compiler` | whole `ShaderCCompiler` behind `shaderc` feature; browser uses naga `WgpuCompiler` only |
| `voxygen/src/render/renderer/pipeline_creation.rs` | shaderc/naga compiler selection | `#[cfg(feature = "shaderc")]` branch; `#[cfg(not)]` forces naga |
| `voxygen/src/render/error.rs` | `shaderc::Error` variants | behind `shaderc` |
| `voxygen/src/render/renderer/compiler.rs` | `tokio::runtime::Runtime::new()` + `block_on(pop_error_scope)` | wasm skips the blocking error-scope poll |
| `voxygen/src/render/renderer/mod.rs` | `Instance::enumerate_adapters` (no wgpu_core on wasm) | cfg-gated; wasm goes straight to `request_adapter` |
| `voxygen/src/main.rs` | `enumerate_adapters` in `ListWgpuDevices` CLI | wasm prints "not supported" |

## 5. Audio

| Location | API | P1 handling |
|---|---|---|
| `voxygen/src/audio/soundcache.rs` | `kira::sound::streaming` (desktop-only in kira 0.12) | all streaming types/loaders behind `not(wasm32)`; wasm loads everything statically |
| `voxygen/src/audio/mod.rs` | `CpalBackend::pop_cpu_usage` (desktop backend only) | wasm returns 0.0 |

## 6. Persistence / filesystem (A6 — pending P5)

`common/base/src/userdata_dir.rs` uses `directories_next` + `std::env` +
`std::path`. It compiles on wasm but must not be used at runtime in the
browser. Callers to revisit when the async storage boundary lands (P5):

- `voxygen/src/main.rs` — logs dir, config dir from `userdata_dir()`
- `voxygen` settings load/save (`Settings::load(&config_dir)`)
- `server` persistence (rusqlite) — excluded from the browser graph entirely

These are compile-clean today because `directories_next`/`std::path` compile
on wasm; the P5 work replaces path-based writes with an awaitable
`UserStorage` boundary.
