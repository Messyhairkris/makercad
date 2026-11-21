# MakerCAD v4 — Architectural Blueprint

Target: i5-6600 / 8 GB / HD 530 · Godot 4.6.2 Compatibility renderer (GL 3.3 / ANGLE) · No Vulkan, no GL_LINES.

## Layers and boundaries

```
GDScript (game/lowfi_orbit_camera.gd, UI)
   │  Variant primitives only: int node indices, floats, packed arrays, Dictionary
   ▼
GDExtension boundary        src/boundary/makercad_extension.{h,cpp}
   MakerCADExtension : MeshInstance3D
   │  opaque ShapeIDs, MeshPayload flat arrays, RouteInfo plain structs (ids.h)
   ▼
Engine core (OCCT-free headers)
   HistoryEngine            src/core/history_engine.{h,cpp}
   ShapeStore (PIMPL wall)  src/core/shape_store.h
   ═══════════ OCCT CONTAINMENT BOUNDARY ═══════════
   ShapeStore::Impl + ShapeAnalyzer                 src/core/shape_store.cpp,
   TopoDS_Shape, BRepPrimAPI_*, BRepAlgoAPI_Cut,    src/core/shape_analyzer.cpp
   BRepFilletAPI_MakeFillet, BRepMesh_*             (the ONLY OCCT-including TUs)
```

Rule enforced by construction: no OCCT type appears in any header reachable from
`boundary/`. `shape_analyzer.h` forward-declares `TopoDS_Shape` and is included
only by the two containment `.cpp` files.

## History engine

Linear vector of `HistoryNode { op, f[4], in[2], SelectionIntent, hash, dirty, result }`
with `in[k] < index` enforced — one left-to-right pass, no graph traversal.
`hash` is FNV-1a over (op, params, intent, upstream hashes): a functional
pipeline hash. `evaluate()` walks `[first_dirty, end)` and hash-skips nodes
whose upstream is unchanged → O(k) partial rebuild. The mesh cache
(`hash → MeshPayload`, capped at 64) makes undo/redo re-display free.

C1 adds `Op::SKETCH`: a 2D polyline profile (xy pairs, mm) on an axis workplane
(0=XY n:+Z, 1=XZ n:+Y, 2=YZ n:+X), implicitly closed, validated OCCT-free by
`sketch_validate.h` (≥3 points, no zero-length segments, O(n²) proper-crossing
check), sealed by `close_sketch`. Its result is a planar WIRE ShapeID (NO_SHAPE
while open); `extrude` on a SKETCH input takes the wire→face→prism path
(`extrude_profile`), while extrude-on-solid still yields NO_SHAPE.

C2 adds picking: `pick`/`pick_edge` cast camera rays against the tip shape —
faces via `IntCurvesFace_Intersector` in the containment TU, edges via
screen-space distance to the SAME segment soup the ribbons render
(`MeshPayload.edge_ids` guarantees index alignment by construction). Picked
indices feed `fillet_edges` / `SelectionIntent::INDEXED` directly and are
valid within one history revision — re-pick after any geometry-affecting op.

`SelectionIntent` records edge selections as 0-based indices in
`TopExp::MapShapes(TopAbs_EDGE)` order — the same stable order `triangulate()`
emits edge polylines in, so viewport picks replay exactly on rebuild.

## Editing, redo, and files

`set_node_params` edits any node in place; the dirty-flag pass rebuilds only
what depends on it (the kernel half of Phase C's "type value → Enter commits").
`get_node_info` returns the timeline row — op, params, inputs, `has_shape`, and
a plain-language `broken_reason` when a node produced nothing (Phase C spec §5:
nothing fails silently). Undo moves trimmed nodes to a redo stack; a new op
clears it.

A `.makercad` file is the **history**, not geometry: ops, params, sketch
points and edge selections, serialized with Godot's own JSON (no added
dependency). Shapes rebuild on load, so files are tiny and stay parametric.
Malformed documents are rejected before anything is cleared, so a bad file
can never leave a half-loaded model. (Phase D's `.makercad-pkg` dieline
documents are a separate format.)

