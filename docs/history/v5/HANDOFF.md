# MakerCAD v5 — handoff, 2026-11-13

Written after the eleven-hour stall, for whoever picks this up next — you, or Fable.

## 1. The correction this document exists to make

You did not ask for a Raspberry Pi case. You asked for **the tools to make one**. The
Pi enclosure in `mc-doc` is a *smoke test* that proves shell, split, fillet, holes and
booleans work end to end — it is not the deliverable and it is not the feature.

The deliverable is the **making path**, at the quality of the open-source programs you
supplied:

```
sketch (geometry + constraints + dimensions + snaps)
  -> profile detection
  -> feature ops (extrude join/cut/new, up-to-face, revolve, fillet, chamfer, shell,
                  pattern, mirror, split)
  -> pick faces and edges in the viewport
  -> export (STL, DXF, cut list) + the project log + Sancho's checks
```

Everything built so far is the floor that path stands on.

## 2. State, measured (not asserted)

| Check | Command | Result |
|---|---|---|
| Tests | `cargo test --workspace` | 45 passed, 0 failed |
| Layer law | `cargo run -q -p mc-cli -- layers` | 53 edges, ok |
| App runs | `cargo run -q -p mc-desktop -- --template --frames 120` | exit 0, 121 frames |
| Frame time | same run, debug build | mean update 24.69 ms, first frame 4282 ms |
| Pi template | `cargo test -p mc-doc` | 2 bodies (base + lid), 0 errors |

Release timing is **not** measured; the 16 ms frame budget is still a claim.

### Kernel vocabulary (`mc-kernel`, only crate that names OCCT)

box, box_at, cylinder (X/Y/Z), extrude polygon, cut_box, cut_hole, cut_with, fuse,
common, **shell** (wall + open face), **fillet**, **chamfer** (all / parallel-to-axis /
at-height edge picks), **split** (plane -> two bodies), translate, **place** (sketch on a
plane), **bbox**, area, volume, **faces** (type, area, centroid, normal), tessellate with
per-triangle B-rep face ids, `last_error`, `dup`.

Bugs the gates found, for the record: `TKFillet`/`TKOffset` were not linked; face ids
drifted when a face had no mesh; `Bnd_Box` padded every dimension by 1e-7; a second
`Common` returned volume 0.

### Document (`mc-doc`)

`Document { version, title, features, log, log_prompts_skipped }`. Feature kinds: Box,
Cylinder, Sketch, Hole, Shell, Fillet, Chamfer, Split, Move. Six named planes with outward
normals. JSON round-trip. A failed step records an error and leaves the body untouched.
`raspberry_pi_enclosure()` builds 20 features end to end.

### Files (`mc-io`)

`.makercad` JSON save/open with a version rule, binary STL, ASCII DXF (sketches + body
outlines), cut list with shelf packing, log Markdown with the Incomplete badge.

### Window (`mc-ui`)

History panel (select / reorder / delete), parameter editor for every feature kind,
Sancho panel (inline rules), log panel, File menu (open/save/STL/DXF/cut list/log),
orbit-pan-zoom camera, rectangle and circle sketching on the floor.

## 3. The honest gap against the references

| Area | SolidCraft | CADCraft | MakerCAD v5 today |
|---|---|---|---|
| Sketch geometry | lines, rects, circles, arcs, polygons, slots, ellipses, splines, conics, text; construction; trim; sketch fillet/chamfer | 2D drafting: lines, polylines, arcs, circles, splines, hatches, blocks | rectangle, circle (polygon approximation) |
| Constraints | coincident, point-on-curve, h/v, parallel, perpendicular, tangent, G2, equal, concentric, collinear, midpoint, symmetry, fix; dimensions drive parameters | drafting dimensions + constraints module (`constraints/{dof,model,residual}.rs`) | none; numbers are static |
| Snaps | adaptive curve picking, occurrence-aware | full object-snap taxonomy, tracking lines, direct distance entry (`snap/*.rs`) | grid only |
| Extrude | join/cut/new, up-to-face, linked/associative, taper | extrude 2D to 3D | along plane normal only, join/cut |
| Modify | fillet, chamfer, shell, patterns, mirror, split, draft | fillet, chamfer, offset, trim, extend | fillet, chamfer, shell, split (no patterns/mirror) |
| Selection | faces and edges in viewport, occurrence-aware | grips, entity selection, snap tracking | none — fillet/shell pick "all vertical" or "top face" |
| Files | DXF current view, STEP, 3MF | DXF round-trip as a first-class format | STL, DXF (write only) |
| Solver | own solver (Owen 1991 / Fudos–Hoffmann) | constraints crate | decided: vendored planegcs — not wired |
| Other | MCP, scripts, kernel ladder regression rig | command line, layers, blocks, WASM | none of these |

