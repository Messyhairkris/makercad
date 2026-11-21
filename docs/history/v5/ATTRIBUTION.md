# Attribution and licence ledger

Release gate: this file must be current before anything ships. It is the v4 ledger's
successor, and it exists because the lamina decision (2026-11-12) proved how sharp the
line is.

## Our licence

MakerCAD v5 is **MIT OR Apache-2.0** unless the PM decides otherwise. Every dependency
below is compatible with that **on the stated linking model**. A row that says
"concepts only" means no code is copied and no dependency edge exists.

## Dependencies we link or bundle

| Dependency | Used for | Licence | Linking model | Notes |
|---|---|---|---|---|
| OpenCASCADE 7.9.3 | B-rep kernel | LGPL-2.1 + OCCT exception | Dynamic DLLs, user-swappable | Ship licence text and the exception notice. Named in `mc-kernel` only |
| planegcs (vendored, FreeCAD) | 2D constraint solver | LGPL-2.1+ | Own DLL (`makercad_solver`) | Source stays visible in the tree; user-swappable DLL is the relink path |
| Eigen (vendored with planegcs) | planegcs linear algebra | MPL-2.0 | Header-only | MPL file-level availability satisfied by the repo |
| Boost Graph/Math (header subset) | planegcs graph partitioning | BSL-1.0 | Header-only | Attribution in About |
| egui / eframe | UI | MIT OR Apache-2.0 | Linked | L5 only, enforced by `makercad layers` |
| wgpu | GPU backend | MIT OR Apache-2.0 | Linked | L5 only |

## Ideas adopted from other programs — no code copied

| Program | Licence | What we take | What we do **not** do |
|---|---|---|---|
| `tangency-labs/solidcraft` | MIT OR Apache-2.0 | the layer law, "everything is a command", the kernel ladder as a regression rig, the flat-pattern data model shape | we do not vendor SolidCore/SMLib; OCCT is our kernel |
| `storytold/cadcraft` | MIT OR Apache-2.0 | object-snap taxonomy and priority, command-line input model, grips, direct distance entry, DXF path | we do not take the drafting feature set into the default toolbox |
| `popupcad/code_popupcad` | MIT | the laminate/layer design model and cross-layer CSG as a modelling mode | we do not port its Python |
| `sbarrak/packbox` | MIT | the parametric dieline template table and the fold-slider interaction | we do not take the React/konva/three stack |
| `oriedita/oriedita` | MIT | flat-foldability and layer-order checks, crease-pattern model | optional module only; not on the maker path |
| `marcelfarres/lamina` | **AGPL-3.0** | **concepts only**: the five construction techniques (stacked, interlocked, radial, curve, folded), the assembly-check list with suggested fixes, the unfolding doctrine (spanning tree, union-area overlap test, strategies, compactness ranking, simplify-before-unfold), the cut/score/engrave export layers | **No code, no vendoring, no linking.** AGPL on a linked library would force this whole program to AGPL. Every line we ship is rewritten in Rust |

## Where the line is, stated once

- **MIT / Apache / BSD / MPL / BSL** → we may copy code with attribution.
- **LGPL** → we may link dynamically, never statically, and the user must be able to swap the library.
- **AGPL** → we may read it, learn it, cite it, describe its algorithms in our own words, and
  re-implement. We may **not** copy code, vendor it, or link it.

If anyone wants to cross a line, that is a PM decision and it changes this row first, not the code.
