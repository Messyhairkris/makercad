# C11 — Sketch Create tools, object snaps, construction planes (2026-09-29)

Session goal (Kris, Notes_9-29): study five inspiration links, find what MakerCAD is
missing, delegate to the Crew, and build toward Fusion-360 parity, **starting with
Sketch and Construction tools**.

## 1. What the inspiration links contribute

| Source | What it is | What MakerCAD takes from it |
|---|---|---|
| **GodotCAD** (forum 135322; github alnicolle/GodotCAD-Godot-AutoCAD-like, MIT) | A 2D AutoCAD-like drafting app written in Godot/GDScript: line/circle/arc, SnapManager (endpoint, midpoint, center, intersection, perpendicular), move/copy/rotate/scale/mirror/offset/trim, window+crossing box select, grips, layers, DXF R12 export | **Object snaps: shipped today** (same five modes plus nearest + grid). Still to take: grips, box select (window vs crossing), offset/trim, layers, DXF export. Proof that this all works in GDScript. |
| **Fusion 360 3D Sketch** (help SKT-3D-SKETCH) | Sketches not confined to a plane; a manipulator of plane + axis + rotation handle places geometry in 3D | Later. The prerequisite is construction planes and a real gizmo (doc 62 gap 12). |
| **Fusion Construct menu** (help SLD-CONSTRUCT-TOOLS) | 8 plane tools, 5 axis tools, 6 point tools | Maths for 11 of 19 is shipped (`SketchGeom`); **Offset Plane and Plane At Angle are wired into the UI today.** |
| **SketchForge-3D** (github Formsmith746, AGPL-3.0) | Browser, local-first "Tinkercad-style" workflow: primitives (box, cylinder, sphere, cone, wedge, torus, tube, text...), a solid/hole toggle, booleans, reversible fillet/chamfer, STL/OBJ/STEP import+export, snap settings, transform handles, ortho/persp toggle (O), MCP tools for AI | A second workstyle MakerCAD should support beside sketch→extrude: **drop a primitive, drag handles, flip it to a hole**. The kernel already has box + booleans. Its MCP server is the model for letting agents drive MakerCAD. **AGPL: study only, copy no code.** |
| **ForgeCAD public kit** (github, MIT kit; the product is commercial) | Code-first parametric CAD in JS (`.forge.js`): primitives, booleans, constrained sketches, params, named faces/edges, assemblies/joints, STL/STEP; docs = runnable examples + API ref + CLI ref + agent-skills library | **Evaluated: useful as documentation style, not code.** Its docs pattern (every API page is a runnable example + an agent skill) is the model for a MakerCAD scripting/agent API page later. MakerCAD's `serialize()` history JSON is already half a script format. |
| **Cheetah Mesh** (itch.io, Android APK, name-your-price) | Mobile, offline, touch-first mesh modeler: vertex control, "covering" (fill), materials, rigging/skeleton library, animation, video-to-animation (beta), import/export, low-poly mode, an in-app guide character | The mesh/poly-modeling + touch workstyle, and the **guide character** idea (an in-app assistant that coaches). Not examinable in depth without running it — see section 5. |

## 2. What shipped today

Shell (`game/scripts/maker_shell.gd`) + new pure-maths helper `game/scripts/sketch_geom.gd`
(`class_name SketchGeom`).

**Sketch mode (K)** — the session now stays open after a profile closes; keep drawing,
**E** extrudes the newest profile.

| Key | Tool (Fusion name) | Clicks |
|---|---|---|
| L | Line (polyline; first point or Enter closes) | open |
| A | next Line segment is a **3-Point Arc**: click end, then a point on the arc | 2 |
| R | 2-Point Rectangle → Center Rectangle → 3-Point Rectangle (press again to cycle) | 2 / 2 / 3 |
| C | Center Diameter Circle → 2-Point Circle → 3-Point Circle (cycle) | 2 / 2 / 3 |
| P | Polygon (inscribed), `[` `]` change sides 3..64 | 2 |
| S | Center-to-center Slot (third click sets the width) | 3 |
| F | **Sketch Fillet** r (`[` `]` ±0.5 mm): click a corner. Open-polyline corners round in place; a closed profile's corner is rebuilt only while it is the newest history node | 1 |
| O | **Offset** d (`[` `]` ±0.5 mm): click inside a closed profile → a new offset profile (original kept, as in Fusion); Shift+click = inward | 1 |
| G | object snaps on/off (grid only) | |

Trim and Extend maths is shipped (`SketchGeom.trim_line` / `extend_line`), but the tools
are not wired: they need sketch lines as editable objects, which comes with constraints.

- **Object snaps** with AutoCAD glyphs: endpoint ▢, intersection ✕, midpoint △,
  center ○, perpendicular ⟂ (to the last point), nearest ⧖, grid. Candidates rank by
  kind before distance, so a far endpoint beats a near "nearest". 12-px aperture.
  The sketch origin is always snappable. Circles keep a true circle for center snaps.
- **H/V inference** (doc 62 gap 9): within 8 px of horizontal/vertical from the last
  point the cursor locks, with an orange dashed guide and an H/V badge.
- **Live preview** of every tool plus a running segment length (a first step on
  doc 62 gap 10, dynamic input).
- Profiles already drawn on the same plane stay visible and snappable.

