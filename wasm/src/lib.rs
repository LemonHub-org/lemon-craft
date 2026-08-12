//! Browser launcher for the LemonCraft singleplayer WASM port.
//!
//! P0: mount a canvas with a version label so the trunk pipeline is
//! reproducible end to end. Browser-only code lives behind
//! `cfg(target_arch = "wasm32")`; native builds of this crate are empty.

#[cfg(target_arch = "wasm32")] mod browser;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() { browser::run(); }

#[cfg(not(target_arch = "wasm32"))]
pub fn start() {}