Everything in the left two columns is documented in the curated extracts (§4.2) — read
those before designing, they are small and specific.

## 4. The reference catalogue

### 4.1 The programs (local clones, with URLs)

All shallow clones, 2026-11-12, in
`C:\Tools\REAForge\work\makercad-2026-11-12\repos\`:

| Folder | URL | Licence | Take |
|---|---|---|---|
| `solidcraft` | https://github.com/tangency-labs/solidcraft (upstream author's fork: https://github.com/bherbruck/solidcraft; kernel: https://github.com/tangency-labs/solidcore) | MIT OR Apache-2.0 | the layer law, "everything is a command", the kernel ladder, the sketch/constraint feature surface, the flat-pattern model shape |
| `cadcraft` | https://github.com/storytold/cadcraft | MIT OR Apache-2.0 | object-snap taxonomy and priority, command-line input model, grips, direct distance entry, DXF, the constraints crate layout |
| `code_popupcad` | https://github.com/popupcad/code_popupcad | MIT | the laminate/layer model, cross-layer CSG as a mode |
| `packbox` | https://github.com/sbarrak/packbox (companion: https://www.pacdora.com/tools/design-a-box) | MIT | parametric dieline table, fold-slider interaction |
| `oriedita` | https://github.com/oriedita/oriedita | MIT | flat-foldability and layer-order checks, crease patterns (optional module) |
| `lamina` | https://github.com/marcelfarres/lamina (app: https://marcelfarres.github.io/lamina/app/) | **AGPL-3.0** | **concepts only — no code, no linking.** Five construction techniques, assembly-check list, unfolding doctrine, cut/score/engrave layers |

### 4.2 The curated extracts (read these first — they are the distilled version)

`C:\Tools\REAForge\work\makercad-2026-11-12\briefs\`

```
solidcraft/docs/capabilities.md     what the whole thing does, feature by feature
solidcraft/docs/features.md         every command, by area
solidcraft/docs/layers.md           the layer law
solidcraft/docs/kernel-ladder.md    the regression rig: 76 reference parts vs analytic volumes
solidcraft/docs/sketch-patterns.md  how sketching is expected to behave
solidcraft/docs/architecture.md
solidcraft/docs/mcp.md, scripts.md, action-matrix.md
cadcraft/snap/snap.rs               the snap taxonomy and priority
cadcraft/snap/curves.rs, iso.rs, tracking.rs, viewport.rs
cadcraft/constraints/model.rs, dof.rs, residual.rs
cadcraft/docs/architecture.md, docs/cli.md, README.md
packbox/types.ts, dimensions.ts, layout.ts, boxBuilders.ts, PackBox-Studio-User-Manual.md
popupcad/layer.py, layerdef.py, laminate.py, README.markdown
```

### 4.3 Every link shared for MakerCAD, by category

**The two that matter most**
- https://github.com/tangency-labs/solidcraft — "very close to how we want our program to be, it follows much of Fusion 360 design ideas"
- https://github.com/storytold/cadcraft — "this should be examined for our Sketch environment"
- https://github.com/tangency-labs/solidcore — SolidCraft's kernel crate

**Modelling modes**
- https://github.com/popupcad/code_popupcad — laminates/layers, cross-layer CSG
- https://github.com/marcelfarres/lamina — folded/layered construction, assembly checks (AGPL: concepts only)
- https://marcelfarres.github.io/lamina/app/ — the running app, for behaviour reference
- https://github.com/oriedita/oriedita — flat-foldability, crease patterns
- https://github.com/sbarrak/packbox — parametric dielines
- https://www.pacdora.com/tools/design-a-box — commercial box designer, for interaction reference

**Packaging / folding research (from FoldCAD)**
- https://boxes.hackerspace-bamberg.de/ — 25+ parametric box generators (GPL: read, don't copy)
- https://github.com/florianfesti/boxes — the Python companion; how a hinge graph is computed
- https://www.templatemaker.nl/en/ — clean SVG dielines, semantic groups
- https://github.com/amandaghassaei/OrigamiSimulator — hinge/kinematics model (MIT)
- https://github.com/zzhuyii/OrigamiSimulator — second implementation, face-tree structure
- https://www.cito.de/en/US/boxline/creasing_matrix/cito_abacus_online/cardboard/ — crease width vs caliper
- https://www.pinterest.com/mxjohnwong/fold-box/ — visual reference only

**Kernel, solver, geometry**
- https://dev.opencascade.org/doc/occt-7.9.0/refman/html/index.html — OCCT 7.9 reference
- https://dev.opencascade.org/doc/occt-7.9.0/overview/html/index.html — OCCT user guides
- https://github.com/FreeCAD/FreeCAD — planegcs (the chosen 2D solver), vendored in `makercad_v4\vendor\planegcs`
- https://github.com/nicholasgasior/2geom (original https://gitlab.com/inkscape/lib2geom) — 2D path/boolean gold standard, reference only
- https://blend2d.com/ — 2D renderer, reference only

**UX and interaction**
- https://help.autodesk.com/view/fusion360/ENU/?contextId=LP-STEPS-P13N-SNP-GS-OTH-CRD-1 — Fusion sketch constraints/snaps
- https://help.autodesk.com/cloudhelp/ENU/Fusion-Model/files/SLD-SELECTION.htm — how selection is meant to feel
- https://help.autodesk.com/view/fusion360/ENU/?guid=GUID-F0491540-0324-470A-B651-2238D0EFAC30 — Fusion toolbar/workspace model
- https://www.smashingmagazine.com/2013/09/introduction-to-polygonal-modeling-and-three-js/ — 2D outline -> 3D mesh (legacy web attempt)

**Older MakerCAD work (context, not code to reuse)**
- `C:\Users\<you>\MakerCAD\MakerCAD_Human_Centered_Professional_CAD_Workspace_UX_Specification.md` — the UX spec, with the Fusion links above
- `C:\Users\<you>\MakerCAD\Learning Projects\FoldCAD\INSPIRATION.md` — where the folding links came from
- `C:\Users\<you>\MakerCAD\makercad_v4\docs\OCCT_REFERENCE.md` — OCCT notes from the Godot attempt

## 5. File map — where everything lives

### The program
```
C:\Users\<you>\MakerCAD\makercad_v5\
  Cargo.toml                       workspace; serde + serde_json added 2026-11-13
  run.ps1  TESTING.md
  gates\build_plan.py              the gate for a Fable architecture pass
  docs\DESIGN.md                   the unified design (Fable pass 1 of the design)
  docs\BUILD_PLAN.md               Fable's architecture pass, 4,899 words (2026-11-12)
  docs\STATUS.md DECISIONS.md PLAN.md ATTRIBUTION.md
  crates\mc-kernel\src\lib.rs      the safe wall over the shim
  crates\mc-kernel\shim\shim.cpp   every OCCT call, plain C only
  crates\mc-kernel\shim\stub.cpp   the no-OCCT build
  crates\mc-kernel\build.rs        finds OCCT, links 15 toolkits, copies 415 DLLs
  crates\mc-doc\src\lib.rs         the feature history + rebuild + Pi template
  crates\mc-io\src\lib.rs          .makercad, STL, DXF, cut list, log
  crates\mc-ui\src\lib.rs          the window (lib_old.rs.bak = the previous one)
  crates\mc-view\src\lib.rs        camera, projection, ground barrier, grid
  apps\mc-desktop\src\main.rs      --frames N, --template
  apps\mc-cli\src\main.rs          `layers`
  xtask\src\lib.rs                 the layer table and the gate
  crates\{mc-assist,mc-log,mc-engine,mc-sketch,mc-solver,mc-sheet,mc-fold,mc-geom,mc-id}\
                                   still 9-line stubs — the layer law is waiting for them
