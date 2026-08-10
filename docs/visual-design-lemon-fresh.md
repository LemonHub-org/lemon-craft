# LemonCraft Visual Design — Lemon Fresh

> Status: palette **v2, active** — light-theme direction replacing v1
> (Warm Craft Fantasy, rejected for clashing with the game's bright voxel
> world). Implementation: `voxygen/src/ui/theme.rs` (`brand::*` tokens).

## 1. Direction

**Lemon Fresh**: light warm-white surfaces, dark ink text, and real lemon
yellow as the single dominant accent. The UI must *sit inside* the bright
voxel world (cool skies, green fields), not dim it:

- Light panels let the world show through (semi-transparent feel)
- Dark ink text on light surfaces (contrast ≥ 7:1)
- Lemon `#FFD600` for borders, highlights, icons — never body text
- Tooltips invert to dark for layering

## 2. Palette v2 (draft)

| Token | Value | Hex | Usage |
|---|---|---|---|
| `PANEL_BG` | `[0.984, 0.945, 0.855]` | `#FBF1DA` | Panel/menu background (alias `MENU_BG`) |
| `PANEL_FILL` | `[0.973, 0.933, 0.835]` | `#F8EED5` | Panel inner fill |
| `PANEL_BG_ALT` | `[0.945, 0.894, 0.769]` | `#F1E4C4` | Alternate panel background |
| `FRAME` | `[0.847, 0.647, 0.0]` | `#D8A500` | Borders, dividers (amber) |
| `UI_MAIN` | `[0.925, 0.831, 0.545]` | `#ECD48B` | Chrome multiply tint |
| `UI_SUBTLE` | `[0.898, 0.870, 0.780]` | `#E5DEC7` | Subtle chrome |
| `UI_HIGHLIGHT` | `[1.0, 0.898, 0.4]` | `#FFE566` | Chrome highlight |
| `TEXT_PRIMARY` | `[0.169, 0.169, 0.122]` | `#2B2B1F` | Body text (dark ink) |
| `TEXT_DISABLED` | `TEXT_PRIMARY @ 0.4` | — | Disabled text |
| `TEXT_MUTED` | `TEXT_PRIMARY @ 0.6` | — | Muted text |
| `TEXT_EMPHASIS` | `[0.659, 0.522, 0.0]` | `#A88500` | Emphasis (deep lemon gold) |
| `ACCENT_LEMON` | `[1.0, 0.839, 0.0]` | `#FFD600` | **Brand lemon** — borders, highlights, logo |
| `ACCENT_LIME` | `[0.486, 0.710, 0.094]` | `#7CB518` | Secondary accent (readable green) |
| `SELECTION_ACTIVE` | `(97, 255, 18)` | `#61FF12` | List selection fill (signal green) |
| `SELECTION_INACTIVE` | `(166, 166, 110)` | `#A6A66E` | Inactive selection (olive on light) |
| `TOOLTIP_BACK` | `[0.169, 0.169, 0.122]` | `#2B2B1F` | Tooltip background (inverted) |
| `TOOLTIP_FRAME_FILL` | `[0.13, 0.13, 0.09]` | `#212117` | Tooltip ImageFrame center fill |
| `OVERLAY_SCRIM_A` | `140 / 255` | — | Overlay scrim alpha (soft dark) |

## 3. Rules

- **Light surfaces, dark ink**: text tokens are dark; panels are warm-white.
  Body text contrast ≥ 7:1 (`#2B2B1F` on `#FBF1DA` ≈ 12:1).
- **Lemon is a frame, not a flood**: `ACCENT_LEMON` for borders, selection
  rings, icons, progress accents — never body text, never panel fills.
- **Citrus ≠ legendary gold**: keep gold-shaped quality colors (`R > G > B`)
  distinct from lemon (`B ≈ 0`, high `G`); tests enforce both shapes.
- **Tooltips invert**: dark tooltip on light UI for hierarchy.
- **Selection ≠ lemon**: list selection uses signal green, never yellow.
- Semantic combat/quality/chat colors stay frozen in the HUD.
- Conversions unchanged: `to_conrod` / `to_iced` / `to_rgba_u8` (round);
  `alpha(c, a)` for faint variants.

## 4. Relationship to the World

- Panels are warm-white and feel translucent; the bright world stays bright.
- No large dark surfaces in normal HUD chrome (only tooltips/scrims).
- Menu background keeps a soft lemon-tinted warm white, not deep brown.

## 5. Versioning

- v1 (Warm Craft Fantasy) is **rejected**; do not restore its dark brown
  surfaces. v2 is the active direction until further design revisions.
- Palette changes require a new version — bump the module doc, update this
  document, migrate consumers explicitly.
