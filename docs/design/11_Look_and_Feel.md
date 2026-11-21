# 11 — Look and Feel

## 11.1 Direction: "Storybook workshop"

The PM asked for *"a bit dreamy, cartoony colour design compared to other CADs, with a Pastel option."* Professional CAD is grey, blue and dense. MakerCAD should feel like **a sunny workshop drawn for a picture book**: warm, soft and friendly. But it is still a precision instrument, so every number stays crisp and legible.

Principles:
1. **Soft surfaces, crisp data.** Rounded panels, gentle shadows and pastel fills. Dimensions, snaps and edges stay high-contrast and sharp.
2. **Colour means something.** Each environment has an accent colour, and each line type (cut, crease, stitch) has a reserved colour used everywhere: screen, PDF, projector.
3. **Hand-made touches, used sparingly:** paper texture on empty canvases, a slightly wobbly "sketched" stroke style for construction geometry (optional), and Sancho.
4. **Our own identity.** It must not resemble SolidCraft's look (licence, doc 02 §2.1.1) or DraftCraft's. Layout ideas are fine; visual identity is ours.

## 11.2 Themes (`mk-theme`)

Themes are token sets (JSON/TOML), hot-swappable, and user-editable.

| Theme | Character | Background | Panels | Ink | Accent use |
|---|---|---|---|---|---|
| **Daydream** (default) | Warm, saturated-soft, cartoony | Sky gradient, cream to pale blue `#FFF8EC → #E6F2FF` | Cream `#FFFDF7`, soft shadow | Deep ink `#2B2A4C` | Per-environment accents at medium saturation |
| **Pastel** | Lighter, calmer, lower saturation | `#F7F4FB` | `#FFFFFF` with lavender border `#E7E0F5` | `#3A3550` | Accents shifted toward pastel (sat ≤ 45%) |
| **Night Workshop** | Dark mode, warm | `#1F1D2B` with subtle vignette | `#2A2838` | `#F2EEE6` | Accents brightened for contrast |
| **Blueprint** | Optional "pro" homage | Deep blue `#123B6D` | `#16477F` | White | Monochrome + amber selection |
| **High Contrast** | Accessibility | White or black | Solid | Pure | WCAG AAA for text and lines |

**Environment accents (Daydream / Pastel):**

| Environment | Daydream | Pastel |
|---|---|---|
| Sketch | Coral `#FF8A7A` | `#FFC2B8` |
| Solid | Sky `#5FA8F5` | `#B5D6FA` |
| Quick Build | Sunflower `#F7C548` | `#FBE3A0` |
| Space & Scan | Mint `#4CC9A6` | `#AEE8D7` |
| Stock & Cut | Wood `#C98B55` | `#E8C9A8` |
| Fold & Package | Lilac `#A68CF0` | `#D6CBF8` |
| Soft Goods | Rose `#F07FB5` | `#F8C3DD` |
| Motion | Teal `#2FB5C8` | `#A5E0E8` |
| Plan | Leaf `#7BC47F` | `#C3E6C4` |
| Journal | Peach `#F5A66B` | `#FAD3B4` |

All values are starting points, to be checked with a contrast validator. Accent colours never carry text below 4.5:1 contrast. Text on accent uses the ink colour.

**Reserved manufacturing line colours** (consistent across theme, PDF and projector, plus line *patterns* so colour-blind users and monochrome printers can tell them apart):

| Line | Colour | Pattern |
|---|---|---|
| Cut | Red `#E5484D` | Solid |
| Crease / valley fold | Blue `#3E7BFA` | Dashed (6-3) |
| Mountain fold | Blue `#3E7BFA` | Dash-dot |
| Perforation | Green `#2BA471` | Dotted (2-2) |
| Score / half-cut | Orange `#F08C00` | Long dash (12-4) |
| Stitch line | Magenta `#C2255C` | Short dash (3-2) |
| Seam allowance / cut edge (fabric) | Ink | Solid thin |
| Bleed | Cyan `#15AABF` | Thin solid |
| Grain line | Ink | Arrowed line |

## 11.3 Viewport style

