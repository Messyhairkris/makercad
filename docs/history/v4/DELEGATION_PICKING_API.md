---
title: "DELEGATION C2 — Picking API (click → face/edge indices)"
project: MakerCAD
status: briefed
assignee: Claude
verifier: Crush
updated: 2026-08-11
tags: [makercad, delegation, C2]
---

# Delegation Brief C2 — Picking API

**Goal:** clicking in the viewport returns stable, boundary-safe references (face/edge indices) so the UX can select entities and feed `SelectionIntent::INDEXED` (the missing piece for per-edge fillet and face-operated verbs). Kernel only — no `game/` changes (that's Crush's lane).

## Read first

- `makercad_v4/ARCHITECTURE.md`, `src/core/ids.h`, `src/boundary/makercad_extension.{h,cpp}` — same invariants as C1.
- Reference port source: old extension `C:\dev\makercad-extension\src` — its `RayPicker` used `IntCurvesFace_Intersector`. Port the concept, not the old code style.
- C1 delivery conventions: `make_sketch_wire`/`extrude_profile` in `shape_store.cpp` are the pattern for new OCCT work (containment TU only).

## Hard constraints

Same as C1 (OCCT confined to the 2 TUs; floats/ints/packed arrays/Dictionaries at the boundary; `Standard_Failure` caught; CULL_DISABLED + ribbons untouched; no new deps; `game/` untouched).

Selection stability rule: indices refer to **`TopExp::MapShapes(TopAbs_FACE/EDGE)` order on the SAME current tip shape** — documented as "valid within one history revision; re-pick after any geometry-affecting op" (v4 honesty rule; the full HeuristicMatcher of the v3 plan is NOT in scope).

## API (boundary, exact signatures)

```gdscript
pick(screen_x: float, screen_y: float) -> Dictionary
# {} when nothing hit. Otherwise:
#   {"node": <history node index the hit belongs to>,
#    "face": <MapShapes FACE index, 0-based> or -1,
#    "point": [x, y, z] world-space hit point}

pick_edge(screen_x: float, screen_y: float, tolerance_px: float = 6.0) -> Dictionary
# {} or {"node": <index>, "edge": <MapShapes EDGE index, 0-based>, "point": [x,y,z]}
# EDGE index = the same order used by fillet_edges SelectionIntent::INDEXED.

get_face_centroid(node: int, face: int) -> PackedFloat32Array   # [cx,cy,cz] or {}
get_edge_endpoints(node: int, edge: int) -> PackedFloat32Array  # [ax,ay,az,bx,by,bz] or {}
```

Implementation notes:
- Ray from camera: boundary receives screen coords + fetches `get_viewport()->get_camera_3d()` (Phase-2-accepted pattern, must `ERR_FAIL_NULL` guard).
- Face pick: `IntCurvesFace_Intersector` per face of the current tip; nearest positive hit wins.
- Edge pick: nearest edge polyline segment within `tolerance_px` in screen space (use the SAME segment soup emitted in `MeshPayload.edges` so index alignment is guaranteed by construction).
- Multi-node scenes: iterate visible/history result shapes; nearest hit across all.

## Tests (additive to `test_api.gd`, T12.x) + procedure appendix

- `T12.1` create box (BOX route), cast `pick` at the box center screen position → returns face ≥ 0, correct node
- `T12.2` `pick` at empty screen region → `{}`
- `T12.3` cut-through shape: `pick_edge` near a known internal edge → returns exact `edge` index; `fillet_edges(node, [that], 0.2)` then succeeds and differs from ALL_EDGES result (edge count changes)
- `T12.4` pick does NOT mutate history (`history_size` unchanged) — read-only by contract
- `T12.5` after `undo_to`, picks reflect the restored state (re-pick semantics documented)

For determinism in headless tests, screen coords that hit geometry may be derived via camera projection of known points — document the helper you use.

## Acceptance (Crush verifies)

1. Build SUCCESS; 19/19 existing + T12.x PASS headless.
2. Interactive proof (Crush captures): click a box edge → its ribbon highlights → `fillet_edges` on it fillets that edge only. (Crush writes the tiny harness after your API lands.)
3. Delivery note in `messages/` with known limits.

## Known-limit honesty (required in delivery note)

Document: MapShapes-order stability claims, behavior during drag (proxy mode = no picks?), precision limits near silhouettes, and behavior with multiple overlapping shapes.

## Non-goals (C2)

HeuristicMatcher/topological naming, marquee/box selection, hover pre-highlight loop (Crush may do that in GDScript from your API), multi-select sets, sketch-geometry picking (C3+ concern).
