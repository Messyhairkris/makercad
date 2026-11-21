# 09 — Scan to Space

The goal: get the real world into the project accurately enough to build against it, at whatever fidelity the user and their computer can manage. The output is not a pretty scan. It is a **Space**: clean, editable reference geometry (walls, floor, openings, frames, furniture), with the scan kept as a ghost overlay for checking.

Base: **SurfaceCraft's** clean, align, segment, fit and deviation pipeline on solidcore (doc 02 §2.6). MakerCAD adds point-cloud and splat ingestion, room semantics, scale calibration and measured-entry tools in `mk-scan`.

## 9.1 Input methods, from cheapest to richest

| Method | Needs | Accuracy (typical) | Output |
|---|---|---|---|
| **Typed measurements** | Tape measure | ± 1–3 mm (user) | Box or outline Space |
| **Sketch-and-measure** | Tape; draw a floor plan, type each wall length and corner angle | ± 2–5 mm | Outline Space with heights; diagonal checks to catch non-square rooms |
| **Photo + reference** | Phone; a known object (A4/Letter sheet, credit card, a printed MakerCAD calibration marker) | ± 1–2% in the reference plane | Calibrated image plane; rectified measurements |
| **Phone LiDAR room capture** (e.g. RoomPlan-type exports, USDZ) | LiDAR phone or tablet | ± 1–3 cm | **Already-semantic** walls, doors, windows, furniture boxes, directly mapped to a room shell |
| **Mesh scan** (photogrammetry or LiDAR: OBJ/PLY/GLB/STL) | Phone app or scanner | ± 0.5–3 cm (room), ± 0.1–1 mm (object, good scanner) | SurfaceCraft pipeline |
| **Point cloud** (PLY/LAS/LAZ/E57/XYZ/PTS) | Scanner, LiDAR app | Device-dependent | Points pipeline (§9.4) |
| **Gaussian splat** (3DGS .ply / .splat / .spz) | Splat capture app | Visual is excellent; geometry depends on capture; scale often arbitrary | Splat pipeline (§9.5) |

The question flow (doc 05) offers these by what the user's inventory says they have: a phone with LiDAR, or a phone without.

## 9.2 Staged import (no frozen app)

From the v4 Universal Geometry proposal §4.2 and §30, kept because it was right:

1. **Inspect** (< 1 s): header only. Format, point/triangle count, bounds, units guess, file size. Shown immediately: "12.4 M points, ~4.1 × 3.2 × 2.6 m, probably metres."
2. **Preview** (seconds): a decimated version (≤ 200k points or triangles on potato) appears as the ghost. The user can start working.
3. **Process** (background job with progress and cancel): clean, align, detect. Results stream in, so walls appear as they're found.
4. **Cache:** processed results (decimated display set, detected primitives, spatial index) go to a project-local cache keyed by the source hash, so reopening is instant.

**Partial success is a feature:** if detection fails in a cluttered corner, the walls that *were* found are kept, and the corner is flagged "needs a manual edge".

## 9.3 The canonical, display and source split

| Layer | Contents | Lives in |
|---|---|---|
| **Source** | The original file, untouched | External path + hash (embedded only with "Pack for sharing") |
| **Canonical** | Cleaned and aligned data; detected primitives; the room shell (B-rep) | Project cache + the `.maker` file (the room shell is small) |
| **Display** | Decimated points or mesh or splat LOD for the current profile | Cache only; regenerated |

Edits (cropping, deleting furniture, fixing a wall) operate on canonical data with full undo. The source is never modified.

## 9.4 Point clouds and meshes: the detection pipeline