- **Shading:** flat colour plus soft rim light and a gentle hemispheric ambient ("clay"), with **ink outlines** (silhouette + crease edges). This cartoony "toon-ish" look is also *cheap*: no shadows maps, no SSAO, no post on potato. On capable machines it adds soft contact shadows and an optional paper grain overlay.
- **Materials display:** wood shows a procedural light grain texture aligned to the stock binding's grain axis (tiny, generated, cached), fabric a weave, cardboard flutes on cut edges. Toggle "Material look" vs. "Design colours" (by part or by stock).
- **Ground:** the stage (doc 05 §5.5). For a room it's a soft floor with the room shell in translucent pastel walls (fade the wall nearest the camera automatically). For a bench it's a cutting mat with a grid.
- **Scan ghost:** points or mesh in desaturated grey-blue at 40% opacity, never competing with design geometry.
- **Selection:** a warm golden outline with a soft glow (no glow on potato). Pre-highlight is a thin outline.
- **Dimensions:** crisp ink on a cream pill background, always screen-aligned and readable.

## 11.4 Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ☁ Alcove dresser ▾  [Intent: "Dresser for the weird cutout…"]   ◷ 12 days    │  project bar
│ ( Sketch )( Solid )( Quick )( Space )( Stock )( Plan )( Journal )  + Env       │  environment tabs (accent-coloured)
├────────┬─────────────────────────────────────────────────────────┬───────────┤
│Toolbox │                                                         │ Context   │
│(dynamic│                    VIEWPORT / STAGE                     │ panel     │
│ per    │                                                         │ (props,   │
│ project│                                   [Sancho]              │  feature  │
│ + env) │                                                         │  dialog,  │
│        │                                                         │  findings)│
│ ────── │                                                         │           │
│Inventory│                                                        │           │
│drawer  │                                                         │           │
├────────┴─────────────────────────────────────────────────────────┴───────────┤
│ Timeline (features — maker wording toggle)                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Pine 2x4 ███████░░ 11/15 · waste 9% │ Fits ✓ 3mm │ $38/$120 │ ⌘K search │ ⚙ │  status bar
└──────────────────────────────────────────────────────────────────────────────┘
```

- **Environment tabs** replace a ribbon of 200 icons. Within an environment, the **toolbox** is short and *project-specific* (built by the interpreter). The full command set is always available via **⌘K / Ctrl+K command search**, which knows synonyms and plain language: "cut a slot", "make it thinner", "fold this edge".
- **Ribbon option:** users coming from AutoCAD/DraftCraft can switch the toolbox to a **compact ribbon** layout (DraftCraft-style density, our look). It is the same commands and data from `ui-registry`, presented differently.
- **Keymaps:** Fusion-style (default), SketchUp-style, AutoCAD-style. Each is complete and switchable, and the conflicts are documented.
- **Context panel** shows the active command's options (live preview = the real document, per v5 rule "preview is push/undo of the real document") and the Findings list.
- **Panels are dockable** with saved layouts per environment. **Small screens** (1366 × 768, common on old laptops) get a collapsed layout automatically: toolbox icons only, context panel as an overlay.

## 11.5 Iconography (`mk-look`)

- Original icon set (licence requirement): **rounded, two-tone, thick-stroke** icons (2 px at 24 px size) with a pastel fill and ink stroke, so they feel hand-drawn but read clearly at 16 px.
- Code-drawn where practical (vector paths in code, as CADCraft does), so icons scale and recolour per theme.
- Environment glyphs: a pencil (Sketch), a cube (Solid), a hand pushing a block (Quick Build), a phone-with-room (Space), a board with a saw line (Stock), a folded box (Fold), a needle and thread (Soft Goods), a hinge (Motion), a calendar (Plan), a notebook (Journal).

## 11.6 Typography

- UI: a friendly humanist sans with good numerals (e.g. **Nunito** or **Atkinson Hyperlegible**, both under the SIL Open Font License). Atkinson has a strong legibility story for the accessibility theme.
- Numbers and dimensions: tabular figures, with fractions rendered properly (³⁵⁄₁₆ style option or 35 7/16).
- Single-stroke engraving font for laser and plotter text (from CADCraft's original single-stroke font, MIT/Apache).
- A tiny handwritten accent font only for Sancho's speech bubbles and the mood board.

## 11.7 Motion and feedback

- UI animation: short (120–200 ms) ease-out, used for state changes only. **Reduced-motion setting** and potato default: no animation.
- Success moments: a subtle confetti burst when the meter goes green, or a stamp "Fits!" on the fit chip. Rare, earned, and disable-able.
- Sounds: optional and off by default. Soft wooden clicks for snaps, if enabled.

## 11.8 Accessibility

- Full keyboard operation. Every command reachable in ≤ 2 keystrokes via search.
- Colour is never the only signal (line patterns, labels, icons).
- UI scale 75–200%, independent of OS scaling.
- Screen-reader labels on panels and dialogs (egui AccessKit support).
- Dyslexia-friendly font option (Atkinson Hyperlegible) and a plain-language Sancho mode.
