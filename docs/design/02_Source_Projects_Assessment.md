# 02 — Source Projects Assessment

The six candidate projects, plus the kernel underneath two of them, as read on 2026-11-19. For each one: what it is, the stack, the licence, how mature it is, **what MakerCAD takes**, **how** we take it (fork, vendor, port or concept), and the risks.

## 2.0 The family picture

Four of the six are "Craft" apps that share a recognisable architecture:

- Rust, a Cargo workspace, a **down-only layer table enforced by `cargo xtask layers`**
- **egui** front end, **wgpu** renderer, often a CPU fallback
- **"Everything is a command"**: one registry drives the GUI, CLI, scripts and an **MCP server**
- wasm32 builds of everything below the UI

This is the same architecture v5 designed for itself (`mc-*` layers L0–L5, layer gate, command registry, MCP). The difference is that these projects have built it out, with thousands of commits. That is the strongest argument for forking rather than continuing v5.

| Project | Stack | UI | Kernel | Licence (code) | Brand restriction | Maturity signal |
|---|---|---|---|---|---|---|
| SolidCraft | Rust | egui + wgpu | solidcore (pure Rust B-rep) | MIT OR Apache-2.0 | **ui-kit, look, theme crates + name/logo under LICENSE-BRAND** | ~10.2k commits, 17k★, 75/76 reference parts pass; "early but usable for simple parts" |
| solidcore | Rust (safe, no `unsafe`) | — | is the kernel (port of NVIDIA SMLib) | Apache-2.0 | none stated | 0.0.x experimental; all SMLib modules ported; integration in progress |
| SketchCraft | Rust core + **React/three.js in Tauri** | web | own mesh/face model | MIT | trademarks (not SketchUp/Trimble-affiliated) | ~576 commits, young |
| CADCraft | Rust | egui + wgpu | 2D drafting geom | MIT OR Apache-2.0 | ArtCraft name/logo only; icons/fonts are original open work | ~1.7k commits, 8.3k★, "early development" |
| DraftCraft | Rust (fork of CADCraft) | egui + wgpu (+ CPU fallback), ribbon UI | 2D geom + `constraints` crate | MIT OR Apache-2.0 | DraftCraft name/logo/icon trademarks | 218 commits, young |
| PlanCraft | Rust | egui/eframe 0.36 + wgpu | — | **CC0 1.0** | none | 129 commits, "early preview" |
| SurfaceCraft | Rust | egui + wgpu (WebGPU/WebGL in browser) | solidcore | MIT OR Apache-2.0, no GPL deps | none stated | 747 commits, MVP M0–M6 done |

## 2.1 SolidCraft — the root

**What it is.** A parametric 3D CAD program in the Fusion 360 style. It has sketches with its own constraint solver, plus extrude, revolve, sweep, loft, fillet, chamfer, shell, draft, holes, threads, patterns, mirror, split and combine. It has a parametric timeline, unit-aware parameters and configurations, and assemblies with joints, motion studies, contact sets, interference checks and mass properties. It has sheet metal (flanges, hems, fold/unfold, flat patterns) and plastic-enclosure features. Imports and exports: STEP, IGES, 3MF, STL, OBJ, glTF, and URDF export. Experimental WASM-component plugins use WIT `craft:plugin@0.2.0`, with permission manifests and a single undo step for each plugin result. It ships as a desktop app, a browser app, a headless CLI and an MCP server.

**Architecture** (docs/architecture.md):

| Layer | Crates |
|---|---|
| L0 | `geom` |
| L1 | `sketch`, `kernel` (vendors solidcore), `render`, `dynamics` |
| L2 | `doc` (parameters, expressions, timeline, incremental evaluation) |
| L3 | `io` |
| L4 | `engine` (session + command registry; "everything is a command") |
| L5 | `gpu` (path tracer), `ui-egui`, `mcp` |
| shared | `color`, `ui-registry`, `ui-text`, `theme`*, `ui-kit`*, `look`*, `mcp-server`, `control`, `control-egui`, `viewport` (camera, CPU rasteriser), `viewport-gpu`, `viewport-egui` |
| other crates seen | `craft-plugin`, `craft-plugin-wasm`, `derive`, `develop`, `display-units`, `format`, `panel-layout`, `profile`, `smlib`, `space-mouse`, `testkit`, `threedx-client` |

\* = Brand Material (see 2.1.1).

The solidcore seam is `solidcore-ops` (modelling), `solidcore-tess` (display meshes) and `solidcore-record` (geometry as data). Every crate below the apps builds for wasm32.