## Measurement and camera framing (C7)

`measure_edge` / `measure_face` / `measure_solid` / `measure_distance` /
`measure_angle` address entities with the **same indices the pick API returns**
(`entity`: 0=SHAPE, 1=FACE, 2=EDGE), so a click feeds a measurement directly.
Units are mm, mm², mm³. Angles are defined only between two planar faces or two
straight edges — anything else returns empty rather than an invented number,
and curved faces report `planar:false` with no normal.

`get_focus_target(node, entity, index)` answers "pivot here and frame it" in
one call: `center` (world space — the selection, per the camera pivot rule) and
`radius` (half the entity's bbox diagonal, never zero).

## Sketch point identity (C4b)

Sketch points carry stable ids (`pt_ids`, parallel to `pts`, handed out by a
per-sketch monotonic counter and never reused). Inserting or deleting a point
leaves every other id untouched, so a `Constraint` — which references points by
**id, not position** — stays valid across editing. Constraints are stored and
hashed but not yet solved; deleting a point drops exactly the constraints that
referenced it. File schema v2 carries both; v1 files upgrade on load by
assigning sequential ids, while a v2 file with a wrong id count, duplicate ids,
or a dangling constraint reference is refused rather than silently renumbered.

## Export

`export_stl(node, path, binary)` and `export_step(node, path, ap214)` write any
history node's shape. STL meshes first at the current display deflection (the
writer will not tessellate for us); STEP sets schema and `write.step.unit=MM`
**per call**, because OCCT's writer options are global state and would
otherwise leak between exports. Godot `res://`/`user://` paths are globalized
to OS paths at the boundary. 1 unit = 1 mm throughout.

## Render routing (ShapeAnalyzer)

Every displayed shape is classified:

| Route    | Condition                                   | Godot output                      |
|----------|---------------------------------------------|-----------------------------------|
| BOX      | 6 planar axis-aligned faces                 | BoxMesh + 12 analytic edge segs   |
| CYLINDER | 1 analytic cylinder + 2 caps, axis-aligned  | CylinderMesh + rim segs           |
| MESH     | anything else                               | tessellated ArrayMesh + ribbon    |

Edges are never GL_LINES: each segment becomes a 2-triangle quad ribbon whose
vertex shader extrudes to a constant pixel width (direction in NORMAL, side
sign in UV.x). Plain spatial shader — safe under Compatibility/ANGLE.

## Low-fi interaction fallback

`set_interaction_state(true)` (called by the orbit camera on drag start) swaps
the displayed mesh for a translucent 12-triangle AABB proxy built from
`ShapeStore::aabb`; `false` restores the routed display. One flag, two
transitions, fully inspectable.

## Build

```
cd makercad_v4
git clone -b godot-4.5-stable https://github.com/godotengine/godot-cpp extern/godot-cpp
cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DOpenCASCADE_DIR=<occt>/cmake
ninja -C build        # → game/bin/makercad.windows.release.dll
```

Static CRT (`/MT`) matches godot-cpp; `win_clean.h` is force-included first in
every TU via `/FI`; macro-conflict diagnostic suppressed via `/wd2589` per the
build constraint. Godot project must set
`rendering/renderer/rendering_method = "gl_compatibility"`.

## Smoke test (GDScript)

```gdscript
var cad: MakerCADExtension = $MakerCAD
var box := cad.create_box(4, 2, 3)          # routes to BoxMesh — no tessellation
var tool_box := cad.create_box(1, 5, 1)
var cut := cad.boolean_cut(box, tool_box)   # routes to MESH + edge ribbon
var soft := cad.fillet_all_edges(cut, 0.2)  # ALL_EDGES intent
print(cad.get_route_info(soft))             # {route: 0, params: [...aabb...]}
cad.undo_to(3)                              # linear undo; cached hashes skip rebuilds
```
