# 14 — Roadmap

Phases are ordered by **risk retired** and **value delivered to one real maker**. Every exit criterion is run on the potato and recorded in STATUS with its command. There are no calendar dates, because pace depends on the team. Each phase states its rough size.

The rule against v5's risk F3 ("breadth before depth") is that **no phase starts until the previous phase's exit criteria are in STATUS**. Parallel work is allowed only on items marked ‖.

## P0 — Prove the foundation (small)

Goal: confirm the fork is viable on the floor machine before investing.

- Build stock SolidCraft (tagged release) on the potato. Run it. Record: does it start? Which wgpu backend? Frame time orbiting a reference part? Idle RSS?
- If it crashes on Vulkan (likely, per v5): find SolidCraft's backend selection, force DX12/GL and record the result. File an upstream issue.
- Run SolidCraft's 76 reference parts on the potato. Time the rebuilds.
- Read: `ui-egui` dependencies on `ui-kit`/`look`/`theme` (how much of the shell must we replace?); the WIT `craft:plugin` world; the `doc` extension options.
- Build CADCraft/DraftCraft and PlanCraft on the potato and record start times.

**Exit:** SolidCraft runs on the potato at ≥ 30 fps orbit on a reference part with a forced backend; a written brand-crate dependency map; a GO/NO-GO note. **NO-GO triggers:** solidcore can't rebuild typical maker parts, or SolidCraft can't be made to run on the potato with reasonable patches. The fallback plan is in doc 15 (D2 reversal).

## P1 — Fork, de-brand, MakerCAD shell (medium)

- Create the `makercad` fork. Add `xtask brand-free`. Remove the brand crates.
- `mk-theme` (Daydream + Pastel + High Contrast tokens), `mk-look` (first 60 icons), `mk-ui-kit` (the widgets the shell needs).
- `mk-desktop`: opening screen, project bar, environment tabs, status bar, command search.
- `mk-probe` with child-process backend tests; profile file; backend selection wired into the viewport.
- `.maker` container v1 (manifest, intent, one design, journal stub), with atomic save and recovery.
- Sancho as a **scale figure** (D27): a still figure posed by code, standing at true size beside the design in the Design workspace, and easy to hide.

**Exit:** a de-branded MakerCAD opens on the potato in < 2 s to the opening screen; SolidCraft modelling works inside our shell; the `brand-free`, `layers` and `wasm` gates are green; the probe picks DX12/GL correctly on the potato and never crashes the app.

## P2 — Project layer: Quest, inventory, Sancho rules (medium-large)

- `mk-inventory` (SQLite library, kits, quick-add parser, taxonomy v1 with ~60 tool kinds and ~80 materials).
- The Quest (questions as data), interpreter rules v1, the review card, scale classes and stages.
- `mk-assist` rules core + structure + tools packs; Findings panel; persona TOML; **Sancho as a measuring stick** (D27): the user picks the figure (placeholder, own model or a rigged scan) and its height, poses it, and reads its height, hand and finger lengths against the design. *Moved to the end of the plan by D27:* GPU skinning, look-at and point-at IK, crossfaded clips (motion capture), VAT and portrait tiers.
- Journal v1: intent, decisions, session interview, open pages.

**Exit:** a new user goes from "Start something new" to a project with tools, materials, scale and toolbox in < 5 min (usability test, n ≥ 5); Sancho's figure on the potato measured (hidden = 0 ms), and his height and hand lengths read correctly against a part; the golden dresser has 5 seeded faults flagged with 0 false flags.

## P3 — Stock & Cut: the differentiator (medium)

- `mk-stockbind`, stock-aware features (board/panel from stock, joinery v1: butt, dado, rabbet, pocket screw, dowel, cam lock).
- `mk-nest` 1D + guillotine 2D, kerf, grain, defects; the rack view; sketch-on-stock.
- Material meter, waste views, usable offcuts back to inventory, shopping list.
- Fit solver strategies 1–4 and 9.

**Exit:** the doc 08 §8.4 example reproduces exactly (11 boards, 9.3% true waste); a 60-part dresser meter updates in < 300 ms on the potato; manual and automatic layouts mix; golden cut-diagram PDFs pass.

## P4 — Sketch upgrade ‖ Quick Build (medium-large; two tracks)