**What we take: almost everything below the UI.** `geom`, `sketch`, `kernel`, `doc`, `io`, `engine`, `dynamics`, `render`, `mcp`, `mcp-server`, `viewport*`, `control*`, `ui-registry`, `ui-text`, `display-units`, `panel-layout`, `craft-plugin*`, `testkit`, `space-mouse`. Also `ui-egui` as the starting point for our shell. Before forking, check whether `ui-egui` itself depends on `ui-kit`; if it does, that dependency has to be replaced.

**How:** a hard fork, tracking upstream. See doc 03 §3.2 for the fork-and-track strategy: our additions live in new crates, and upstream crates get as few patches as possible so we can keep pulling.

### 2.1.1 The brand licence — the most important finding in this set

LICENSE-BRAND covers **code**, not just artwork:

- `crates/ui-kit`: controls, panels, menus, ribbon, window chrome (code and design)
- `crates/look`: icon masters, code-drawn glyphs, built-in themes
- `crates/theme`: design tokens, palettes, accent ramps, sizes, spacing
- the name, logo, app icon and brand kit

Without written permission, these cannot be **distributed** except as part of SolidCraft, cannot be **linked from non-SolidCraft software**, and a fork cannot ship **"with a look that could be taken for SolidCraft's"**.

**Consequence:** MakerCAD must have its own `mk-ui-kit`, `mk-look` and `mk-theme`, written from scratch (clean room; do not copy structure from the brand crates). This is work, but it is work we wanted: doc 11 describes a deliberately different, storybook-pastel look. It also keeps MakerCAD from looking like a reskinned SolidCraft.

**Optional:** ask Tangency Labs in a SolidCraft issue (their stated process) whether the *structure* of `ui-kit` (the widget set, without its look) can be used. Don't plan on a yes.

**Risks.** solidcore is 0.0.x (see 2.2). The project moves very fast (5k+ issues, ~1k PRs), so tracking upstream is a standing cost. The egui version has to stay aligned with the other Craft crates we vendor.

## 2.2 solidcore — the kernel

A safe-Rust port of NVIDIA's SMLib (via USD BRep, Apache-2.0). Exact B-rep over analytic and NURBS geometry. In scope: NURBS maths, intersections, booleans, fillets and chamfers, offset and shell, sweeps and skins, primitives, healing and stitching, mass properties. Out of scope: OCCT translators, USD and Python bindings. It checks for wasm32 in CI. It is a **read-only mirror of SolidCraft's `solidcore/` folder**; issues and PRs go upstream to SolidCraft.

**Decision:** solidcore replaces OCCT as MakerCAD's kernel. Doc 03 §3.4 has the reasoning and the escape hatch.

**Risk:** "deferred paths still limit coverage." Fillets on hard edge chains, and booleans on near-tangent faces, are the usual places a young kernel fails. Mitigation: MakerCAD's maker-path geometry is dominated by boxes, extrusions, holes, slots and simple fillets, which the 75/76 parts show working. We add a **maker-part regression corpus** of 50 typical furniture, packaging and fix-it parts to CI and report kernel failures upstream.

## 2.3 SketchCraft — the "Quick Build" way of modelling

**What it is.** A SketchUp-like modeller: draw an outline, then pull it up. It has push/pull, lines that cut faces, arcs, circles, offset, Follow Me, move, rotate, scale, copy; typed exact dimensions and inference snapping; groups, components, tags and an outliner. Imports: `.skp`, DXF/DWG, IFC, FBX, OBJ and **point clouds**. Exports: glTF, OBJ, STL, DXF, IFC, PDF, PNG, MP4. Section cuts, floor plans, sun studies, and a **cut-list plugin that estimates boards and sheets to buy**. Every action is a JSON command, plugins are sandboxed JS, and there is an MCP server.

**Stack mismatch.** The UI is **React and three.js inside Tauri**, not egui. We cannot embed its UI in an egui shell.

**What we take:**
- **The interaction model**: push/pull, inference snapping (on-axis, on-edge, midpoint, parallel, perpendicular colour cues), typed values in the measurement box, "draw on face", components. This becomes the **Quick Build environment** (doc 06 §6.3), re-implemented in egui over *our* document. Quick Build operations become ordinary timeline features (push/pull = an extrude or offset-face feature), so a beginner's model can be refined later in Solid without conversion.
- **Code from its Rust `core/`** (MIT), where it is separable: the `.skp` importer, point-cloud import and the cut-list estimator logic. Vendor these as crates behind our `io` and `stock` layers.
- **Its tool vocabulary and shortcuts** (SketchUp-familiar), as an optional keymap preset: "SketchUp-style" alongside "Fusion-style" (SolidCraft default) and "AutoCAD-style" (CADCraft).

