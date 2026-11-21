---
title: "DELEGATION C1 — Sketch Node (workplane profile → extrude)"
project: MakerCAD
status: briefed
assignee: Claude
verifier: Crush
updated: 2026-08-11
tags: [makercad, delegation, C1]
---

# Delegation Brief C1 — Sketch Node

**Goal:** MakerCAD v4 gains its first real modeling verb: a closed 2D profile drawn on a workplane, extruded into a solid. This is the kernel half of Phase C ("sketch-first workflow"); UI is out of scope for you.

## Read first (in this order)

1. `C:\Users\<you>\MakerCAD\makercad_v4\ARCHITECTURE.md` — layer boundary, history model, render routing.
2. `C:\Users\<you>\MakerCAD\makercad_v4\src\core\history_engine.h` — `HistoryNode`, `Op`, `SelectionIntent`, param-slot conventions.
3. `C:\Users\<you>\MakerCAD\makercad_v4\src\boundary\makercad_extension.cpp` — how ops are bound and displayed.
4. `C:\Users\<you>\MakerCAD\makercad_v4\build.ps1` — your build+verify loop (idempotent; closes Godot for you; runs the editor import).

## Hard constraints (project invariants — violating any = rejection)

- **OCCT containment:** OCCT headers allowed ONLY in `src/core/shape_store.cpp` and `src/core/shape_analyzer.cpp`. Sketch/profile building goes in `shape_store.cpp` (or a new `sketch` private helper included only there). No OCCT type in `.h` reachable from `boundary/`.
- **Boundary data:** GDScript-facing API passes ints, floats, packed arrays, Dictionaries only. Sketch geometry crosses as flat float arrays.
- **History model:** new ops append nodes; params live in `f[4]` or a new serializable params struct; results are ShapeIDs; failed ops yield `NO_SHAPE`, never throw across the boundary (catch `Standard_Failure`).
- **Renderer:** Godot 4.6.2 Compatibility/ANGLE. CULL_DISABLED for solids, 2.5 px edge ribbons, no GL_LINES, no `gl_FragDepth`. You do NOT touch `game/` (that's Crush's lane).
- **No new dependencies.** OCCT 7.9.3 + godot-cpp + third_party/nlohmann only.
- **Perf contract:** median sketch-extrude op < 100 ms on the reference machine for ≤100-segment profiles.

## What to implement

### 1. Sketch storage op (new `Op::SKETCH`)

- Node params: plane id (v1: `0=XY, 1=XZ, 2=YZ` — axis planes only) + a serializable point list.
- Points are 2D in plane coords (mm). Cross the API as `PackedFloat32Array` [x0,y0,x1,y1,...].
- **Closure rule:** profile = polyline through points in order, implicitly closed (last→first). `close_sketch` validation: ≥3 points, no zero-length segments, no self-intersections (cheap check is fine; document its limits).
- Sketch results need an edge representation but NOT a solid: store the 2D wire. A sketch is displayable later; for C1, visibility via its extrusion result is sufficient.

### 2. Sketch → solid extrusion (extend `Op::EXTRUDE`)

- `extrude(sketch_node, vx, vy, vz)` where the input is a SKETCH node: build `BRepBuilderAPI_MakePolygon` wire → `BRepBuilderAPI_MakeFace` (plane mapped from plane id) → `BRepPrimAPI_MakePrism(face, vec)`. Assess wire orientation; fix with `wire.Reverse()` if the prism fails (defensive).
- Existing scope choice preserved: extrude on a SOLID still yields `NO_SHAPE` (retry semantics stay as-is).
- Units: 1 unit = 1 mm. Coordinates flow through unchanged.

### 3. API (boundary, exact signatures)

```gdscript
create_sketch(plane_id: int) -> int                      # node index or -1
add_sketch_points(node: int, pts: PackedFloat32Array) -> bool   # appends 2D pts; returns false on invalid node/not-a-sketch
close_sketch(node: int) -> bool                          # validates + seals; re-open = new profile
extrude(node: int, vx: float, vy: float, vz: float) -> int      # works on SKETCH nodes now
get_sketch_points(node: int) -> PackedFloat32Array       # boundary-safe readback
```

Node-type introspection: sketches participate in history like other nodes (`in[k]`, hashes, undo). A SKETCH node's result ShapeID may remain `NO_SHAPE` until extruded; document that in `history_engine.h` comments.

### 4. Tests you deliver (extend `game/scripts/test_api.gd` — allowed exception to the lane rule, additive only)

- `T11.1` create_sketch(0) returns node 0; add points of a 4-vertex rectangle; close_sketch true
- `T11.2` close with <3 points returns false (validation)
- `T11.3` extrude(sketch, 0,10,0) returns node index ≥ 0; `get_route_info` non-empty (route MESH=0); produced solid has edge segments
- `T11.4` extrude a triangle → history_size correct; undo_to(0) clears; readback `get_sketch_points` round-trips exactly
- `T11.5` non-planar garbage rejected gracefully (e.g., NaN coords → false, no crash)

Add the matching T-numbers to `HUMAN_TEST_PROCEDURE.md` appendix table (one line each, human check not needed — these are headless).

## Acceptance criteria (Crush verifies)

1. `build.ps1` → SUCCESS on the reference machine, no new warnings beyond existing D9014.
2. Previous smoke suite still 13/13; new T11.x all PASS (headless run).
3. Visual: a scene creating a house-shaped prism (square + roof triangle profile, 40×30 extrusion 50) renders solid with edges on the HD 530 — Crush captures this; you just ensure it's achievable via the API.
4. All invariants kept (grep: no OCCT include outside the 2 containment TUs).
5. Delivery note in `wiki/projects/makercad/messages/2026-MM-DD - claude.md`: what changed, test output, any known limits (esp. self-intersection check honesty).

## Explicit non-goals (C1)

Constraint solving (that's C4/planegcs), dimensions/editing UI, splines/arcs (polyline only), multiple profiles per sketch, workplane-on-face (axis planes only in v1), picking (C2), 2D overlay rendering (Crush does this in the UX stub).

## Questions?

Post in `wiki/projects/makercad/messages/` rather than guessing. If a decision is needed, propose your best option flagged "PROPOSED" — Kris arbitrates.
