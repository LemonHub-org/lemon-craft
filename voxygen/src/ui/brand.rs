//! LemonCraft product shell strings and external-link policy.
//!
//! Do **not** point any URL at Veloren properties. Leave `Option` URLs as
//! `None` until a first-party site exists (design D1).

pub const PRODUCT_NAME: &str = "LemonCraft";
pub const WINDOW_TITLE: &str = "LemonCraft";
/// Wayland reverse-DNS application id.
pub const WAYLAND_APP_ID: &str = "net.lemoncraft.lemoncraft";
pub const WAYLAND_APP_TITLE: &str = "lemoncraft";

/// Issue tracker URL for crash dialogs. `None` → no external link.
pub const ISSUE_TRACKER_URL: Option<&str> = None;
/// Wiki home URL for `/wiki`. `None` → localized not-configured message.
pub const WIKI_HOME_URL: Option<&str> = None;
/// Wiki search URL template; `{query}` is replaced with `+`-joined terms.
pub const WIKI_SEARCH_URL_TEMPLATE: Option<&str> = None;
/// Community Discord invite. `None` → omit from panic text.
pub const DISCORD_URL: Option<&str> = None;

/// Mumble positional-audio identity (plugin / link name).
pub const MUMBLE_PLUGIN_NAME: &str = "lemoncraft";
pub const MUMBLE_PLUGIN_DESCRIPTION: &str = "lemoncraft-voxygen";

/// Format `LemonCraft <version>` for UI chrome.
pub fn version_line(display_version: impl AsRef<str>) -> String {
    format!("{PRODUCT_NAME} {}", display_version.as_ref())
}
