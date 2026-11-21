# MakerCAD Phase 2 — Expert Review & Updated Technical Plan

**Project:** MakerCAD — Open Source Parametric CAD on OpenCASCADE + Godot 4  
**Review Panel:** Claude (AI Engineering Advisor) & Otherguy (Senior Systems Reviewer)  
**Date:** April 27, 2026  
**Status:** Phase 2 In Progress — Rendering & Input Unblocked  

---

## Preamble

This document consolidates three rounds of technical review conducted by You and Otherguy against the original Phase 2 implementation brief submitted by Kris. It supersedes individual review passes. Where the two reviewers disagreed, the final position is stated explicitly with rationale. This document is intended to be read alongside `MakerCAD_Project_Plan_v3.md` and replaces ad-hoc debugging notes for Phase 2.

---

## Section 1 — What Has Been Confirmed

The following are not in dispute. Both reviewers agree these are the root causes of the two blocking issues.

### 1.1 Glitchy Cube — Root Cause: OCCT Face Orientation

When OCCT tessellates a solid, each `TopoDS_Face` carries an `Orientation()` of either `TopAbs_FORWARD` or `TopAbs_REVERSED`. A reversed face has its triangle winding order flipped relative to the outward surface normal. If this is ignored during mesh conversion, approximately half the faces of any convex solid will have inverted normals and will be culled by Godot's default backface culling. This is a textbook symptom match for "~2 faces visible on a cube."

**The fix is mandatory and foundational.** Every solid rendered by MakerCAD will be unreliable without it.

```cpp
// In mesh_generator.cpp — triangle index extraction loop
Standard_Integer n1, n2, n3;
triangles(i).Get(n1, n2, n3);

// Flip winding order for reversed faces
if (orient == TopAbs_REVERSED) {
    std::swap(n2, n3);
}

// Convert 1-based OCCT indices to 0-based Godot indices
indices.push_back(base + n1 - 1);
indices.push_back(base + n2 - 1);
indices.push_back(base + n3 - 1);
```

### 1.2 Glitchy Cube — Root Cause: Normals Not Computed

`BRepMesh_IncrementalMesh` does not automatically populate vertex normals on `Poly_Triangulation`. Godot requires `Mesh::ARRAY_NORMAL` for any lit material. Without normals, rendering bugs masquerade as geometry bugs, making them much harder to isolate.

**Option A — OCCT utility (preferred for smooth shading):**
```cpp
#include <BRepLib_ToolTriangulatedShape.hxx>

// After BRep_Tool::Triangulation():
BRepLib_ToolTriangulatedShape::ComputeNormals(face, tri);
// Then read: tri->Normal(i) for node index i
```

**Option B — Cross product per triangle (flat shading, simpler):**
```cpp
gp_Vec v1(p1, p2), v2(p1, p3);
gp_Vec normal = v1.Crossed(v2);
normal.Normalize();
// Assign to all 3 vertices of this triangle
```

If using Option B, the `std::swap` fix above automatically corrects normal direction — the cross product will point outward after the winding flip.

### 1.3 Glitchy Cube — Root Cause: TopLoc_Location Not Applied

All vertex positions must be transformed by the face's `TopLoc_Location` before being passed to Godot. Omitting this on any face causes mixed-space geometry, which explains the "rotates around a corner" symptom as much as the pivot issue does.

```cpp
gp_Pnt p = tri->Node(i).Transformed(loc);
// Use p.X(), p.Y(), p.Z() — never the raw Node() coordinates
```

### 1.4 Controls Not Responding — Root Cause: Input Not Wired

In Godot 4 GDExtension, input processing is not automatic. The node must explicitly opt in during `_ready()`.

```cpp
void CameraController::_ready() {
    set_process_input(true);
    set_process_unhandled_key_input(true);
    set_process(true);

    camera = get_viewport()->get_camera_3d();
    ERR_FAIL_NULL_MSG(camera, "CameraController: No active Camera3D in viewport.");
}
```

Additionally, `CameraController` must be explicitly registered in `register_types.cpp`:

```cpp
void initialize_makercad_module(ModuleInitializationLevel p_level) {
    if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) return;
    GDREGISTER_CLASS(MakerCADExtension);
    GDREGISTER_CLASS(CameraController);   // confirm this is present
    GDREGISTER_CLASS(WorkplaneGrid);
    GDREGISTER_CLASS(RayPicker);
}
```

The node class name in `test_phase2.tscn` must match the registered name exactly. A mismatch causes Godot to silently fall back to `Node3D`, disabling all overrides with no error message.

Keyboard input should use `_unhandled_key_input` to avoid UI conflicts:

```cpp
void CameraController::_unhandled_key_input(const Ref<InputEvent>& p_event) {
    Ref<InputEventKey> key = p_event;
    if (!key.is_valid() || !key->is_pressed()) return;
    if (key->get_keycode() == KEY_ESCAPE) { _reset_camera(); }
    if (key->get_keycode() == KEY_K)      { /* trigger pick */ }
}
```

---

## Section 2 — Architectural Decisions