**Risk:** if its core is entangled with its own face model, port the algorithms rather than the crates. Its licence lets us do either.

## 2.4 CADCraft and DraftCraft — the drafting brain for the sketcher

**CADCraft:** a 2D/3D drafting app aimed at the AutoCAD workflow. Command line, object snaps (endpoint, midpoint, centre, tangent and more), polar tracking, ortho, grid snap, direct distance entry, hot grips (Space cycles stretch, move, rotate, scale, mirror). Draw: LINE, PLINE, CIRCLE, ARC, RECTANG, POLYGON, ELLIPSE, SPLINE, POINT, XLINE, RAY, DONUT, TEXT, MTEXT. Modify: ERASE, MOVE, COPY, ROTATE, SCALE, MIRROR, OFFSET, TRIM, FILLET, CHAMFER, ARRAY and more. Parametric constraints with AUTOCONSTRAIN, PARAMETERS with expressions, and conflict detection. Native DXF read/write and DWG via acadrust. Crates: `geom`, `dxf`, `color`, `doc`, `fonts`, `render`, `io`, `engine`, `ui-egui`, `mcp`. "Nothing below `ui-egui` knows about egui."

**DraftCraft:** a fork of CADCraft that adds a **ribbon UI**, a **`constraints` crate**, its own **`dwg`** crate, **CONIC** arcs (start, apex, end and rho; saved as a rational spline), revision clouds, wipeouts, stretch, break, join, explode, multileaders, tables, blocks with attributes, paper-space layouts, wgpu with a CPU fallback, and copy-on-write undo. Every command runs behind a guard that restores the drawing on failure.

**What we take:**

| Take | From | Into |
|---|---|---|
| Modify algorithms: trim, extend, offset, fillet, chamfer, break, join, stretch, array (rect, polar, path) | CADCraft/DraftCraft `geom` + `engine` | `mk-sketch-tools`, which operates on SolidCraft sketch entities |
| Object snap and polar tracking engine | CADCraft | `mk-sketch-tools::snap` |
| Grips with Space-cycling modes | CADCraft | Sketch environment interaction |
| CONIC tool | DraftCraft | Sketch tool (important for packaging curves and soft-goods panels) |
| AUTOCONSTRAIN and conflict detection ideas | CADCraft/DraftCraft `constraints` | Compared with SolidCraft's `sketch` solver; adopt whichever wins on the test corpus, and keep one solver |
| DXF/DWG read/write | `dxf`, `dwg` crates | `io` (essential for laser, plotter and die-cut output) |
| Paper-space layouts, dimensions, multileaders, tables, title blocks | DraftCraft | **Drawings environment** (blueprints, doc 06 §6.11) |
| Command-line UX and the command guard (restore on failure) | both | Sketch command line; the guard pattern applies engine-wide |
| Hatch patterns, linetypes, single-stroke font (original work, open) | CADCraft | Drawings, cut templates, engraving text for lasers |
| Appearance cues | DraftCraft "is closer in appearance to what we want" (PM) | Treat this as layout reference only: ribbon density and palette placement. **Not** its identity; see doc 11. |

**How:** vendor the needed `geom` / `dxf` / `dwg` / `constraints` crates at a pinned commit, behind adapter crates. Port the modify algorithms onto SolidCraft's sketch entity types. Don't run two sketch models side by side.

**Risk:** CADCraft is 2D-drawing-centric (DXF entities). SolidCraft sketches are constraint-driven profiles on planes. The adapter maps one to the other: the drafting tools produce sketch entities, and auto-constraints are inferred. This is the main engineering task of the Sketch environment.

## 2.5 PlanCraft — the planning engine

**What it is.** A desktop project planner. Tasks, subtasks, links with lead and lag, summary tasks, deadlines, critical path, recurring and split tasks, three-point estimates, milestones with deliverables, and resources (people, teams, vendors, contractors, **equipment**, each with rates), calendars and holidays, baselines, 13 views (Gantt, network, board, workload, calendar...). MS Project XML and CSV/XLSX import. An MCP server with 92 tools, where agent edits are marked ⚡ and can be undone. Atomic saves, recovery copies.

