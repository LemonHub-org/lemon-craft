# LemonCraft Visual Design — Warm Craft Fantasy

> Status: palette **v1, frozen** (implemented in `voxygen/src/ui/theme.rs`).
> This document is the source of truth for brand/chrome colors shared by the
> Conrod HUD and Iced menus. Semantic combat/quality/chat colors stay in the
> HUD (frozen, untouched by branding).

## 1. Direction

**Warm Craft Fantasy**: deep warm-brown surfaces, warm off-white text, and a
citrus accent that reads as *lemon*, not legendary gold. The palette replaces
the upstream cool-teal Veloren chrome (`#9cb3b3`) while preserving the game's
craft/fantasy feel.

## 2. Palette v1 (frozen)

| Token | Value | Hex | Usage |
|---|---|---|---|
| `PANEL_BG` | `[0.102, 0.090, 0.071]` | `#1A1712` | Panel/menu background (alias `MENU_BG`) |
| `PANEL_FILL` | `[0.180, 0.204, 0.157]` | `#2E3428` | Panel inner fill |
| `PANEL_BG_ALT` | `[0.141, 0.125, 0.094]` | `#242016` | Alternate panel background |
| `FRAME` | `[0.541, 0.451, 0.282]` | `#8A7348` | Frame borders |
| `UI_MAIN` | `[0.604, 0.518, 0.376]` | `#9A8460` | PNG chrome multiply tint |
| `UI_SUBTLE` | `[0.165, 0.149, 0.125]` | `#2A261F` | Subtle chrome |
| `UI_HIGHLIGHT` | `[0.769, 0.659, 0.416]` | `#C4A86A` | Chrome highlight |
| `TEXT_PRIMARY` | `[0.949, 0.937, 0.902]` | `#F2EFE6` | Body text (warm off-white) |
| `TEXT_DISABLED` | `TEXT_PRIMARY @ 0.2` | — | Disabled text |
| `TEXT_MUTED` | `TEXT_PRIMARY @ 0.5` | — | Muted text |
| `TEXT_EMPHASIS` | `[1.0, 0.85, 0.5]` | `#FFD980` | Menu emphasis (not quality gold) |
| `ACCENT_LEMON` | `[0.847, 0.878, 0.290]` | `#D8E04A` | Brand citrus; outline/logo only |
| `ACCENT_LIME` | `[0.788, 0.851, 0.290]` | `#C9D94A` | Secondary accent (replaces `TEXT_VELORITE`) |
| `SELECTION_ACTIVE` | `(97, 255, 18)` | `#61FF12` | List selection fill (signal green) |
| `SELECTION_INACTIVE` | `(97, 97, 25)` | `#616119` | Inactive selection fill |
| `TOOLTIP_BACK` | `(20, 18, 10)` | `#14120A` | Tooltip background |
| `TOOLTIP_FRAME_FILL` | `[0.08, 0.07, 0.04]` | `#14120A` | Tooltip ImageFrame center fill |
| `OVERLAY_SCRIM_A` | `230 / 255` | — | Overlay scrim alpha |

## 3. Rules

- **Citrus ≠ gold**: `ACCENT_LEMON`/`ACCENT_LIME` are green-yellow
  (`G ≥ R`, `B ≤ 0.35`); legendary quality gold stays warm
  (`R > G > B`). Tests enforce both shapes.
- **Selection ≠ citrus**: list selection uses signal green, never lemon.
- Semantic combat/quality/chat colors are frozen in the HUD and must not be
  re-themed.
- Conversions: `to_conrod` / `to_iced` / `to_rgba_u8` (round, matches
  historical `156/179/179` behavior); `alpha(c, a)` derives faint variants.

## 4. Versioning

- Palette changes require a new frozen version (v2, v3…) — bump the module
  doc, update this document, and migrate consumers explicitly.
- Do **not** fork per-widget colors; extend tokens here instead.