### 2.1 Coordinate Space Invariant (Enforced Immediately)

**Decision:** All geometry entering Godot must be in local object space, centered at (0,0,0). No `TopLoc_Location` transforms may remain unapplied. The Node3D transform handles world placement.

This is not a future cleanup item. The "rotates around a corner" symptom is already a violation of this invariant. It must be verified now, not after camera controls are working.

**Verification test:** Print the first 10 vertices from the tessellator and compute their centroid. If the centroid is not near (0,0,0), the invariant is already violated.

**Required comment block in `mesh_generator.cpp`:**

```cpp
// COORDINATE SPACE INVARIANT
// - All output vertices are in local object space
// - Mesh is centered at (0, 0, 0)
// - No TopLoc_Location transforms remain unapplied
// - Units: 1 Godot unit = 1 mm (adjust if changed)
//
// VIOLATION SYMPTOMS:
// - Object orbits around a corner instead of its center
// - Picking cursor offset does not match selected face
// - Inconsistent scaling between shapes
// - Camera instability when zoomed in
```

**Centering implementation:**

```cpp
// Compute bounding box center after tessellation
Bnd_Box bbox;
BRepBndLib::Add(shape, bbox);
Standard_Real xmin, ymin, zmin, xmax, ymax, zmax;
bbox.Get(xmin, ymin, zmin, xmax, ymax, zmax);
gp_Vec center((xmin+xmax)/2.0, (ymin+ymax)/2.0, (zmin+zmax)/2.0);

// Subtract center from every vertex during conversion
gp_Pnt p = tri->Node(i).Transformed(loc);
p.Translate(-center);
```

The caller stores `center` and applies it as the `MeshInstance3D` position in the scene.

### 2.2 Camera Lookup Strategy

**Decision:** `get_viewport()->get_camera_3d()` is acceptable for Phase 2. Dependency injection via `NodePath` is deferred to Phase 3 when multiple viewports or editor integration may require it. The `ERR_FAIL_NULL_MSG` assertion (see Section 1.4) is mandatory — this keeps the approach simple without creating invisible Phase 3 bugs.

This decision was contested. Otherguy recommended immediate dependency injection. Claude recommended deferral. The reasoning for deferral: early CAD engines are more often harmed by premature architecture than by missing abstractions. The assertion provides the safety net.

### 2.3 ANGLE / OpenGL ES 3.0 Scope

**Decision:** ANGLE is not a blocking concern for mesh rendering or input. It does affect depth ordering and is relevant to the workplane grid.

Specific constraints:

| Area | Stance |
|---|---|
| Mesh rendering | Ignore ANGLE — no known blockers |
| Input handling | Ignore ANGLE — unrelated |
| Grid depth bias | Be conservative — avoid `gl_FragDepth` writes |
| Geometry shaders | Not available under GLES3/ANGLE — do not use |
| Performance | Keep triangle count below ~200k/frame for 30 FPS on HD 530 |

For the workplane grid, use render priority instead of depth hacks:

```gdscript
material.render_priority = -1  # renders grid behind everything else
```

### 2.4 Float/Double Precision

OCCT uses `double` (64-bit). Godot uses `float` (32-bit). This will cause jitter at tight zoom levels and picking inaccuracies if not managed.

**Decision:** Not urgent for Phase 2. Define the unit scale now — recommended `1 Godot unit = 1 mm` — and document it in the invariant comment. Revisit in Phase 3 when zoom and picking accuracy matter.

---

## Section 3 — Updated Validation Pipeline

Both reviewers agreed the original debugging approach — fixing tessellation, conversion, rendering, and materials simultaneously — was inefficient and made it impossible to isolate failure points.

**The following staged pipeline is now required for any rendering regression in MakerCAD:**

### Stage 0 — Baseline (Godot-only, no OCCT)

Construct a cube using hardcoded `ArrayMesh` vertices in C++. Bypass OCCT entirely. Use unshaded material, culling disabled.

- If this **fails** → the bug is in the Godot bridge layer (ArrayMesh construction, array format, strides, indexing)
- If this **passes** → the bug is in the OCCT conversion layer

This single test eliminates half the uncertainty before any other work begins.

### Stage 1 — Geometry Sanity

Use OCCT mesh. Unshaded material. Culling disabled. Render as wireframe if possible.

- If broken → tessellation or index construction issue

### Stage 2 — Topology Correctness

Enable culling. Still unshaded.

- If broken → winding order or face orientation issue

### Stage 3 — Lighting

Enable normals and shading.

- If broken → normals not computed, not populated, or direction wrong

Skipping directly to Stage 3 is not permitted during active debugging.

---

## Section 4 — Definitive Execution Order for Phase 2 Completion

The following ordered conditions must become true. Each condition enables the next. Do not combine steps.