**Architecture:** `plancraft-core` (pure logic: data model, CPM scheduler, calendars, edits, undo, import/export), `plancraft-mcp`, `plancraft-server`, `plancraft-app` (egui/eframe 0.36 + wgpu). **CC0**, so there are no licence obligations at all.

**What we take:**
- **`plancraft-core` as a dependency**, wrapped by `mk-plan`. The CPM scheduler, calendars, estimates and baselines are exactly right.
- Its resource model maps onto MakerCAD: **people** = the maker plus helpers; **equipment** = items from the tool inventory (the table saw, the 3D printer, the borrowed sewing machine with "available Saturdays" as its calendar); **cost rates** = material costs and tool rental.
- A subset of its views, redrawn in our UI kit: **Timeline** (Gantt), **Board** and **Calendar**. The other ten stay hidden unless the project grows.
- Its MCP tool set is merged into MakerCAD's MCP namespace under `plan.*`.
- Patterns: ⚡ marking of agent changes, and atomic save with a recovery copy. We adopt both engine-wide.

**Key integration:** tasks are not typed by hand. They are **generated from the design** (doc 10): cut, drill, glue-up, cure, sew, fold and finish steps come from features and stock assignments, with durations from tool and technique rates. The user then edits the schedule like any PlanCraft plan.

**Risk:** egui version alignment (0.36 here). We only use the core crate, which has no UI dependency, so this is low risk.

## 2.6 SurfaceCraft — scan to CAD

**What it is.** Clean-room reverse engineering on **solidcore**, the same kernel as SolidCraft. Mesh import (STL, OBJ, PLY); cleaning (hole fill, spike and debris removal, QEM decimation, Taubin smoothing, noise estimate); alignment (3-2-1 datums, N-point Kabsch, point-to-plane ICP, world axes); automatic region segmentation; plane, cylinder, cone, sphere and torus fits with automatic type detection and symmetry planes; NURBS patch fitting and AutoSurface; a constraint sketcher with sections cut from the mesh; solid features; colour deviation maps. STEP and IGES out. Accuracy claims are CI-tested (e.g. ICP RMS 0.015 mm at 0.02 mm noise; primitive auto-type 5/5).

**What we take:** the clean, align, segment, fit and deviation pipeline as the core of the **Space and Scan environment** (doc 09), plus **as-built compare**. Because the kernel is shared, fitted planes and cylinders become native MakerCAD geometry with no translation step.

**Gaps we fill:**
- **Point clouds** (no mesh): add PLY/LAS/E57/XYZ ingestion with voxel downsampling, normal estimation and RANSAC plane and primitive detection operating on points.
- **Gaussian splats** (3DGS `.ply` / `.splat` / `.spz`): treat splat centres, filtered by opacity and scale, as a weighted point cloud for measurement and plane fitting. Render the splat itself only on capable hardware (doc 09 §9.3).
- **Room semantics**: walls, floor, ceiling, openings (doors and windows), wall-mounted rectangles (frames, switches), and furniture boxes. This is a MakerCAD layer above SurfaceCraft's primitives.
- **Scale calibration** for photo-only or unscaled meshes.
- Performance for million-triangle scans is still on SurfaceCraft's roadmap (P2). We stream and decimate on import in the meantime.

## 2.7 Summary: fork, vendor, port or concept

| Project | Relationship | Effort | Value |
|---|---|---|---|
| SolidCraft | **Fork** (root), minus brand crates | High (ongoing tracking) | Highest |
| solidcore | Comes with the fork | — | Removes OCCT |
| CADCraft / DraftCraft | **Vendor** geometry/IO crates; **port** modify tools and snaps | Medium-high | Sketch quality, DXF/DWG, drawings |
| PlanCraft | **Depend** on `plancraft-core` | Low-medium | Schedules and resources |
| SurfaceCraft | **Vendor** pipeline crates (same kernel) | Medium | Scan to space |
| SketchCraft | **Port** the interaction model; vendor `.skp` import and the cut-list logic | Medium | Beginner on-ramp |

## 2.8 Other open-source pieces worth knowing (not requested; flagged for later)

- **truck** and **Fornjot**: other Rust B-rep kernels. Fornjot is no longer developed. Neither is needed while solidcore holds.
- Any **nesting** library: we write our own 1D (lumber) and guillotine/rectangle nesters (doc 08). True-shape nesting for lasers is a later environment; check the licence of any library first.
- **Slicing**: a CURA-like environment would sit as a **sidecar** around an existing slicer engine, not be reimplemented (doc 06 §6.13). Licences vary (many slicers are AGPL), so the sidecar boundary matters.