- **Track A, Sketch:** vendor CADCraft/DraftCraft `geom`/`dxf`/`dwg`/`constraints`; port trim, extend, offset, fillet/chamfer, break, join, stretch, arrays; osnaps, polar tracking, dynamic input, grips; conic; line types with manufacturing meaning; trace image.
- **Track B, Quick Build:** push/pull, inference engine, measurement box, groups/components, paint with stock. All emitting timeline features.

**Exit:** v5 M2's bracket test (sketch, extrude, fillet, hole in < 10 min for a new user); the ported `sketch_shapes` cases pass; a Quick Build model opens in Solid with editable features; DXF round-trips through a common laser workflow.

## P5 — Plan + Output (medium)

- `mk-plan` over `plancraft-core`: step derivation, ordering rules, rates, calendars, Timeline/Board/Calendar views, re-plan with preserved edits.
- Drawings environment (DraftCraft paper space), the export pack, tiled PDF templates with test squares, build guide HTML/PDF.
- Projector mode with homography calibration.

**Exit:** doc 10 §10.8 acceptance lines; the golden dresser's plan respects cure lags and work windows; a stranger builds the battery cover from the HTML guide.

## P6 — Space & Scan (large)

- Vendor the SurfaceCraft pipeline; staged import; point-cloud ingestion; the plane-detection room shell; openings; wall-mounted items; furniture boxes; RoomPlan-type import; calibration references and mat; splat ingestion (points first, rendering on modest+); the measurement probe; as-built compare.

**Exit:** doc 09 §9.10 acceptance lines.

## P7 — Fold & Package (medium-large)

- `mk-fold`: dieline graph, line types, hinge graph, kinematics with collision checks, caliper allowances, templates (FEFCO 0201, ECMA reverse tuck, mailer, sleeve, tray), flaps toolkit, artwork mapping, dieline export layers, press-sheet nesting; lamina-style techniques (concepts only).

**Exit:** v5 M3's fold criteria (0 penetrations at 0–100%); the golden candle box plotter SVG cuts and folds correctly in a physical test; the print-shop PDF passes a prepress check.

## P8 — Soft Goods ‖ Motion (large; two tracks)

- **Soft Goods:** `mk-pattern` flattening + distortion map, seams, allowances, notches, hardware library, fabric nesting, sewing order.
- **Motion:** named states, path clearance sweep, counterbalance helper, pinch and tip-over rules (on SolidCraft assemblies + dynamics).

**Exit:** the golden backpack's pattern sews into the modelled shape within ± 1 cm (physical test); the golden fold-bed transitions are collision-checked, and the strut sizing agrees with a hand calculation within 5%.

## P9 — Publish, the Mind, MCP polish (medium)

- Build-log publisher with privacy pass; "build from this" re-fit on open; share bundles.
- The Mind: provider config, MCP client, ⚡ proposals, budget meter, the zero-network test.
- `mk-mcp`: full namespace set, including `plan.*` from PlanCraft.

**Exit:** the no-network gate is green with the Mind off; Rules ≥ 60% of findings with the Mind on; a published golden log reads well to 3 outside readers.

## P10+ — Environments beyond

In order of maker value: Laser/Plotter (true-shape nesting), 3D Print Prep (sidecar), CNC/CAM (2.5D), Render & Present, Electronics (enclosure-focused), Generative (sidecar), Fluids (sidecar), workflow packs (VFX, game dev, classroom), browser build (wasm), remote sidecars.

## 14.6 What to do with v5

| v5 item | Action |
|---|---|
| Workspace crates `mc-*` | **Archive** (read-only tag `v5-final`). Do not port wholesale |
| `mc-view` camera clamp tests | Port as tests against the SolidCraft camera, if behaviour matches the desired feel |
| `xtask-layers` test cases | Contribute any useful cases to SolidCraft's layer gate (upstream PR) |
| `sketch_shapes` cases | Port to P4 Track A tests |
| `mc-kernel` OCCT shim | Keep in the archive as the seed of the optional OCCT exchange sidecar (doc 03 §3.4) |
| STATUS discipline, DECISIONS format, the probe idea, Sancho/log contracts | **Carried forward** (this document set) |
| `makercad_v5_backup_2026-11-13_pre-pass2` | Can be deleted once the `v5-final` tag is confirmed (PM decision) |
| v4, Godot, LoCAD/RustCAD/FoldCAD/ThreeJS learning projects | Reference only; nothing carried |