| # | Condition | Test |
|---|---|---|
| 1 | ArrayMesh construction is correct | Hardcoded cube renders unlit, all faces visible |
| 2 | Coordinate space invariant holds | Centroid printed ≈ (0,0,0); `loc` applied to all vertices |
| 3 | Winding is correct | `std::swap` for reversed faces; culling enabled; all 6 faces visible |
| 4 | Normals are present and correct | `ComputeNormals()` called; `ARRAY_NORMAL` populated; Phong responds |
| 5 | Input reaches the controller | `set_process_input(true)` confirmed; class registered; viewport focused |
| 6 | Camera works on known-good geometry | Orbit/pan/zoom tested against Godot built-in mesh, not OCCT output |
| 7 | Systems reconnected | OCCT mesh substituted; no regression in conditions 1–6 |
| 8 | Picking added | `IntCurvesFace_Intersector` integrated after all above are stable |

**Picking must be last.** It depends on correct transforms, correct camera, and correct mesh topology. Integrating it earlier wastes time and introduces confounding variables.

---

## Section 5 — Updated Acceptance Criteria

The original Phase 2 acceptance criteria are retained. The following additions and clarifications apply.

### Additions

- [ ] **Coordinate invariant verified** — mesh centroid confirmed at (0,0,0) for test box shape
- [ ] **Hardcoded baseline cube passes** — Stage 0 validation complete before any OCCT debugging
- [ ] **Winding fix applied** — `std::swap` for `TopAbs_REVERSED` faces confirmed in `mesh_generator.cpp`
- [ ] **Normals computed** — `BRepLib_ToolTriangulatedShape::ComputeNormals()` called; `ARRAY_NORMAL` populated
- [ ] **`loc` transform applied** — all vertex positions use `.Transformed(loc)` before output
- [ ] **Camera assertion present** — `ERR_FAIL_NULL_MSG` on camera lookup in `_ready()`
- [ ] **Input opt-in confirmed** — `set_process_input(true)` and `set_process_unhandled_key_input(true)` in `_ready()`
- [ ] **Class registration verified** — all GDExtension classes confirmed in `register_types.cpp`
- [ ] **Grid uses render priority** — no `gl_FragDepth` writes in grid shader

### Clarifications to Existing Criteria

- **"Solid box renders all faces"** — verified at Stage 2 (culling enabled, unshaded) before shading is added
- **"30+ FPS on Intel HD 530"** — triangle budget ≤ 200k/frame; low-fidelity drag mode (bounding box during camera move) is required, not optional

---

## Section 6 — Phase 3 Prerequisites

The following items are deferred to Phase 3 but must be decided before Phase 3 begins. Raising them now prevents mid-phase blockers.

**Float/double precision policy:** Define how OCCT `double` coordinates are downcast to Godot `float`. If shapes span large coordinate ranges (e.g. assemblies in millimeters), consider a scene-origin offset or fixed-point scheme.

**Camera dependency injection:** When a second camera or editor viewport is added, `get_viewport()->get_camera_3d()` must be replaced with explicit `NodePath`-based lookup. Design the interface now even if implementation is deferred.

**Normal averaging at shared vertices:** If vertices are reused across faces (shared-vertex mesh), normals must be averaged at shared points. Flat-shaded per-triangle normals avoid this complexity but produce faceted rendering. Decide which is acceptable for Phase 3 display quality.

**Undo/redo system and dirty flag:** Phase 3 introduces history. The coordinate space invariant and mesh centering strategy must be compatible with shape replacement — verify that updating a shape ID in `ShapeStore` and regenerating the mesh does not violate any Phase 2 invariant.

---

## Appendix A — Quick Debug Reference

### Immediate rendering debug (GDScript)

```gdscript
# Temporarily applied to MeshInstance3D to isolate rendering issues
var mat = StandardMaterial3D.new()
mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
mat.cull_mode = BaseMaterial3D.CULL_DISABLED
mesh_instance.material_override = mat
```

### Centroid verification (C++)

```cpp
// Print in mesh_generator.cpp after vertex loop
float cx = 0, cy = 0, cz = 0;
for (auto& v : vertices) { cx += v.x; cy += v.y; cz += v.z; }
cx /= vertices.size(); cy /= vertices.size(); cz /= vertices.size();
UtilityFunctions::print("Mesh centroid: ", cx, cy, cz);
// Expected: near (0, 0, 0) after centering
```

### Input not firing checklist

1. `set_process_input(true)` called in `_ready()`?
2. `set_process_unhandled_key_input(true)` called in `_ready()`?
3. Class registered in `register_types.cpp`?
4. Node class name in `.tscn` matches registered name exactly?
5. Viewport has focus? (`get_viewport().gui_disable_input = false` to verify)
6. Mouse mode set? (`Input.MOUSE_MODE_VISIBLE` for orbit/pan)

### ANGLE shader rules

- ✅ `texelFetch` — supported
- ✅ Uniform buffer objects — supported
- ✅ `dFdx` / `dFdy` — supported
- ✅ `inverse()` in GLSL — supported
- ⚠️ `gl_FragDepth` — unreliable on HD 530, avoid
- ❌ Geometry shaders — not available under GLES3/ANGLE

---

*Document authored by Claude and Otherguy following three rounds of technical review. Approved for use in MakerCAD development. MIT License applies to all code samples.*
