//! Warm Craft Fantasy theme tokens (palette v1).
//!
//! Single source for brand/chrome colors shared by Conrod HUD and Iced menus.
//! Semantic combat/quality/chat colors stay in `hud` (frozen). See
//! `docs/visual-design-warm-craft-fantasy.md`.

/// sRGB 0..=1 channel storage (not premultiplied). Palette **v1** frozen.
pub mod brand {
    pub const PANEL_BG: [f32; 4] = [0.102, 0.090, 0.071, 1.0];
    /// Alias of [`PANEL_BG`]; do not fork a second menu background value.
    pub const MENU_BG: [f32; 4] = PANEL_BG;
    pub const PANEL_FILL: [f32; 4] = [0.180, 0.204, 0.157, 1.0];
    pub const PANEL_BG_ALT: [f32; 4] = [0.141, 0.125, 0.094, 1.0];

    pub const FRAME: [f32; 4] = [0.541, 0.451, 0.282, 1.0];
    /// PNG chrome multiply tint (`#9a8460`).
    pub const UI_MAIN: [f32; 4] = [0.604, 0.518, 0.376, 1.0];
    pub const UI_SUBTLE: [f32; 4] = [0.165, 0.149, 0.125, 1.0];
    pub const UI_HIGHLIGHT: [f32; 4] = [0.769, 0.659, 0.416, 1.0];

    /// Warm off-white body text (`#f2efe6`).
    pub const TEXT_PRIMARY: [f32; 4] = [0.949, 0.937, 0.902, 1.0];
    pub const TEXT_DISABLED: [f32; 4] = [0.949, 0.937, 0.902, 0.2];
    pub const TEXT_MUTED: [f32; 4] = [0.949, 0.937, 0.902, 0.5];
    /// Menu emphasis (not item quality gold).
    pub const TEXT_EMPHASIS: [f32; 4] = [1.0, 0.85, 0.5, 1.0];

    /// Brand citrus accent (`#d8e04a`); outline/logo only — not legendary gold.
    pub const ACCENT_LEMON: [f32; 4] = [0.847, 0.878, 0.290, 1.0];
    /// Secondary brand accent / former `TEXT_VELORITE` replacement (`#c9d94a`).
    pub const ACCENT_LIME: [f32; 4] = [0.788, 0.851, 0.290, 1.0];

    /// List selection fill — pure signal green; never citrus lemon.
    pub const SELECTION_ACTIVE: [f32; 4] = [97.0 / 255.0, 1.0, 18.0 / 255.0, 1.0];
    pub const SELECTION_INACTIVE: [f32; 4] = [97.0 / 255.0, 97.0 / 255.0, 25.0 / 255.0, 1.0];

    pub const TOOLTIP_BACK: [f32; 4] = [20.0 / 255.0, 18.0 / 255.0, 10.0 / 255.0, 1.0];
    /// HUD tooltip / item-tooltip `ImageFrame` center fill.
    pub const TOOLTIP_FRAME_FILL: [f32; 4] = [0.08, 0.07, 0.04, 1.0];

    /// Overlay scrim alpha (≈230/255).
    pub const OVERLAY_SCRIM_A: f32 = 230.0 / 255.0;
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

/// Convert theme sRGBA to `Rgba<u8>` with **round** (matches historical menu
/// `156/179/179`).
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
        // Old char-select UI_MAIN was (0.61, 0.70, 0.70) → Rgba(156, 179, 179, 255)
        let c = to_rgba_u8([0.61, 0.70, 0.70, 1.0]);
        assert_eq!(c, vek::Rgba::new(156, 179, 179, 255));
    }

    #[test]
    fn channel_round_not_truncate() {
        // 0.61 * 255 = 155.55 → round 156, truncate would be 155
        assert_eq!(channel_u8(0.61), 156);
        assert_eq!(channel_u8(0.70), 179);
    }

    #[test]
    fn alpha_uses_same_round_rule() {
        assert_eq!(channel_u8(0.2), 51); // 51.0
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
    fn accent_lemon_is_citrus_not_gold() {
        let c = brand::ACCENT_LEMON;
        assert!(c[1] >= c[0], "G >= R");
        assert!(c[1] - c[0] >= 0.02, "G - R >= 0.02");
        assert!(c[2] <= 0.35, "B <= 0.35");
    }

    #[test]
    fn legendary_ref_stays_gold_shaped() {
        let c = QUALITY_LEGENDARY_REF;
        assert!(c[0] > c[1] && c[1] > c[2]);
        assert!(c[0] - c[1] >= 0.10);
    }

    #[test]
    fn selection_active_is_signal_green() {
        let c = brand::SELECTION_ACTIVE;
        assert!(c[1] >= 0.90);
        assert!(c[0] <= 0.50);
        assert!(c[2] <= 0.20);
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
}