```

### The research trail
```
C:\Users\<you>\ATLAS-api-harness\runs\crew\pa\chats\ses_e2eeb572cffe1dXl1dwoPYkJy0\
  plan.md                    the approved plan (453 lines + §11 decisions)
  tasks.json                 the task board
  walkthrough.md             the running record (read this first)
  fable_brief_1.md           Fable pass 1 (architecture) — the brief that produced BUILD_PLAN.md
  fable1.log                 its run log
C:\Users\<you>\ATLAS-api-harness\runs\crew\fable\20261112-184700\
  result.json                Fable's reply, gate results, file hashes
  scratch\docs\BUILD_PLAN.md  the plan as written (copied into the project)
C:\Users\<you>\ATLAS-api-harness\runs\crew\special_teams\
  UNIFIED_PLAN.md  UNIFY_NOTES.md   the 12 inconsistencies Fable found in the design
C:\Users\<you>\ATLAS-api-harness\crew\
  FABLE_PROMPTS.md            the five rules for briefing Fable
  contracts\mc_sancho_rules.md  contracts\mc_log_render.md
  hidden_tests\makercad\hidden_sancho_rules.rs  hidden_log_render.rs
  campaigns\makercad_v5_rules.json
```

### The vault
`C:\Users\<you>\Documents\Obsidian\Knowledge_Base\wiki\projects\makercad\`
`00 - Project Status and Direction`, `10 - Tandem Work Plan`, `20/21 - Delegation Briefs`,
`30 - Phase C Interaction Spec`, `40 - Packaging Data Model`, `50 - planegcs Spike`,
`50 - UX Spec Implementation Map`, `51 - C4a Readiness Package`,
`60 - Universal Geometry Proposal`, `61 - UI Map`, `62 - Fusion 360 Sketching Gap Analysis`.

### TheCrew (the tooling, fixed 2026-11-13)
```
C:\Users\<you>\TheCrew\
  bin\winguard-hook.exe     rebuilt 2026-11-13 with the bash fix
  src\bin\winguard_hook.rs  the PreToolUse guard (9 tests)
  src\bin\progress.rs       the task board
  README.md                 install = build + register; both steps now documented
