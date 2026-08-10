//! Lemon Fresh theme tokens (palette **v2** — light theme).
//!
//! Single source for brand/chrome colors shared by Conrod HUD and Iced menus.
//! Semantic combat/quality/chat colors stay in `hud` (frozen). See
//! `docs/visual-design-lemon-fresh.md`.
//!
//! v1 (Warm Craft Fantasy, dark brown surfaces) is **rejected** — do not
//! restore.

/// sRGB 0..=1 channel storage (not premultiplied). Palette **v2** (Lemon
/// Fresh).
pub mod brand {
    /// Warm cream panel/menu background (`#FBF1DA`).
    pub const PANEL_BG: [f32; 4] = [0.984, 0.945, 0.855, 1.0];
    /// Alias of [`PANEL_BG`]; do not fork a second menu background value.
    pub const MENU_BG: [f32; 4] = PANEL_BG;
    /// Panel inner fill — warm cream, never pure white (`#F8EED5`).
    pub const PANEL_FILL: [f32; 4] = [0.973, 0.933, 0.835, 1.0];
    /// Alternate panel background — soft warm yellow (`#F1E4C4`).
    pub const PANEL_BG_ALT: [f32; 4] = [0.945, 0.894, 0.769, 1.0];

    /// Amber borders / dividers (`#D8A500`).
    pub const FRAME: [f32; 4] = [0.847, 0.647, 0.0, 1.0];
    /// PNG chrome multiply tint (`#ECD48B`) — light warm, not dark brown.
    pub const UI_MAIN: [f32; 4] = [0.925, 0.831, 0.545, 1.0];
    /// Tint applied to legacy generic menu button artwork outside the main
    /// menu, where the light button treatment is still intentionally used.
    pub const BUTTON_IMAGE_TINT: [f32; 4] = UI_MAIN;
    /// Lemon-gold tint for the main-menu button states (`#D9AD00`).
    pub const MAIN_MENU_BUTTON_TINT: [f32; 4] = [0.851, 0.678, 0.0, 1.0];
    /// Dark ink text used on the lemon main-menu button states.
    pub const MAIN_MENU_BUTTON_TEXT: [f32; 4] = [0.169, 0.169, 0.122, 1.0];
    pub const MAIN_MENU_BUTTON_TEXT_DISABLED: [f32; 4] = super::alpha(MAIN_MENU_BUTTON_TEXT, 0.45);
    pub const UI_SUBTLE: [f32; 4] = [0.929, 0.871, 0.651, 1.0];
    pub const UI_HIGHLIGHT: [f32; 4] = [1.0, 0.898, 0.4, 1.0];

    /// Shared interaction surfaces for ordinary menu chrome.
    pub const SURFACE_HOVER: [f32; 4] = PANEL_BG_ALT;
    pub const SURFACE_PRESSED: [f32; 4] = UI_SUBTLE;
    pub const SURFACE_DISABLED: [f32; 4] = super::alpha(PANEL_BG_ALT, 0.55);
    pub const BORDER_SUBTLE: [f32; 4] = super::alpha(FRAME, 0.55);
    pub const FOCUS_RING: [f32; 4] = ACCENT_LEMON;
    pub const INPUT_SELECTION: [f32; 4] = super::alpha(ACCENT_LEMON, 0.22);
    pub const INPUT_BORDER: [f32; 4] = BORDER_SUBTLE;
    pub const INPUT_BORDER_FOCUSED: [f32; 4] = FOCUS_RING;
    pub const SCROLLBAR_THUMB: [f32; 4] = super::alpha(FRAME, 0.78);

    /// Dark ink body text (`#2B2B1F`) — light surfaces, dark ink.
    pub const TEXT_PRIMARY: [f32; 4] = [0.169, 0.169, 0.122, 1.0];
    pub const TEXT_DISABLED: [f32; 4] = super::alpha(TEXT_PRIMARY, 0.4);
    pub const TEXT_MUTED: [f32; 4] = super::alpha(TEXT_PRIMARY, 0.6);
    /// Menu emphasis — deep lemon gold (`#A88500`), not item quality gold.
    pub const TEXT_EMPHASIS: [f32; 4] = [0.659, 0.522, 0.0, 1.0];

