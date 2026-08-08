# LemonCraft Brand Guide

> Status: v0.1 — foundation for visual identity work (menus, HUD, logo, loading screens)

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

| Token | Hex | Usage |
|---|---|---|
| `lemon` (primary) | `#FFD600` | Brand accent, highlights, logo rind |
| `zest` (secondary) | `#A8D400` | Crafting/positive states, secondary accents |
| `pith` (light) | `#FDF6E3` | Backgrounds, cards, text on dark |
| `peel` (dark) | `#1B1B10` | Deep backgrounds, title screen base |
| `leaf` (green) | `#2E7D32` | Success/health accents |
| `pit` (ink) | `#0E0E08` | Text, darkest surfaces |

Principles:

- Lemon yellow is the *single* dominant accent; use it sparingly for impact
- Dark warm-green-black surfaces (`peel`/`pit`) carry the UI; `lemon` never
  becomes a background color
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

- **Title screen**: `lemon` wordmark on `peel` background, voxel-lemon mark
  center stage
- **Loading screen**: mark + progress on `peel`, subtle lemon-yellow accents
- **HUD**: `lemon` for the hotbar highlight and interaction prompts;
  `zest` for positive feedback; item quality colors unchanged (RPG semantics)
- **Error/danger**: keep warm red (`#C62828`) for safety semantics — never
  yellow for errors

## 7. Out of Scope (until art pipeline)

- Final logo voxel model + PNG export (art task)
- Wordmark custom typeface (art task)
- Sound identity / audio logo