**Construction planes** (command palette, S):
- *Construct: Offset Plane* (and a *flipped side* variant), *Construct: Plane At Angle*
  (about the base plane's u axis). The base is the selected construction plane, else
  the selected planar face, else the ground. The value is entered through the same
  live-preview parameter panel as extrude.
- Planes draw as translucent orange squares; click one to select it; **K sketches on
  it**. The sketch node stores its frame (schema v3 `plane[9]`), so the geometry
  persists even though the plane object itself is shell-side (see gaps).

Also fixed on the way: `_next_sketch_id` never incremented (every sketch had id 1);
E in sketch mode with no closed profile crashed on an empty array.

## 3. Verification (all run 2026-09-29)

| Check | Result |
|---|---|
| Kernel gate `scenes/test_api.tscn` | 63/63 |
| GDScript port vs Python reference, `scenes/test_sketch_geom.tscn` (659 vectors from `tests_py/gen_sketch_geom_vectors.py`, incl. trim/extend/fillet/offset) | 659/659 |
| Shell end-to-end `maker_shell.tscn -- --sketch-steps` (real kernel, clicks through the snap path; S14 fillet, S15 offset) | 15/15 |
| Crew hidden tests (`check_accepted.py`) | 398 passed across all accepted modules |

```
Godot_v4.6.2-stable_win64_console.exe --headless --path game res://scenes/maker_shell.tscn -- --sketch-steps
Godot_v4.6.2-stable_win64_console.exe --headless --path game res://scenes/test_sketch_geom.tscn
```

Two things the tests taught us:
- The headless viewport is 64×64 px; a pixel-sized snap aperture then covers 15 mm.
  The self-test zooms to 8 px/mm first. Real windows are fine.
- float32 `Vector2` makes a true half-circle measure a hair over π, so the port's
  segment ceil uses 1e-6 slack, not the contract's 1e-9 (documented in the file).

**Not yet verified by a human**: the look and feel of the glyphs, the preview, and
plane picking with a real mouse. A test pass is the next step.

## 4. Crew record

Plan big (contracts, references, hidden tests by Claude), delegate small, unify big.

| Module | Author | Tester | Result |
|---|---|---|---|
| `sketch_shapes.py` (13 functions) | Qwen | Gordon | both accepted 1st try; 4 + 3 requests |
| `sketch_snap.py` (7 functions) | Gordon | Qwen | both accepted 1st try; 4 + 4 requests |
| `construct_geom.py` (20 functions) | Nemotron | Barry | module accepted 1st try, 2 requests; tests: see REPORT.md |
| `sketch_edit.py` (trim/extend/fillet/offset, wave 2) | Hoss | Qwen | module accepted 1st try, 2 requests in 50 s; tests: see REPORT.md |

Contracts: `ATLAS-api-harness/crew/contracts/mc_*.md`. Mutation testing found one blind
spot shared by Gordon's tests *and* the hidden tests (slot direction was only tested
from the origin); a hidden test now covers it.

## 5. What MakerCAD is still missing (ranked)

Sketch & construct (the current focus):
1. **Sketch constraints + dimensions** (C4a planegcs build → C11 constraint UX): blue/black
   under/fully-constrained, H/V/coincident/parallel/perpendicular/tangent/equal, editable
   dimensions. The single biggest Fusion gap.
2. **Trim / Extend / Offset / Fillet (sketch)**: GodotCAD has trim+offset; `SketchGeom`
   intersections are the base for trim.
3. **Profiles from overlapping curves** (Fusion finds closed regions where lines cross);
   today each tool makes its own closed wire.
4. **True arcs in the kernel** (doc 62 C9 v2): circles are 48-gons today, so a cylinder is
   faceted.
5. **Construct UI for the rest**: Midplane, Plane Through 3 Points, Plane Along Path, the
   axes and points (maths is done, needs vertex/edge picking). **Persist construction
   planes in the document** (a CONSTRUCT node type — kernel ask).
6. **Project / Include** edges into a sketch (doc 62 gap 6), Construction-line toggle (X).
7. **Box select window/crossing + grips** (GodotCAD).
8. **Sketch mirror / rectangular + circular pattern** — declined by doc 62, but Kris's
   9-29 note ("replicate all the same tools found in Fusion360") may supersede that.
   **Needs a ruling.**

Other workstyles from the links:
9. **Primitive + hole workflow** (SketchForge/Tinkercad style): place box/cylinder/sphere,
   drag handles, toggle solid/hole.
10. **Gizmo** (move/rotate/scale handles) and the Fusion 3D-sketch manipulator.
11. **Agent API / MCP** (SketchForge, ForgeCAD): drive the shell from an agent; the
    Interaction Recorder already logs `sketch` and `tool` events for it.
12. **Mesh/touch workstyle** (Cheetah Mesh).
13. **DXF export** (GodotCAD), STL/OBJ import (SketchForge).

## 6. Open items

- `--auto-steps` (the older C.5–C.10 demo harness) **hangs headless** — it hung before
  today's edits too. Needs a look.
- makercad_v4 is **not a git repository** any more (memory records commit faf5fea).
  Today's pre-edit copies are in `agent_workspace/backup_2026-09-29/`.
- Android emulator for Cheetah Mesh: possible on this PC (Hyper-V on, 7.7 GB RAM, 74 GB
  free, no Android SDK yet) — awaiting Kris's go-ahead before any download.