    /// Brand lemon (`#FFD600`) — borders, highlights, logo only; never body
    /// text.
    pub const ACCENT_LEMON: [f32; 4] = [1.0, 0.839, 0.0, 1.0];
    /// Secondary accent / former `TEXT_VELORITE` replacement (`#7CB518`).
    pub const ACCENT_LIME: [f32; 4] = [0.486, 0.710, 0.094, 1.0];

    /// List selection fill — lemon yellow, matching the menu chrome.
    pub const SELECTION_ACTIVE: [f32; 4] = [1.0, 0.839, 0.0, 1.0];
    /// Inactive selection — soft lemon that reads on light panels (`#EBD680`).
    pub const SELECTION_INACTIVE: [f32; 4] = [0.922, 0.839, 0.502, 1.0];

    /// Tooltip background (inverted dark) (`#2B2B1F`).
    pub const TOOLTIP_BACK: [f32; 4] = [0.169, 0.169, 0.122, 1.0];
    /// HUD tooltip / item-tooltip `ImageFrame` center fill (`#212117`).
    pub const TOOLTIP_FRAME_FILL: [f32; 4] = [0.13, 0.13, 0.09, 1.0];

    /// Overlay scrim alpha (soft dark ≈140/255).
    pub const OVERLAY_SCRIM_A: f32 = 140.0 / 255.0;
}

/// Reference legendary quality gold (must stay in `hud`; duplicated here for
/// tests only).
#[cfg(test)]
const QUALITY_LEGENDARY_REF: [f32; 4] = [0.92, 0.76, 0.0, 1.0];

/// Convert theme sRGBA to a Conrod color.
#[inline]
pub const fn to_conrod(c: [f32; 4]) -> conrod_core::Color {
    conrod_core::Color::Rgba(c[0], c[1], c[2], c[3])
}

/// Override the alpha channel of a theme sRGBA (e.g. faint text variants).
#[inline]
pub const fn alpha(c: [f32; 4], a: f32) -> [f32; 4] { [c[0], c[1], c[2], a] }

/// Convert theme sRGBA to an Iced color (includes alpha).
#[inline]
pub const fn to_iced(c: [f32; 4]) -> iced::Color {
    iced::Color {
        r: c[0],
        g: c[1],
        b: c[2],
        a: c[3],
    }
}

/// Convert theme sRGBA to a floating-point `vek::Rgba` for Iced slider styles.
#[inline]
pub const fn to_vek(c: [f32; 4]) -> vek::Rgba<f32> { vek::Rgba::new(c[0], c[1], c[2], c[3]) }

/// Convert theme sRGBA to `Rgba<u8>` with **round** (e.g. 0.61×255 → 156).
#[inline]
pub const fn to_rgba_u8(c: [f32; 4]) -> vek::Rgba<u8> {
    vek::Rgba::new(
        channel_u8(c[0]),
        channel_u8(c[1]),
        channel_u8(c[2]),
        channel_u8(c[3]),
    )
}

