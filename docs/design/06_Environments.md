# 06 — Environments

An **environment** is a mode of working with its own tools, panels, overlays, document data and Sancho rules. Environments are modules: the project declares what it needs, the host initialises them on demand, and they can be unloaded. Mechanics are in doc 03 §3.7.

## 6.0 Catalogue at a glance

| # | Environment | Kind | Source | Default load | Cost (RAM, idle → active) | Min profile |
|---|---|---|---|---|---|---|
| 6.1 | **Sketch** | built-in | SolidCraft `sketch` + CADCraft/DraftCraft tools | Always with Solid | 5 → 30 MB | potato |
| 6.2 | **Solid** | built-in | SolidCraft (solidcore) | Most projects | 0 → 80–250 MB | potato |
| 6.3 | **Quick Build** | built-in | SketchCraft interaction model, our code | Beginners, room-scale | 0 → 40 MB | potato |
| 6.4 | **Space & Scan** | built-in | SurfaceCraft pipeline + mk-scan | When a scan or space exists | 0 → 150 MB–2 GB (scan-size dependent, streamed) | potato (points/mesh), modest (splat render) |
| 6.5 | **Stock & Cut** | built-in | mk-stockbind, mk-nest, SketchCraft cut-list logic | Whenever materials exist | 0 → 30 MB | potato |
| 6.6 | **Fold & Package** | built-in | mk-fold, SolidCraft sheet metal, lamina concepts | Packaging, papercraft, carton | 0 → 40 MB | potato |
| 6.7 | **Soft Goods** | built-in | mk-pattern | Sewing, textiles | 0 → 60 MB | potato |
| 6.8 | **Motion & Physics** | built-in | SolidCraft assemblies + `dynamics` | Moving parts, fold-away furniture | 0 → 50–200 MB | potato (kinematic), modest (dynamic) |
| 6.9 | **Plan** | built-in | plancraft-core | Every project (lightweight) | 0 → 15 MB | potato |
| 6.10 | **Journal** | built-in | mk-log | Every project | 2 → 10 MB | potato |
| 6.11 | **Drawings** | built-in | DraftCraft paper space, dims, tables | On first blueprint | 0 → 40 MB | potato |
| 6.12 | **Output** | built-in | mk-export | On export | 0 → 50 MB | potato |
| 6.13 | 3D Print Prep | sidecar | an existing slicer engine | Optional | separate process | modest |
| 6.14 | Laser / Plotter / Vinyl | built-in, later | mk-nest true-shape + DXF/SVG | Optional | 0 → 30 MB | potato |
| 6.15 | CNC / CAM | WASM or sidecar, later | toolpaths | Optional | separate | modest |
| 6.16 | Electronics | WASM, future | schematic → enclosure cutouts | Optional | — | modest |
| 6.17 | Generative / Lightweighting | sidecar, future | topology optimisation | Optional | separate | capable |
| 6.18 | Fluids / Airflow | sidecar, future | CFD | Optional | separate | capable |
| 6.19 | Render & Present | built-in (SolidCraft `gpu` path tracer) | — | Optional | 0 → 300 MB+ | modest |
| 6.20 | Projector | built-in | mk-export + calibration | When a projector is in inventory | 0 → 20 MB | potato |

Cost numbers are **budgets to verify**, not measurements. Each gets a STATUS row once measured on the reference machine.

**Always present and not environments:** the project shell, Intent card, Inventory drawer, Material meter, Sancho, command search, history, Plan mini-bar and Journal button. They are small, and they are what make MakerCAD feel like one program.

## 6.1 Sketch

**Purpose:** 2D profiles on planes or faces; the source of most solid, fold, pattern and cut geometry. It is also a **"design flat"** workspace for people who think in 2D (packaging, patterns, laser parts).

**Base:** SolidCraft's sketch (lines, arcs, splines, text, its own solver, parametric dimensions).

**Added from CADCraft/DraftCraft** (`mk-sketch-tools`):
- **Draw:** polyline with arc segments, rectangle variants (2-point, 3-point, centre), polygon, ellipse, slot (straight, arc), **conic** (rho), points, construction lines and rays, text (single-stroke font for engraving).
- **Modify:** trim, extend, offset (with chain), fillet and chamfer between entities, break, join, stretch, mirror, rectangular, polar and path arrays, explode, scale and rotate by reference.
- **Precision:** object snaps (endpoint, midpoint, centre, quadrant, intersection, perpendicular, tangent, nearest, extension, parallel), polar tracking, ortho, grid snap, **dynamic input** (type length and angle at the cursor), **direct distance entry**.
- **Grips**: hot grips with Space-cycling (stretch → move → rotate → scale → mirror).
- **Command line** (optional, AutoCAD-style), always paired with the toolbox.
- **Auto-constrain**: inferred constraints from the snaps used (horizontal, coincident, tangent, equal), shown as chips you can delete; conflict detection explains *which* constraints fight.

