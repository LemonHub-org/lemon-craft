# LemonCraft Brand Guide

> Status: v0.2 — foundation for visual identity work (menus, HUD, logo, loading screens)

## 1. Brand Essence

**LemonCraft** is a voxel action-adventure RPG: handcrafted worlds with a
zesty, fresh twist on the classic craft-and-explore loop.

- **Personality**: fresh, playful, handcrafted, adventurous
- **Tone**: bright but not childish; sour humor, sweet payoff
- **Tagline**: **"Squeeze the world."**
  - Alternatives: "Fresh voxel adventures." / "Sour. Sweet. Crafted."

## 2. Logo

Concept: a **lemon slice made of voxels** — the game's world is blocks, so the
logo is a lemon rendered as a voxel model.

- Primary mark: voxel lemon slice (yellow rind, lighter flesh, white segments)
- Wordmark: "LemonCraft" set in a fantasy-flavored display typeface
- Constraint: the mark must read at 16px (HUD) and at 512px (title screen)
- The mark doubles as the in-game crafting motif (the game's "lemon" is both
  a resource and the world's signature material)

## 3. Color System

The UI palette is **Warm Craft Fantasy** — deep warm-brown surfaces, warm
off-white text, citrus accents. See
[`visual-design-warm-craft-fantasy.md`](visual-design-warm-craft-fantasy.md)
for the frozen palette v1 token table and rules.

Key tokens:

| Token | Hex | Usage |
|---|---|---|
| `PANEL_BG` / `MENU_BG` | `#1A1712` | Panel/menu background |
| `TEXT_PRIMARY` | `#F2EFE6` | Warm off-white body text |
| `TEXT_EMPHASIS` | `#FFD980` | Menu emphasis (not quality gold) |
| `ACCENT_LEMON` | `#D8E04A` | Brand citrus — outline/logo only |
| `ACCENT_LIME` | `#C9D94A` | Secondary accent (replaces old teal) |
| `SELECTION_ACTIVE` | `#61FF12` | List selection (signal green) |

Principles:

- Lemon/citrus is the *single* dominant accent; use sparingly for impact
- Citrus must never read as legendary gold (`G ≥ R`, `B` low) — tests enforce
- Deep warm backgrounds carry the UI; citrus never becomes a background color
- Do not mix with cold blues for branding (reserved for item-quality semantics)

## 4. Typography

- **Display / wordmark**: a fantasy-flavored serif or pixel-fantasy face
  (the game already ships `Alkhemikal`/`Metamorphous` — candidate for the
  wordmark; a custom ligature for "LemonCraft" is a future art task)
- **UI/HUD**: high-legibility pixel or humanist sans (existing `bdfUMplus`
  pixel face for numbers; `OpenSans`/`Sarabun` for body)
- **Fallback**: CJK faces already shipped (NotoSansTC / WenQuanYiZenHei)

## 5. Voice & Naming

- Always write the name as **LemonCraft** (one word, camel case). Never
  "Lemon Craft" or "LEMONCRAFT".
- Project identifier: `lemoncraft` (lowercase) for crates, binaries, paths.
- Community-facing copy uses the tagline tone: short, playful, concrete.

## 6. Application

- **Title screen**: `lemon` wordmark on deep warm background, voxel-lemon mark
  center stage
- **Loading screen**: mark + progress on deep background, subtle citrus accents
- **HUD**: citrus for hotbar highlight and interaction prompts; signal green
  for positive feedback; item quality colors unchanged (RPG semantics)
- **Error/danger**: keep warm red for safety semantics — never yellow for errors

## 7. Out of Scope (until art pipeline)

- Final logo voxel model + PNG export (art task)
- Wordmark custom typeface (art task)
- Sound identity / audio logo