```
Registered in `C:\Users\<you>\.claude\settings.json` (backup:
`settings.json.bak-2026-11-13`). Pushed to https://github.com/Messyhairkris/TheCrew at
`34fab91`.

### Older versions (docs are the asset; demos all failed)
`C:\Users\<you>\MakerCAD\`: `makercad_v4` (Godot), `Learning Projects\`
(`FoldCAD`, `LoCAD`, `RustCAD`, `ThreeJS_CAD`), plus the UX spec and project plans at the
root.

## 6. Commands

```powershell
cd C:\Users\<you>\MakerCAD\makercad_v5
cargo test --workspace                                  # 45 tests
cargo run -q -p mc-cli -- layers                        # the architecture gate
cargo run -p mc-desktop                                 # the app
cargo run -p mc-desktop -- --template                   # start with the Pi case loaded
cargo run -p mc-desktop -- --frames 120                 # measured smoke run, prints timings
```

## 7. Decisions already made — do not relitigate without the PM

1. **OpenCASCADE 7.9.3** is the kernel, behind a plain-C shim; no OCCT type reaches Rust;
   `mc-kernel` is the only crate that names it.
2. **SolidCraft is the shape**: a document of features and commands, not a viewport that
   draws whatever was last clicked.
3. **The layer law** (`xtask`) holds; new crates get a row before they get code.
4. **The potato is the target**: i5-6600 / HD 530 / 8 GB. No Vulkan (it segfaults), so
   `WGPU_BACKEND=dx12,gl`; a CPU path must exist as the floor.
5. **Sancho** is deterministic first (rules), voiced second, AI last; muteable.
6. **The log** starts at project creation with the user's own description, is editable,
   prompts at close, and an AI rewrite is optional; skipped prompts => **Incomplete**.
7. **Licences**: MIT/Apache/BSD/MPL/BSL may be copied with attribution; LGPL may be linked
   dynamically only; **lamina is AGPL — concepts only**. The ledger is
   `docs\ATTRIBUTION.md` and it is a release gate.
8. The Pi enclosure is a **test**, not the product.

## 8. What to ask Fable, and how to run it

The prompt is `fable_brief_2.md` in this folder. Run it yourself:

```powershell
cd C:\Users\<you>\ATLAS-api-harness
python -m crew.fable_scratch run --project "C:\Users\<you>\MakerCAD\makercad_v5" --brief "C:\Users\<you>\ATLAS-api-harness\runs\crew\pa\chats\ses_e2eeb572cffe1dXl1dwoPYkJy0\fable_brief_2.md" --gate "cargo run -q -p mc-cli -- layers" --gate "python gates/increment.py"
```

Fable works in a **scratch copy**; nothing reaches the real project until you run

```powershell
python -m crew.fable_scratch promote <result_dir>
```

`result_dir` is printed at the end (it is under `runs\crew\fable\<timestamp>\`).
Approval runs to 2026-11-15, and the hard cutoff is 11:50 that day. One pass: it is
expensive and it is at its best on a finished base, which is what §2 is.
