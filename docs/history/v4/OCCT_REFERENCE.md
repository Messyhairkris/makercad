# OCCT 7.9.3 — working reference (MakerCAD v4)

Shared by Claude + Crush. Keep additions to the cheat sheet as ops land.

## Version trap

`dev.opencascade.org/doc/refman/` redirects to the **latest dev docs (8.0.x)** —
NOT our version. Always use the pinned 7.9 tree:

- Reference manual: https://dev.opencascade.org/doc/occt-7.9.0/refman/html/index.html
- Overview / user guides: https://dev.opencascade.org/doc/occt-7.9.0/overview/html/index.html
- Class page URL pattern: `.../refman/html/class_<snake_case_with_underscores>.html`
  e.g. `class_b_rep_builder_a_p_i___make_face.html` (each capital gets `_`, `::`/`_` becomes `__`).

Ground truth beats docs: the installed headers at
`C:\OpenCASCADE\opencascade-7.9.3-vc14-64\inc\*.hxx` are exactly what we
compile against — grep them when the web docs disagree (Crush has local shell;
Claude works from the mounted repo, so paste header excerpts into the vault
if Claude needs them).

## Cheat sheet — APIs in use (containment TUs only)

| Class / call | Use in MakerCAD | Gotchas we hit |
|---|---|---|
| `BRepPrimAPI_MakeBox(dx,dy,dz)` | `make_box` | throws on ≤0 dims — we pre-check |
| `BRepPrimAPI_MakePrism(shape, gp_Vec)` | `extrude`, `extrude_profile` | rejects solids (our retry semantics); happily makes zero-volume prisms from in-plane sweeps — we reject via normal·v |
| `BRepAlgoAPI_Cut(t, o)` | `boolean_cut` | check `IsDone()`; can "succeed" with junk on tangent faces |
| `BRepFilletAPI_MakeFillet` | fillets | `Build()` + `IsDone()`; large radius vs edge length fails as expected → NO_SHAPE |
| `BRepBuilderAPI_MakePolygon` | `make_sketch_wire` | `Close()` before `Wire()`; collinear points are fine |
| `BRepBuilderAPI_MakeFace(wire, planar=true)` | `extrude_profile` | wire must be planar+closed; orientation matters → our `Reverse()` retry |
| `BRepMesh_IncrementalMesh(s, defl, rel, ang, parallel)` | `triangulate` | mutates shape's triangulation store; run before reading `BRep_Tool::Triangulation` |
| `Poly_Triangulation::Node(i)` | mesh readback | **1-based**; ≥7.6 API (older `Nodes()` array is gone) |
| `TopExp::MapShapes(s, TopAbs_EDGE, map)` | stable edge order | 1-based map; our SelectionIntent indices are this order **0-based** |
| `GCPnts_TangentialDeflection` | edge polylines | needs `NbPoints() ≥ 2` guard on tiny edges |
| `BRepAdaptor_Surface::GetType()` | ShapeAnalyzer routing | `GeomAbs_Plane`/`GeomAbs_Cylinder` only when the surface is *analytically* that type — booleans can demote to BSpline |
| `Bnd_Box` + `BRepBndLib::Add` | aabb, proxy | check `IsVoid()`; box is slightly inflated by tolerance |
| `IntCurvesFace_Intersector` | `raycast_faces` (C2 pick) | per-face construct + `Perform(lin, 0, RealLast())`; filter `State()==TopAbs_OUT` and `WParameter()<=0`; `gp_Dir` throws on zero vector — caught |
| `BRepGProp::SurfaceProperties` + `GProp_GProps` | `face_centroid`, `measure_face` | `CentreOfMass()` is area centroid, not bbox center |
| `BRepGProp::LinearProperties` | edge length (C7) | `Mass()` = length. **`CentreOfMass()` is NOT on the curve** for arcs — use `curve.Value((t0+t1)/2)` for a point that is |
| `BRepGProp::VolumeProperties` | `measure_solid` | `Mass()` ≈ 0 for wires/open shells — test before trusting `CentreOfMass()` |
| `BRepExtrema_DistShapeShape(a, b)` | `measure_distance` | 2-shape ctor performs immediately (no `Perform()` needed); check `IsDone() && NbSolution() >= 1`; solution index is **1-based** |
| `BRepFilletAPI_MakeChamfer::Add(d, edge, face)` | chamfer | needs the edge's owning face — build an edge→face map with `TopExp::MapShapesAndAncestors` |
| face normal orientation | `measure_face` | `surf.Plane().Axis().Direction()` is the *surface* normal; reverse it when `face.Orientation() == TopAbs_REVERSED` to get outward |
| `StlAPI_Writer` | `export_stl` | will NOT tessellate — run `BRepMesh_IncrementalMesh` first or you write an empty file |
| `STEPControl_Writer` + `Interface_Static` | `export_step` | writer options are **global state** — set schema/unit per call, never once at startup |
| `TopExp::Vertices(edge, v1, v2)` | `edge_endpoints` | null vertices on degenerate/closed edges — guard |
| `Standard_Failure` | every op | catch at the containment wall; NEVER let it cross the boundary |

## Project invariants (reminder)

OCCT headers in `src/core/shape_store.cpp` + `src/core/shape_analyzer.cpp`
ONLY. Everything crossing out is ShapeID / flat arrays / plain structs.