#[inline]
pub const fn channel_u8(x: f32) -> u8 {
    let x = if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    };
    (x * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_matches_historical_menu_ui_main() {
        // Historical char-select cold teal (0.61, 0.70, 0.70) → Rgba(156, 179, 179,
        // 255)
        let c = to_rgba_u8([0.61, 0.70, 0.70, 1.0]);
        assert_eq!(c, vek::Rgba::new(156, 179, 179, 255));
    }

    #[test]
    fn channel_round_not_truncate() {
        assert_eq!(channel_u8(0.61), 156);
        assert_eq!(channel_u8(0.70), 179);
    }

    #[test]
    fn alpha_uses_same_round_rule() {
        assert_eq!(channel_u8(0.2), 51);
        let c = to_rgba_u8([1.0, 1.0, 1.0, 0.2]);
        assert_eq!(c.a, 51);
    }

    #[test]
    fn round_trip_error_within_half_lsb() {
        let src = brand::UI_MAIN;
        let u8c = to_rgba_u8(src);
        let back = [
            u8c.r as f32 / 255.0,
            u8c.g as f32 / 255.0,
            u8c.b as f32 / 255.0,
            u8c.a as f32 / 255.0,
        ];
        for i in 0..4 {
            assert!(
                (back[i] - src[i]).abs() <= 0.5 / 255.0 + f32::EPSILON,
                "channel {i}: src={} back={}",
                src[i],
                back[i]
            );
        }
    }

    #[test]
    fn accent_lemon_is_pure_yellow_not_body_text() {
        // #FFD600 — bright pure yellow: high R/G, near-zero B (frame/highlight only).
        let c = brand::ACCENT_LEMON;
        assert!(c[0] >= 0.95, "R high");
        assert!(c[1] >= 0.80, "G high");
        assert!(c[2] <= 0.05, "B ≈ 0");
    }

    #[test]
    fn legendary_ref_stays_amber_gold_shaped() {
        // Legendary stays amber/gold (R > G, moderate G) — distinct from pure lemon
        // yellow.
        let c = QUALITY_LEGENDARY_REF;
        assert!(c[0] > c[1] && c[1] > c[2]);
        assert!(c[0] - c[1] >= 0.10);
        assert!(c[1] < 0.80, "legendary G lower than pure lemon yellow");
    }

    #[test]
    fn lemon_and_legendary_are_distinguishable() {
        let lemon = brand::ACCENT_LEMON;
        let leg = QUALITY_LEGENDARY_REF;
        // Pure lemon is brighter/yellower (higher G) than legendary amber gold.
        assert!(lemon[1] > leg[1] + 0.05);
    }

    #[test]
    fn selection_active_is_lemon() {
        let c = brand::SELECTION_ACTIVE;
        assert_eq!(c, brand::ACCENT_LEMON, "selection follows brand lemon");
        assert!(c[1] >= 0.8, "G high for lemon yellow");
        assert!(c[2] <= 0.15, "B low for lemon yellow");
    }

    #[test]
    fn text_primary_is_dark_ink() {
        let c = brand::TEXT_PRIMARY;
        assert!(c[0] < 0.25 && c[1] < 0.25 && c[2] < 0.25);
    }

    #[test]
    fn panel_bg_is_warm_lemon_white() {
        let c = brand::PANEL_BG;
        assert!(c[0] > c[1] && c[1] > c[2]);
        assert!(c[0] >= 0.95, "panel background should read as warm white");
        assert!(c[2] >= 0.85, "warm tint (not pure white)");
    }

    #[test]
    fn to_conrod_preserves_channels() {
        let c = to_conrod(brand::TEXT_PRIMARY);
        match c {
            conrod_core::Color::Rgba(r, g, b, a) => {
                assert!((r - brand::TEXT_PRIMARY[0]).abs() < f32::EPSILON);
                assert!((g - brand::TEXT_PRIMARY[1]).abs() < f32::EPSILON);
                assert!((b - brand::TEXT_PRIMARY[2]).abs() < f32::EPSILON);
                assert!((a - brand::TEXT_PRIMARY[3]).abs() < f32::EPSILON);
            },
            _ => panic!("expected Rgba"),
        }
    }

    #[test]
    fn to_iced_preserves_channels() {
        let c = to_iced(brand::ACCENT_LIME);
        assert!((c.r - brand::ACCENT_LIME[0]).abs() < f32::EPSILON);
        assert!((c.g - brand::ACCENT_LIME[1]).abs() < f32::EPSILON);
        assert!((c.b - brand::ACCENT_LIME[2]).abs() < f32::EPSILON);
        assert!((c.a - brand::ACCENT_LIME[3]).abs() < f32::EPSILON);
    }

    #[test]
    fn to_vek_preserves_channels() {
        let c = to_vek(brand::FOCUS_RING);
        assert_eq!(
            c,
            vek::Rgba::new(
                brand::FOCUS_RING[0],
                brand::FOCUS_RING[1],
                brand::FOCUS_RING[2],
                brand::FOCUS_RING[3],
            )
        );
    }

    #[test]
    fn text_disabled_uses_primary_rgb() {
        assert_eq!(brand::TEXT_DISABLED[0], brand::TEXT_PRIMARY[0]);
        assert_eq!(brand::TEXT_DISABLED[1], brand::TEXT_PRIMARY[1]);
        assert_eq!(brand::TEXT_DISABLED[2], brand::TEXT_PRIMARY[2]);
        assert!((brand::TEXT_DISABLED[3] - 0.4).abs() < f32::EPSILON);
    }
}