1. **Units and up-axis:** guess from the bounds plus gravity-like dominant plane normals, and confirm with one question ("Is this floor?" with the plane highlighted).
2. **Clean:** statistical outlier removal; voxel downsample (adaptive: 5 mm for objects, 1–2 cm for rooms on potato); SurfaceCraft's mesh cleaning (hole fill, spikes, Taubin smooth, QEM decimation) for meshes.
3. **Crop:** a box or lasso crop tool. "Keep this room" lets the user discard the hallway the scanner wandered into.
4. **Align:** the floor goes to Z = 0. Then the **Manhattan frame**: the dominant orthogonal wall directions become the X/Y axes, because most rooms are mostly square, and this makes everything downstream snap cleanly. N-point and ICP alignment (SurfaceCraft) for registering multiple scans together.
5. **Plane detection:** efficient RANSAC or region-growing on normals. Each plane is classified by orientation and position: **floor** (lowest large horizontal), **ceiling** (highest), **walls** (large verticals bounding the floor), and **other horizontals** (bed top, desk, shelf, windowsill).
6. **Room shell:** intersect wall planes with the floor and ceiling to get a **watertight room B-rep** with exact corners. Out-of-square angles are preserved and shown ("this corner is 91.3°"), never silently squared. The **alcove** example lives exactly here: the cutout becomes real faces with real dimensions.
7. **Openings:** detect holes in wall planes (doors, windows, the cutout) from missing points in the wall region plus depth discontinuities. Fit rectangles, measure sill and head heights, and add **door swing** keep-outs (side asked once).
8. **Wall-mounted items:** small rectangles proud of a wall plane by 5–60 mm become frames, switches, outlets, radiators and skirting boards. The type is guessed by size and height and confirmed in a quick list ("I found 3 frames, 2 outlets, 1 switch — right?"). Outlets and switches matter: Sancho warns when furniture blocks them.
9. **Furniture and objects:** remaining clusters above the floor are fitted with oriented bounding boxes (and cylinders), labelled by size heuristics ("bed-sized", "chair-sized"), and kept as **simple editable bodies**. The user can delete them (it's moving out), keep them as obstacles, or "replace with a model".
10. **Edges and silhouettes:** sharp feature lines (plane intersections, depth edges) are extracted as **reference curves**. Sketch can project them, and snaps can catch them (corner of the window frame, top of the skirting).

Every detected element is **ordinary editable geometry** with a link back to its evidence (the points it came from), plus a **confidence** shown as a subtle dashed edge when below 0.8. "Refit" re-runs a single element after the user adjusts its region.

## 9.5 Gaussian splats

Splats look real but have no surfaces and no edges, and their scale is often arbitrary. The 2026-08 walkthrough raised exactly this: "we will need to think through how the tool would work allowing the user to exist in the splat environment while still modelling a functional part."

MakerCAD's answer: **a splat is two things at once.**

1. **A visual environment** (capable and modest profiles): rendered as a backdrop the user can orbit inside, with design geometry drawn over it and correctly depth-composited where possible. On potato it is replaced by its point-cloud proxy or by the room shell with photo textures.
2. **A point cloud for measurement and detection:** splat centres, filtered by opacity (e.g. > 0.5) and by scale (drop huge, blurry background Gaussians), weighted by opacity, go through the §9.4 pipeline.

**Splat-specific tools:**
- **Measurement probe** (UX spec §28): click two points on the splat. Each click ray-marches the splat and picks the depth where the accumulated opacity crosses 0.5, giving a 3D point. The distance shows with an uncertainty band from the local Gaussian spread.
- **Scale calibration** (required unless the capture app provides metric scale): "Measure something real." The user picks two points on a known object (a door's width, an A4 sheet placed in the scene before capture, or a printed marker) and types its real length. The whole splat rescales. A **second reference** is strongly suggested as a check, and the residual is shown ("second check: 0.4% off ✓").
- **Plane snapping**: detected planes become snap targets inside the splat, so drawing "on the wall" works even though the splat has no wall.
- **Crop and clean**: lasso-delete floaters, box-crop to the room.

## 9.6 Object scans and fix-it parts

For bench-scale work (the battery cover, a knob, a bracket):
- A mesh scan goes through SurfaceCraft's primitive fitting (planes, cylinders, cones, spheres, tori, symmetry planes), then into **sketch sections cut from the mesh**, then parametric features. This is reverse engineering for real.
- **Photo-measure** when there's no scanner: photograph the broken part (or the gap it fits) flat on a reference grid (a printed MakerCAD calibration mat with known squares). The image is rectified with a homography, so it is metrically flat, and the user traces it in Sketch. A **calipers prompt** follows for the critical dimensions (clip width, latch depth).
- **Mating geometry first**: for a battery cover, the important geometry is the *opening* it fits into. MakerCAD suggests modelling the opening first and deriving the cover from it with clearance, the same pattern as "box around the product".

## 9.7 Scale references for photos (shared tool)

Used for salvage intake, accessory layouts (backpack, doc 13 §13.3), mood-board references and defect mapping:
- Built-in references: A4, Letter, ISO credit card (85.60 × 53.98 mm), US quarter, and the **MakerCAD calibration mat** (printable PDF with an ArUco-style marker grid, giving automatic detection and perspective correction).
- Auto-detect a printed marker; otherwise click the four corners of the reference.
- Output: a rectified, scaled image plane. Outlines can be traced into bodies of the right size (extruded by a typed or measured thickness).

## 9.8 As-built comparison

After the build, scan the result (or the room with the result in it), then **Compare**:
- Align the scan to the design (ICP, seeded by the user picking 3 matching points).
- **Deviation map** (SurfaceCraft): a colour heat map of distance from the design, with a legend in the user's units.
- Measurement callouts at key dimensions ("width: designed 806, built 803 (−3 mm)").
- The comparison is saved as a **journal record** with screenshots, and feeds the published build log ("How close did I get?").
- Persistent deviations feed back into the tool's accuracy record, so Sancho learns "your jigsaw cuts run 1 mm wide."

## 9.9 Performance on low-end machines

| Profile | Point budget (display) | Processing | Splat |
|---|---|---|---|
| potato | ≤ 300k points / 200k tris | Chunked and out-of-core; offer "process overnight"; never more than 1.5 GB RSS | Not rendered; centres used as points |
| modest | ≤ 2M points | In-memory up to ~20M source points | Rendered at reduced Gaussian count (importance-pruned) |
| capable | ≤ 10M | Fully in memory | Full |

The **computer as a tool** shows up here (doc 12): "This scan has 48M points. On this PC, cleaning takes about 25 minutes in chunks. Start now and keep designing on the preview?"

## 9.10 Acceptance

| Criterion | Test |
|---|---|
| A synthetic room with an alcove (known dimensions, 2 mm noise) gives a room shell within ± 3 mm on every wall and opening | `xtask corpus --scan room_alcove` |
| A splat with a known reference rescales to within 1% at the second check | Corpus splat `bedroom_ref.ply` |
| Detection of 3 frames, 2 outlets and 1 door in the test room: ≥ 5/6 found, 0 hallucinated | Corpus |
| A 10M-point room on potato: the preview appears in < 10 s and the app never freezes > 100 ms | Potato bench |
| As-built compare on a 1 m test cabinet scan: reported deviation within ± 2 mm of caliper truth at 5 points | Physical test (manual STATUS row) |
