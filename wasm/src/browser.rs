//! Browser-only bootstrap: mount the canvas and draw a version label.

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, window};

pub fn run() {
    let document = window()
        .expect("no window")
        .document()
        .expect("no document");

    let canvas = document
        .get_element_by_id("lemoncraft-canvas")
        .expect("missing #lemoncraft-canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("canvas is not an HtmlCanvasElement");

    let width = canvas.client_width() as u32;
    let height = canvas.client_height() as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")
        .expect("2d context unavailable")
        .expect("context is not 2d")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("not a CanvasRenderingContext2d");

    context.set_fill_style_str("#58c0ff");
    context.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

    context.set_font("16px monospace");
    context.set_fill_style_str("#ffffff");
    let version = env!("CARGO_PKG_VERSION");
    let _ = context.fill_text(&format!("LemonCraft v{version}"), 12.0, 24.0);
}