**MakerCAD-specific:**
- **"Sketch on stock"**: sketch directly on a board or sheet from inventory (it appears as a sketch plane with real outline, grain arrows and defects). This is the "draw cuts onto the boards" workflow from the brief.
- **Trace image**: calibrated reference images (from the mood board or a photo with a scale reference) under the sketch, with optional auto-trace to polyline and fit to arcs.
- **Sketch from space**: project room-shell edges, wall outlines or scan sections into the sketch as reference geometry.
- **Line types with meaning** (shared with Fold and Soft Goods): cut, crease, perforate, score, half-cut, stitch, fold-valley, fold-mountain, bleed, grain. Each has a manufacturing meaning used on export.

**Minimum viable version:** SolidCraft sketch plus trim, extend, offset, fillet, osnaps, dynamic input and sketch-on-stock.

## 6.2 Solid

**Purpose:** exact parametric 3D: SolidCraft's full feature set (doc 02 §2.1), with timeline and parameters.

**MakerCAD additions:**
- **Stock-aware features**: "Panel from stock" (pick a sheet: thickness and grain come from the item; the panel is bound), "Board from stock" (pick a board: the profile is the real section; length is the parameter), and "Hole for hardware" (pick a part from inventory: clearance holes sized from the part).
- **Joinery features** (maker joints as timeline features): butt, rabbet, dado, groove, half-lap, finger/box joint, dovetail (hand-tool-friendly angles), mortise and tenon, pocket screw, dowel, biscuit, cam lock, tab-and-slot (for laser and CNC). Each joint knows which **tool capabilities** it needs (doc 04), so the joint menu sorts by "you can do this".
- **Drawer and door wizards**: parametric sub-assemblies (box, runners or slides, clearances from the slide's datasheet).
- **Clearance and fit** as first-class: `fit` relations between bodies or between a body and a Space ("3 mm each side"), checked live.
- **Two styles of timeline display**: technical (feature names) and **maker** (log templates: "Cut a dado 6 mm deep for the bottom panel").

## 6.3 Quick Build

**Purpose:** the SketchUp-style on-ramp. Draw a shape on any face or the floor, push/pull it, type a number, done. It is the quickest way to rough in furniture against a room.

**Behaviour** (re-implemented after SketchCraft's model):
- Tools: line, rectangle, circle, arc, push/pull, offset, move/copy, rotate, follow-me, tape measure, protractor, paint (material from inventory).
- **Inference engine** with coloured cues (red, green and blue axes; magenta for parallel/perpendicular; on-edge, on-face and midpoint tooltips) and inference locking (Shift).
- **Measurement box** (type `600`, `600,400`, `2'6"`) after any operation.
- Groups and components (repeated drawer fronts update together).

**The key difference from SketchCraft: everything is parametric underneath.** A push/pull writes an extrude or face-offset feature to the timeline, and a typed value becomes a named parameter if the user wants. A Quick Build model is a normal Solid model; switching environments is just switching tools. Beginners never hit a wall where they must "convert" or "start over in real CAD".

**Stock-aware Quick Build:** painting a face with "pine 2x4" snaps its thickness to 38.1 mm and binds it to stock.

## 6.4 Space & Scan

Full detail in doc 09. Summary: import a scan or measurements; clean it; detect walls, floors, openings, frames, furniture; produce a **room shell** of clean editable geometry plus the original scan as a ghost overlay; calibrate scale; measure with a probe; compare as-built against design.

## 6.5 Stock & Cut

Full detail in doc 08. Summary: the inventory as a visual rack of real boards and sheets; live binding of bodies to stock; automatic and manual layout of parts onto stock (1D for boards, 2D for sheets and fabric); kerf, grain and defect handling; waste visualisation; shopping list; **"make it fit what I have"** suggestions.

## 6.6 Fold & Package

**Purpose:** production-grade dielines for packaging and papercraft. You can design **assembled (3D, then add cut lines)** or **flat (sketch the dieline, then fold it)**, and both stay in sync.

**Model** (`mk-fold`):
- A **dieline** is a planar graph of **panels** (faces) and **edges** typed as cut, crease (valley/mountain), perforation, cut-crease (combo), half-cut (kiss), or glue-flap edge.
- A **hinge graph** (panels as nodes, creases as hinges) with fold angles: **fold kinematics** animates 0–100% and checks panel collisions (v5 exit criterion: 0 penetrations).
- **Board model**: material caliper (thickness), flute direction for corrugated, grain direction for folding cartonboard. Fold allowances are computed from caliper: inner panels shrink by about 1× caliper per fold for tuck boxes, and corrugated uses the standard joint allowances. Shown as an explicit table so experts can override.

**Two workflows:**
1. **3D first:** model the box (or pick the product, and the box cavity grows around it with clearance), then click edges to mark *fold*, *cut* or *glue*. MakerCAD unfolds with the doc 01 doctrine (spanning tree on the dual graph, union-area overlap test), adds glue flaps and tuck flaps from rules, and shows the dieline beside the 3D view, live.
2. **Flat first:** draw the dieline in Sketch with typed line types, then fold. The 3D preview folds it and flags panels that don't close, overlaps and gaps.

**Library of parametric templates:** FEFCO (0201 RSC, 0203, 0427 mailer…), ECMA (A20.20 reverse tuck end, straight tuck, auto-bottom/crash-lock, sleeve), pillow box, gable box, tray with lid, display stand, inserts (egg-crate dividers via the "interlocked" technique). All driven by the content's dimensions plus clearance.

**Flaps and features toolkit:** dust flaps, tuck flaps with friction locks (slit locks), auto-lock bottoms, hang tabs (euro slot), windows (with film overlap), thumb notches, tear strips (perforation patterns with cut:tie ratio), handles, inserts.

**Artwork layer:** import artwork (SVG/PDF/PNG) mapped onto panels in 3D; bleed (3 mm default) and safe zone overlays; export dieline + art for the printer.

**Production output** (via Output): DXF/PDF with **standard dieline layers** (CUT, CREASE, PERF, BLEED, DIMENSIONS, ART) with spot-colour naming as print shops expect; cutting-plotter SVG; laser files; a 1:1 tiled print for hand cutting; and nesting multiple dielines on a press sheet for quantity runs.

**Lamina-style techniques** (concepts only, AGPL source not used) live here and in Stock & Cut: stacked slices, interlocked egg-crate, radial, curve ribs, folded nets (papercraft from any mesh, with numbered joint tabs).

## 6.7 Soft Goods

**Purpose:** design textile objects in 3D and get sewing patterns and an ordered sewing plan.

**Workflow:**
1. **Envelope first** (optional): lay out the contents (doc 13 §13.3): photos of items with a scale reference turned into boxes or extruded outlines, or simple primitives. These form an `envelope` Space, and the bag must contain it with clearance.
2. **Model the shape** in Solid or Quick Build (boxy bags), or with a **panel modeller**: draw seam lines on a 3D surface to split it into panels.
3. **Flatten panels** (`mk-pattern`): developable panels unfold exactly; doubly-curved panels flatten with LSCM/ARAP and a **distortion map** (colour = stretch or compression %). This is checked against the fabric's stretch (woven nylon ≈ 0%, so curvature has to be resolved with darts or more seams; Sancho suggests a dart where distortion > threshold).
4. **Seams:** each seam is an edge pair with a type (plain, French, flat-felled, bound, top-stitched, welded), **seam allowance** (default 10 mm / ⅜″, per seam), notches (auto at seam matching points), and stitch type and length.
5. **Hardware:** zippers (length, gauge, coil vs. Vislon), buckles (webbing width), D-rings, grommets, Velcro, elastic, piping, foam padding (as thick "fabric" layers), tubing (hydration port). All from the parts library.
6. **Pattern output:** each piece with grain line, cut quantity ("cut 2, mirror"), fabric, notches, labels, seam allowance line plus stitch line, and a 1:1 tiled PDF (A4/Letter with registration marks and a test square), A0/plotter PDF, and **projector mode** (doc 10 §10.6). Nested layout on the fabric width (`mk-nest`, roll form) with grain constraints.
7. **Sewing order:** the seam graph plus rules ("attach pockets before closing the panel", "install zipper before side seams", "bind raw edges last") produce a sewing sequence that feeds Plan and the build instructions.

**Rubber, tubing, foam:** the same panel model, with material behaviour flags (glue or weld instead of sew; no seam allowance for butt-welded joints).

## 6.8 Motion & Physics

**Purpose:** moving parts and **multi-state designs**: fold-away furniture, sharpener mechanisms, lids and hinges.

**Base:** SolidCraft assemblies (joints, motion studies, contact sets, interference, mass properties) and `dynamics` (rigid bodies, joints, forces, integration).

**MakerCAD additions:**
- **Configurations as named states**: "Bed down", "Craft table out", "Living mode". A state slider animates between them, and clearance is checked **along the whole path** (swept volume against the room shell, other furniture and keep-outs such as the door swing). The fold-away bedroom (doc 13 §13.4) depends on this.
- **Counterbalance helper**: for Murphy beds and flip-up tables, compute the torque curve over the swing and size gas struts or springs (force, stroke, mount points), with "feels heavy at 30°" shown as a graph.
- **Human interaction checks**: Sancho reaches, opens and steps through. Reach envelopes (seated, standing), handle heights, finger pinch points (gaps 8–25 mm in moving joints get flagged).
- **Kinematic first, dynamic optional**: potato machines get kinematics and interference; "simulate drop / tip-over" (stability of a tall dresser, with an anti-tip recommendation) runs dynamics when the profile allows, or as a slower offline job.

## 6.9 Plan

Full detail in doc 10. Summary: PlanCraft core under the hood, with tasks generated from the design, resources from the inventory (tools as equipment, the maker's work windows as the calendar), deadline, critical path, timeline/board/calendar views, and "you are 2 evenings behind" with options.

## 6.10 Journal

Full detail in doc 10 §10.4. The project log: intent history, decisions with "why", auto-written steps from commands, session interviews, photos and video, as-built comparisons, and the publisher.

## 6.11 Drawings

**Purpose:** blueprints. Orthographic and section views of designs, dimensions, notes, part balloons, BOM tables, cut lists, title block. Based on DraftCraft's paper-space layouts, dimensions, multileaders, tables and blocks, driven by the 3D model (views update when the model changes).

Maker-friendly defaults: an **exploded assembly view per build step**, a **one sheet per part** option (for printing and pinning above the bench), imperial fractional dimensions, and big readable text.

## 6.12 Output

**Purpose:** the export pack. Doc 10 §10.5 lists everything it produces: blueprints, cut lists and diagrams, templates (1:1 tiled, plotter, projector), DXF/DWG/SVG/PDF, STEP/3MF/STL/OBJ/glTF/URDF, the build guide, and share bundles. It is one pipeline with many writers.

## 6.13 3D Print Prep (sidecar)

Orient parts (suggest flat-face-down, minimise supports), check wall thickness against the nozzle, check part size against the printer's bed (from inventory), estimate filament against inventory spools and time. Slicing is delegated to a **sidecar** wrapping an existing slicer engine. Check licences: many are AGPL, which is fine behind a process boundary as a separately distributed sidecar, but get legal review. Output: 3MF with settings, or G-code from the sidecar.

## 6.14 – 6.20 Later environments

| Env | Why it matters for makers | Sketch of approach |
|---|---|---|
| **Laser / Plotter / Vinyl** | Most packaging, signage and tab-and-slot work | True-shape nesting; power/speed per material from inventory; kerf compensation; lead-ins; layer = operation |
| **CNC / CAM** | Plywood furniture on a shared CNC | 2.5D toolpaths (profile, pocket, drill), dogbone fillets automatically on inside corners, tabs, sheet nesting from Stock & Cut |
| **Electronics** | Enclosures, lit signs | Import a board outline and connector positions (KiCad), generate cutouts, standoffs and cable paths. Not a full EDA tool |
| **Generative / Lightweighting** | "Remove material where it isn't needed" | Topology optimisation sidecar on capable machines; result is a mesh body plus a "rebuild as sheet ribs" option (curve-rib technique) |
| **Fluids / Airflow** | Vents, fans, enclosures | Simple CFD sidecar; capable profile only |
| **Render & Present** | Pretty pictures for the log and for clients | SolidCraft `gpu` path tracer; offered only above potato; potato gets the CPU storybook render |
| **Projector** | Cut and sew directly from projected lines | Doc 10 §10.6 |

## 6.21 Workflow packs (for other user types)

A **workflow pack** is a bundle of questions, interpreter rules, environment set, templates, toolbox layout and export presets: data plus optional WASM, with no core changes.

- **Maker** (default).
- **Packaging studio**: quantity-run questions, press-sheet nesting, printer-ready dielines.
- **Soft goods**.
- **Room planner**.
- **VFX / animation** (walkthrough §"Others"): props built to scale from real references; glTF/USD out; no stock/plan by default.
- **Game dev**: modular kits on a grid, LOD-aware exports, collision hulls.
- **Classroom**: locked-down tool set, teacher templates, simplified Sancho tone.

## 6.22 How the host decides what to load (rules)

1. On project open: load the declared environments in **Dormant → Loaded**, lazily. Active is only the one in view.
2. On a command from a not-loaded environment (from search, Sancho or MCP): offer "Load *Fold & Package*? (≈40 MB)". On capable profiles, just load it.
3. Memory budget per profile (doc 12): when it's exceeded, unload the least-recently-active environment with no unsaved state. Its data stays in the document; only runtime structures are freed.
4. Never unload during an operation, a drag, a solve or a running job.
5. Each environment's settings page shows its live cost, measured: "Space & Scan: 412 MB (scan cache 380 MB) [Free cache]".
