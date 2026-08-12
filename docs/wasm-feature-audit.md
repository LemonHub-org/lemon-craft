# WASM Feature Graph Audit — P1 Result

> Status: recorded 2026-08-12, after Stage A (P1) landed
> Companion: [`wasm-migration-guide.md`](wasm-migration-guide.md) §2 (Stage A)
> Checkpoints: `cargo check --target wasm32-unknown-unknown` for
> `lemoncraft-common`, `lemoncraft-world`, and `lemoncraft-client` /
> `lemoncraft-voxygen` with `--no-default-features --features wasm-singleplayer`.

## 1. Feature graph (browser build)

The browser build is `wasm32-unknown-unknown` with
`--no-default-features --features wasm-singleplayer`. Enabled features:

| Crate | Feature | What it enables |
|---|---|---|
| `lemoncraft-voxygen` | `wasm-singleplayer` | `client/wasm-singleplayer`; no server, shaderc, egui, discord, native-dialog, multiplayer |
| `lemoncraft-client` | `wasm-singleplayer` | No `quic`, no `networking`, no `auth` |
| `lemoncraft-network` | (default-features off) | `compression` only; no `quic`, no `networking`, no `metrics` |
| `lemoncraft-common` | (default-features off) | None of `simd`/`bin_*` |

Everything else is excluded at the Cargo level; nothing relies on the linker
to drop unused network code.

## 2. New / reworked features

| Crate | Feature | Notes |
|---|---|---|
| `lemoncraft-voxygen` | `wasm-singleplayer` | owned by the browser target; forwards `client/wasm-singleplayer` |
| | `multiplayer` | `client/quic` + `client/networking` + `client/auth`; part of `default-publish` (native default unchanged) |
| `lemoncraft-client` | `wasm-singleplayer` | browser singleplayer |
| | `quic` | `network/quic` + quinn + rustls + hickory-resolver (was `default`); default |
| | `networking` | `network/networking` (TCP/UDP transports); default |
| | `auth` | `authc` (auth-server login, hyper/rustls chain); default |
| `lemoncraft-network` | `quic` | quinn + rustls, optional deps |
| | `networking` | socket2 + `tokio/net` (TCP/UDP transports) |
| | `wasm-singleplayer` | declared for symmetry; the browser graph simply does not enable `quic`/`networking` |
| `lemoncraft-server` | `networking` | forwards `network/networking` (server is not part of the P1 browser graph) |

## 3. Target-specific dependencies

| Dependency | Moved to | Used by |
|---|---|---|
| `thread-priority` | `cfg(not(target_arch = "wasm32"))` | common, common-state |
| `getrandom` (0.2/0.3/0.4) | `cfg(target_arch = "wasm32")` with `js`/`wasm_js` feature | common (feature unification) |
| `tokio` `rt-multi-thread` | `cfg(not(target_arch = "wasm32"))` on top of base `rt` | client, voxygen |
| `socket2` | optional behind `networking` | network |
| `rustls` | optional behind `quic` | network |
| `quinn` | optional behind `quic` | network |
| `hickory-resolver` | optional behind `quic` | client |
| `authc` | optional behind `auth` | client |
| `window_clipboard`, `open` | `cfg(not(target_arch = "wasm32"))` | voxygen |
| `mumble-link` | `cfg(all(not(macos), not(wasm32)))` | voxygen |
| `shaderc` | optional (implicit `shaderc` feature) | voxygen |
| `tracing-appender` | `cfg(not(target_arch = "wasm32"))` | common-frontend |

## 4. Environment / toolchain requirements

- `rustup target add wasm32-unknown-unknown` (official dist server; the local
  TUNA mirror lacks this nightly's component).
- `.cargo/config.toml`:
  `[target.wasm32-unknown-unknown] rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]`
  — required by getrandom 0.3+; the cargo feature alone is insufficient.
- Vendored `iced` fork (`vendor/iced`, patched in `[patch.'https://github.com/Imberflur/iced']`)
  with wasm32 fixes for the subscription `EventStream` (LocalBoxStream).

## 5. Verified exit conditions

- `cargo check --target wasm32-unknown-unknown -p lemoncraft-common` — pass
- `cargo check --target wasm32-unknown-unknown -p lemoncraft-world` — pass
- `cargo check --target wasm32-unknown-unknown -p lemoncraft-client --no-default-features --features wasm-singleplayer` — pass
- `cargo check --target wasm32-unknown-unknown -p lemoncraft-voxygen --no-default-features --features wasm-singleplayer` — pass
- No `quinn`/`socket2`/`mio`/`ring`/`shaderc`/`rusqlite`/`hickory-resolver`/
  `authc`/`window_clipboard`/`mumble-link`/`thread-priority` in the wasm graph
  (verified via `cargo tree --target wasm32-unknown-unknown -e features`).

## 6. Native verification

- `cargo check -p lemoncraft-common`, `-p lemoncraft-client`,
  `-p lemoncraft-network`, `-p lemoncraft-wasm-launcher` — pass.
- Native `lemoncraft-voxygen`/`lemoncraft-server` are blocked in the working
  tree by uncommitted feature work (server `sys/metrics.rs` syntax error and
  the in-flight ice UI widgets) — unrelated to the P1 changes; the P1 diff
  keeps the native feature graph identical (default features unchanged).
