# Third-party notices

MakerCAD is MIT OR Apache-2.0. These are the third-party works that ship inside its programs.
Add a row in the same change that adds a bundled work.

## In `makercad` (apps/mk-desktop)

| Work | Where it comes from | Licence | Licence text |
|---|---|---|---|
| egui, eframe, egui-wgpu, egui-winit, epaint, emath, ecolor (0.37) | vendored in SolidCraft (`third_party/solidcraft/vendor/egui`), by the egui authors | MIT OR Apache-2.0 | `vendor/egui/LICENSE-MIT`, `LICENSE-APACHE` |
| winit (0.30, SolidCraft's vendored copy) | `third_party/solidcraft/vendor/winit` | Apache-2.0 | `vendor/winit/LICENSE` |
| Hack Regular (egui's monospace font) | Source Foundry; DejaVu parts public domain | MIT (Bitstream Vera parts under their own licence) | `epaint_default_fonts/fonts/Hack-Regular.txt` |
| Ubuntu Light (egui's proportional font) | Canonical | Ubuntu Font Licence 1.0 | `epaint_default_fonts/fonts/UFL.txt` |
| Noto Emoji | Google | SIL Open Font Licence 1.1 | `epaint_default_fonts/fonts/OFL.txt` |
| emoji-icon-font and its subset egui-icons | John Slegers | MIT | `epaint_default_fonts/fonts/emoji-icon-font-mit-license.txt` |
| wgpu, naga and the other crates.io dependencies | crates.io | MIT and/or Apache-2.0 (each crate's own) | `cargo tree` lists them; a generated list comes with the first release |

## Assets

| Work | Where it comes from | Licence | Where it is |
|---|---|---|---|
| BodyKun, Sancho's placeholder figure (decision D26) | "BodyChan – BodyKun, Posable Drawing Figures" by vinchau, BlendSwap 23521 (blendswap.com/blend/23521); file provided by Kris 2026-11-20, sha256 `d01efc62…83af` | **CC0 1.0** (public domain dedication; no attribution required, given anyway) | `crates/mk-sancho/assets/sancho-bodykun.glb`, rebuilt by `tools/blender/export_bodykun.py`: only the 71 deform bones, the 12 body meshes joined, our wood material, scaled to 1.68 m, our own Wave and Grip |

Not bundled: SolidCraft's brand-licensed crates (`ui-kit`, `look`, `theme`, `ui-egui`), its name
and its logo. `tools/brand_free.ps1` checks that they never enter our dependency graph.

## Drawn or written for MakerCAD (no third-party rights)

- Sancho on the opening screen: drawn from code (`apps/mk-desktop/src/sancho.rs`).
- The MakerCAD mark: drawn from code (`apps/mk-desktop/src/home.rs`).
- Icons drawn by the Crew from doc 18 (`crew/contracts/mk_icons.md`); reviewed on a contact
  sheet against `reference/` for borrowing before they land.
