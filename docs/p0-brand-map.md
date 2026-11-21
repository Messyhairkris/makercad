# P0 — SolidCraft brand-crate dependency map

Pinned checkout: `third_party/solidcraft`, rev `906b5e86` (2026-11-19). This is one of P0's exit items
(doc 14): which parts of SolidCraft MakerCAD may build on, and how much of the shell it must replace.

## What the brand licence covers

`LICENSE-BRAND` covers SolidCraft's look and feel, separately from its MIT/Apache code:

| Brand material | What it is |
|---|---|
| `crates/ui-kit` (`solidcraft-ui-kit`) | the code and design of the controls, panels, menus, ribbon, app bar, history bar, navigation bar, view compass, Preferences window, start page, window chrome |
| `crates/look` (`solidcraft-look`) | the icon masters and artwork (441 asset files), the code-drawn glyphs, the built-in themes |
| `crates/theme` (`solidcraft-theme`) | design tokens, palettes, accent ramps, sizes and spacing |
| name, logo, app icon, brand kit | `assets/branding`, `assets/icons/app` |

Using any of it in another program, including **linking to** the UI kit, the look or the theme, needs
the maintainers' written permission (§3a), and so does a look "that could be taken for SolidCraft's"
(§3c). MakerCAD does not ask: we do not contact the SolidCraft makers. So MakerCAD links none of
these crates and looks nothing like SolidCraft.

## Who depends on them

Every `Cargo.toml` under `crates/`, `apps/` and `solidcore/crates/` was searched for the four crates
(direct dependencies; nothing else depends on them, so this is the whole picture):

| Crate | Depends on |
|---|---|
| `solidcraft-ui-kit` | theme |
| `solidcraft-look` | ui-kit, theme |
| `solidcraft-ui-egui` | ui-kit, look, theme |
| `solidcraft` (the app) | look, theme, ui-egui |
| `solidcraft-web` | look, ui-egui |

`ui-egui` itself is MIT/Apache, not brand material, but it links all three brand crates, so using it
as it is would break §3a.

**Clean** (no brand crate, directly or through another crate): `kernel`, `engine`, `doc`, `sketch`,
`io`, `render`, `viewport`, `viewport-gpu`, `viewport-egui`, `control`, `control-egui`, `panel-layout`,
`ui-registry`, `gpu`, `mcp`, `mcp-server`, `craft-plugin*`, `solidcraft-cli` and all of `solidcore`.

## How much of `ui-egui` touches the brand crates

`ui-egui` is 467 files, 20.8 MB of Rust (`crates/ui-egui/src`). Uses of each brand crate:

| Crate | References | Files | Most used |
|---|---|---|---|
| `solidcraft_theme` | 699 | 200 (41 of them tests) | `Tokens` (428), `set_dark` (64), `is_dark` (45), `ColorVision` (33), `contrast_ratio` (26) |
| `solidcraft_look` | 31 | 14 | `icons`, `themes`, `pointer`, `compass`, `fonts`, `glyphs` |
| `solidcraft_ui_kit` | 25 | 6 | `app_bar`, `icons`, `history_bar`, `nav_bar`, `ribbon`, `prefs_window`, `start_page` |

The UI kit and the look are used in a few places, but the theme's `Tokens` run through 200 files.
De-branding a fork of `ui-egui` would mean rewriting styling in about 200 files and then keeping that
fork in step with an upstream that moves quickly (about 10,000 commits so far).

## What this means for MakerCAD

- **Keep D23: MakerCAD has its own shell** (`apps/mk-desktop`, our own `theme.rs`, icons and widgets)
  and builds on the clean crates above, starting with `viewport-egui`, `engine`, `doc` and `kernel`.
- `tools/brand_free.ps1` checks every build: none of `solidcraft-ui-kit`, `-look`, `-theme` or
  `-ui-egui` may appear in MakerCAD's dependency tree (passes: 193 packages, 2026-11-19).
- Reading `ui-egui`'s MIT/Apache code to learn how it drives the clean crates (camera, picking,
  commands, the control channel) is allowed. Copying styling, layouts or token values from the three
  brand crates is not.
- Doc 14's P1 names the replacements: `mk-theme` (tokens), `mk-look` (icons), `mk-ui-kit` (the widgets
  the shell needs). The first of each exists inside `apps/mk-desktop` today.

## Commands

```
# direct dependencies on the brand crates
Get-ChildItem third_party/solidcraft/crates, third_party/solidcraft/apps, third_party/solidcraft/solidcore/crates -Filter Cargo.toml -Recurse -Depth 2 |
  Select-String '^\s*solidcraft-(ui-kit|look|theme|ui-egui)\s*='
# uses inside ui-egui
Get-ChildItem third_party/solidcraft/crates/ui-egui/src -Recurse -Filter *.rs | Select-String '\bsolidcraft_theme\b'
# MakerCAD stays clean
pwsh tools/brand_free.ps1
```
