fn main() {
    #[cfg(windows)]
    // `cfg(windows)` refers to the host when cross-compiling, so check the
    // actual target explicitly: the browser build has no native .exe icon.
    if std::env::var("CARGO_CFG_TARGET_ARCH").is_ok_and(|arch| arch != "wasm32") {
        //Set executable logo with winres here:
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/voxygen/logo.ico");
        res.compile().expect("failed to build executable logo.");
    }
}
