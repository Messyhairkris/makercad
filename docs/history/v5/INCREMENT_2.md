# Increment 2 - sketch and features, and the MakerCAD shell around them

2026-11-13. Written for whoever picks this up next. It covers pass 2 of the build brief
(`fable_brief_2.md`, needs 1-9) and the first slice of the MakerCAD ideas from the PM's
V5 description: the opening screen, the new-project questions, Sancho's rule engine, the
end-of-session log, the themes, the machine profile and the environment list.

This pass was run directly in the project folder, not in a scratch copy. A copy of the
sources as they were before it is at
`C:\Users\<you>\MakerCAD\makercad_v5_backup_2026-11-13_pre-pass2` (no `target/`).

## What changed

### The making path (brief needs 1-8)

| Need | Where | What is there now |
|---|---|---|
| 1. Sketch model | `crates/mc-sketch/src/lib.rs`, `profile.rs` | `Sketch { points, entities, constraints }`. Entities are `Line`, `Arc`, `Circle` over shared point ids, so coincidence is structural. A polyline or rectangle is lines that share points. Construction flag. `rectangle`, `slot`, `arc_through`, `fillet_corner`, `merge_points`, `remove_entity`. `profiles()` walks the planar graph, prunes dangling edges, and returns regions with holes and a nesting depth. |
| 2. Constraints and dimensions | `crates/mc-solver/src/lib.rs` | Own Levenberg-Marquardt, pure Rust, no dependencies. Coincident, point-on-curve, midpoint, horizontal, vertical, parallel, perpendicular, tangent, equal, concentric, fix; distance, horizontal distance, vertical distance, radius, diameter, angle. `solve`, `try_add`, `set_dimension`, `drag`, `dof`. A conflicting or redundant constraint is refused with a reason and the sketch is byte-for-byte unchanged. DOF is unknowns minus Jacobian rank. |
| 3. Arcs reach the kernel | `crates/mc-kernel/shim/shim.cpp`, `src/lib.rs` | `Shape::extruded_profile(loops, dz)`: real line and arc edges into wires, a face with holes, a prism. An arc is one cylindrical face. |
| 4. Extrude modes | `crates/mc-doc/src/lib.rs` | The `Sketch` feature grew `sketch`, `op` (Join / Cut / New), `flip`, `extent` (Blind / ThroughAll / UpToFace) and `regions`, all `#[serde(default)]`, so pass-1 files load. A sketch with height 0 is a drawing and builds nothing. |
| 5. Picking | `crates/mc-view/src/lib.rs`, `crates/mc-ui/src/viewport.rs` | `pick(mesh, camera, screen_point, w, h) -> Option<(face_id, distance)>`, `pick_hit`, `pick_polyline` for edges. The viewport highlights the face or edge under the pointer; click selects, Shift adds. Fillet, chamfer and shell take picked edges/faces; "upright edges" and "open the top" remain as shortcuts. A pick is saved as a place (edge midpoint, face centroid) and matched to the nearest edge or face within 1 mm on each rebuild. |
| 6. Snaps | `crates/mc-sketch/src/snap.rs` | Endpoint, midpoint, centre, intersection, on-curve, grid, in that priority. The viewport shows the snapped point and names the rule that won. While drawing a line, a nearly level or upright line becomes exactly that and gets the constraint. |
| 7. Undo everywhere | `crates/mc-ui/src/lib.rs` | Every command pushes one undo step; a refused command pushes none. Ctrl+Z / Ctrl+Y restore the document and the rebuild. A dragged number is one step, not one per frame. Redo is new. |
| 8. Keep what works | - | The 45 pass-1 tests still pass, the layer gate passes, the Pi template still rebuilds into two bodies. |

Also in the kernel: `edges`, `edge_polyline`, `fillet_edges`, `chamfer_edges`,
`shell_faces`, `mirrored`. Also in the document: `Mirror` and `Pattern` features,
`Plane::from_face`, `Frame::to_world` / `to_plane`, `rebuild_until`.

### The MakerCAD shell (beyond the brief - strike what is not wanted)

- **Opening screen**: Start something new / Continue (recent projects in
  `Documents\MakerCAD`) / open a file / open the example. `crates/mc-ui/src/screens.rs`.
- **New-project questions**: name, goal, tools, materials (with sizes and counts), the
  space it must fit, deadline, inspiration, environments. Stored as `Document::brief`
  (`mc-doc`), and the goal opens the project log in the maker's own words.
- **Sancho** (`crates/mc-assist/src/lib.rs`): a pure rule engine over the document, the
  brief and measured facts. Rules: failed steps, the sketch on the table (open / free /
  fully constrained), too big for the measured space, the material budget (longest stock,
  total volume, percentage used and a waste reminder), the deadline countdown, thin walls
  for printers. He is drawn at true height (1500 mm) at the back of the workspace, so a
  bench-top part and a dresser read differently at a glance.
- **End of session**: "End session", or closing the window with unsaved work, asks what
  got done, what worked, what did not and why, what is next. Answers go in the log;
  skipping marks the log Incomplete.
- **Themes**: Dreamy (default), Pastel, Workshop. `crates/mc-ui/src/theme.rs`.
- **This computer**: a Light / Standard / Full display profile chosen from the core count
  and stepped down automatically when frames stay slow. `crates/mc-ui/src/machine.rs`.
- **Environments**: a list a project loads from. Solid and Sketch are built; Folding,
  Animation/physics, Scan data, 3D print prep, CNC and Circuits are listed as not built.
- **Exports**: DXF now writes true ARC and CIRCLE entities; a true-size SVG cut template
  with a 100 mm scale bar (`mc_io::cut_template_svg`); the cut list uses the maker's own
  largest stock as the sheet.
- `mc-desktop --demo` starts inside a sketch in progress, for showing and checking.

### Bugs found and fixed on the way

1. **`Hole` drilled into thin air.** The pass-1 `build_hole` put the cylinder on the
   outward side of the plane, so the Pi template's four M2.5 pilots and its audio jack cut
   nothing. Fixed, with a test that a hole from the top removes pi r^2 x depth.
2. **Tangent at a shared endpoint is a double root.** "Distance from centre to line equals
   radius" has zero slope exactly at the answer, so a filleted corner read 2 DOF too many.
   Where the line ends on the arc's own end the solver now states it as "the radius meets
   the line square".
3. **The DXF of a sketch on a side plane was flattened onto world XY**, collapsing it to a
   line. Sketches now go out in their own plane coordinates.
4. The shim's edge walk visited each edge once per face; picked edges use a unique map.
5. `build.rs` printed the literal text `{copied}`; it now prints the count.
6. **`mc_shape_split` used a pointer after the handle table could have moved.** It kept
   `ShapeEntry*` across an `add_shape()` that can reallocate the table. The pass-1 split
   test failed once in this pass's full-suite runs because of it. The shim now takes its
   own handle on the solid first; the kernel suite then passed 6 runs in a row.

### Files, one line each

- `crates/mc-sketch/src/lib.rs` - sketch model, constraint data, editing operations.
- `crates/mc-sketch/src/profile.rs` - segments, loops, closed-region detection.
- `crates/mc-sketch/src/snap.rs` - object snaps and segment intersections.
- `crates/mc-sketch/Cargo.toml` - serde added.
- `crates/mc-solver/src/lib.rs` - the constraint and dimension solver.
- `crates/mc-kernel/shim/shim.cpp` - profile extrude, edge queries, picked fillet/chamfer/shell, mirror.
- `crates/mc-kernel/src/lib.rs` - safe wrappers and tests for the above.
- `crates/mc-kernel/build.rs` - the DLL-count message.
- `crates/mc-doc/src/lib.rs` - extrude modes, picks, Mirror/Pattern, the brief, the hole fix.
- `crates/mc-io/src/lib.rs` - DXF arcs, SVG cut template, recent projects, file names.
- `crates/mc-io/Cargo.toml` - depends on mc-sketch.
- `crates/mc-view/src/lib.rs` - `ray`, `unproject_plane`, `look_along`, `pick`, `pick_hit`, `pick_polyline`.
- `crates/mc-assist/src/lib.rs` - Sancho's rules, project scale, date arithmetic.
- `crates/mc-ui/src/lib.rs` - app state, undo/redo, commands, panels (rewritten).
- `crates/mc-ui/src/sketcher.rs` - the sketch environment.
- `crates/mc-ui/src/viewport.rs` - drawing and model picking.
- `crates/mc-ui/src/screens.rs` - opening screen, questions, end of session.
- `crates/mc-ui/src/theme.rs`, `machine.rs` - colours; display profile and environments.
- `crates/mc-ui/Cargo.toml` - depends on mc-sketch, mc-solver, mc-assist.
- `apps/mc-desktop/src/main.rs` - `--demo`.

Nothing was copied from solidcraft or cadcraft. The snap taxonomy and its order follow
CADCraft, which the existing "ideas adopted" row in `docs/ATTRIBUTION.md` already covers;
no attribution row was added because no code was taken.

## How it was checked

All on the reference machine (i5-6600, HD 530, 8 GB), debug build, from the project root.

| Command | Result |
|---|---|
| `cargo run -q -p mc-cli -- layers` | `layers ok: 57 dependency edges checked` |
| `cargo test -p mc-sketch` | 25 passed |
| `cargo test -p mc-solver` | 15 passed |
| `cargo test -p mc-doc` | 20 passed |
| `cargo test -p mc-io` | 11 passed |
| `cargo test -p mc-kernel` | 25 passed |
| `cargo test -p mc-view` | 13 passed |
| `cargo test -p mc-assist` | 8 passed |
| `cargo test -p mc-ui` | 24 passed |
| `cargo test --workspace` | 144 passed, 0 failed (was 45) |
| `target\debug\mc-desktop.exe --template --frames 120` | exit 0, 121 frames, mean update 13.93 ms (was 24.69), worst 160 ms, first frame 3846 ms. A second run straight after a rebuild: mean 12.67 ms, but first frame 47 s while Windows loaded the freshly written DLLs - startup time is not under control yet |
| `python gates/increment.py` | see the last line of this section |

The acceptance tests the brief names, and the numbers they hold:

- Rectangle of four lines is one loop of 800 mm^2; a circle inside it gives a plate with
  one hole (800 - 25 pi) plus the disc; a dangling line adds no loop.
- A rough four-line rectangle solves to (0,0) (40,0) (40,20) (0,20) within 1e-6; a 50 mm
  diagonal on it is refused as a conflict and the sketch is unchanged; a second 40 mm on
  the opposite edge is refused as redundant; the constrained sketch reports 0 DOF.
- The 40 x 10 slot extruded 5 mm measures 1892.70 mm^3 against the analytic
  (30 x 10 + 25 pi) x 5 = 1892.70, with 6 faces of which 2 are cylinders. **The brief
  quotes 1963.5 mm^3 for this slot; that figure is 625 pi and is not this shape.** The
  test holds the analytic value to 0.01 %.
- Cut through-all removes a 5 mm radius hole from plates 3, 20 and 75 mm thick, each
  within 0.05 mm^3. Up-to-face on a shelled box removes exactly the 2 mm wall (200 mm^3).
- `pick` from above hits the top face at the exact eye distance; from the front hits the
  front face at y = 0; beside the part hits nothing.
- One test per snap kind, and one that an endpoint beats a slightly nearer midpoint.
- Driven the way the pointer drives it (`mc-ui`, `sketcher.rs` tests): draw a rough
  rectangle, fix a corner, dimension it 40 x 20, round four corners at R5, give it 8 mm:
  volume (800 - 4(25 - 25 pi / 4)) x 8 within 0.01. Change the 40 to 60 afterwards and the
  solid follows. Pick an edge, fillet; pick a face, shell; undo three times to nothing,
  redo three times back.

Looked at by eye, from window captures: the example in model mode, `--demo` in sketch
mode (geometry, constraint marks, dimension values, DOF line, Sancho's tip), and the
opening screen. Three things were fixed from looking: Sancho was hidden on the Light
profile, the top view was turned 45 degrees, and the volume read "-0.0".

`python gates/increment.py`: OK - the increment is on disk and its suites pass.

## Known limits

- The window was captured, not operated: no click, drag or keystroke has been sent to the
  real window. The sketch tools are tested through the functions the pointer calls, not
  through the pointer. The new-project questions and end-of-session screens have not been
  seen on screen at all.
- Release build timings are still not measured. The frame numbers above are debug.
- Lines that cross without a shared point are not split there; regions are found only
  where entities meet at points.
- No trim, extend, offset, sketch mirror or sketch pattern. No ellipse, spline, polygon
  tool or text.
- No collinear, symmetry or G2 constraints. Horizontal/vertical apply to lines, not to
  pairs of points.
- A dimension cannot be dragged or placed; its label sits at the midpoint of what it
  measures. Dimensions are numbers, not named parameters or expressions.
- DOF is a rank estimate; at a degenerate configuration it can read one high.
- A sketch can only go on a face that looks along X, Y or Z. Slanted faces are refused.
- The camera cannot look from underneath (the ground barrier), so a sketch on the Bottom
  plane is drawn from the side.
- A saved pick finds the nearest edge or face within 1 mm. A larger upstream change loses
  the pick and the feature says "pick them again"; there is no topological naming.
- Up-to-face measures from one point inside the first region, against the display mesh.
- Regions of a sketch are chosen from a checkbox list, not by clicking them.
- Hidden edges are drawn on top of the solid; there is no hidden-line removal.
- Revolve, sweep, loft, draft and taper are not built.
- Mirror and Pattern copy whole bodies, not features.
- Scan data, edge detection from scans, the folding environment, animation/physics, 3D
  print prep, CNC, circuits: listed as environments, none built. Unloading an environment
  hides its tools; nothing is loaded lazily yet.
- The mood board is a list of text lines. Images are not shown or projected into the scene.
- Material use is a volume budget and a longest-piece check. Laying cuts out on the actual
  boards, and drawing cuts onto stock, are not built.
- Blueprints (dimensioned drawings), build order, and build photos/video are not built.
  The build log exports as Markdown; it does not yet include the brief.
- Sancho speaks through text only. No voice, no AI.
- The hardware probe is the core count and measured frame time. It does not read RAM or
  the GPU.
- The no-OpenCASCADE stub (`shim/stub.cpp`) does not define the new shim functions, as it
  did not define most of the old ones.
- `docs/ATTRIBUTION.md` and `docs/DECISIONS.md` still name planegcs as the solver. The
  brief required an own solver with no new dependencies, and that is what is in the tree;
  those two rows are the PM's to change.
- `mc-engine`, `mc-log`, `mc-sheet`, `mc-fold`, `mc-geom`, `mc-id` are still stubs. The
  undo stack and Sancho wiring live in `mc-ui`; they belong in `mc-engine`.

## Addendum, 2026-11-14: the sketch-to-solid flow, after the PM tried it

The PM drew lines that would not become a shape to extrude, and objected to the
floor being assumed as the place to sketch. Reference read: SolidCraft's own guide
(`docs/book/src/guide/ui-tour.md`, `sketching.md`, `command-guides/solid.extrude.md`
in the local clone) and the Autodesk "Fusion 360 UI" step-by-step PDF.

### What changed

- **Clicks are no longer lost.** A test that drives real pointer events through egui
  showed the cause: a click during which the hand moved more than 6 pixels placed
  nothing and orbited the camera instead. Drawing tools now place a point wherever the
  left button comes up. The left button never orbits.
- **Navigation is SolidCraft's**: right drag orbits, middle drag (or Shift + right
  drag) pans, wheel zooms. The camera may go under the floor (pitch -89 to 89 degrees),
  so side planes are seen level and the underside can be looked at.
- **Create Sketch** shows the three origin planes (Top XY, Front XZ, Right YZ); the
  one under the pointer lights up; a click on a plane or on a flat face starts the
  sketch. The grid lies on the sketch plane while sketching, at the snap spacing.
- **Profiles.** A closed shape is shaded the moment it closes. Loose ends of an
  outline are ringed in red and counted in the panel.
- **Extrude (E)** is a dialog on the right with a live solid: click shaded shapes to
  choose them, set distance / direction / operation, Enter accepts, Esc cancels. One
  undo step. On an open outline it reopens the sketch and says how many ends are loose.
- **Toolbar** is a ribbon that follows the work: CREATE and MODIFY in the model,
  SKETCH while sketching, OK / Cancel while a dialog is open. A **timeline** row of
  features sits along the bottom (click selects, double-click edits a sketch).
- Sancho is drawn only when he fits in the view; zoomed in on a small part his coat
  filled the background.

### How it was checked

`cargo test --workspace`: 148 passed, 0 failed (mc-ui 28). New: four mouse clicks and
a fifth on the start close a shape, with and without a 9 pixel slip; Create Sketch
waits for a click on a plane; Extrude previews, cancels, and is one undo step; an
open outline reopens the sketch. Layers gate and `gates/increment.py` pass. The
sketch, plane-pick and Extrude screens were captured and looked at; the window was
still not operated by hand.

### Known limits

- No value boxes at the cursor while drawing, no Dimension tool with click-to-place;
  dimensions are still typed in the right panel.
- No view cube, no marking menu, no command search, no browser tree of origin /
  bodies / sketches; the left panel is still the history and parameter editor.
- No drag arrow on the extrusion; no symmetric, two-sided or tapered extent.
- Sketches still go only on faces that look along X, Y or Z. Back, Left and Bottom
  faces open rotated half a turn.
- The pasted Fusion screenshot arrived as a 50 x 2000 pixel image and could not be
  read; the DelDOT guide is images only; the Autodesk help link is a dead page.

## Addendum 2, 2026-11-14: after reading the PM's 45 Fusion help screenshots

Read: `Documents\ShareX\Screenshots\2026-11\Fusion - (1..45).png`: the Fusion
interface page, Home, Workspaces, "Perform basic tasks", "Walkthrough: Create a
design", and the tutorial "Fully defining a sketch" (the wall outlet).

### What changed

- **Sketch origin.** A new sketch holds a pinned origin point. A shape drawn from it
  stays put, so a dimension sizes it instead of sliding it; "fix a corner first" is gone.
- **Dimension tool (D).** Click a line, a circle or arc, two points, two level or two
  upright lines, or a point and such a line; a value box opens at the pointer with the
  size as drawn; type and press Enter. Dimensions are drawn with witness lines and arrows.
- **Constraints by clicking.** A constraint button with nothing selected is picked up
  as a tool: click the things it relates and it applies. Horizontal / Vertical also
  takes two points (two centres lined up).
- **Centre rectangle**, with its diagonals as construction lines. **X** toggles
  construction. **Show constraints** checkbox.
- **Browser** on the left: Origin (click a plane to sketch on it), Sketches, Bodies.
- **View buttons** (Home, Top, Front, Right) where Fusion keeps its cube, and a
  **right-click menu** in the model view.

### How it was checked

`cargo test --workspace`: 149 passed, 0 failed. New test
`the_wall_outlet_tutorial_can_be_followed` drives the opening of Fusion's tutorial:
centre rectangle on the origin, 70 and 114 typed into the Dimension box, a 12 mm hole,
two centres lined up with Vertical, a point-to-line dimension; it ends fully defined.
Sketch and model screens captured and looked at; the window was not operated by hand.

### Known limits against those pages

- No Mirror in a sketch, no box selection, no trim or offset: the tutorial's cutout
  cannot be mirrored to the lower half yet.
- No real ViewCube (four buttons instead), no S command search, no workspace switcher,
  no eye icons in the Browser, no play / step on the timeline.
- Box, Cylinder and Hole are still placed by numbers, not by clicking a plane or face
  and dragging; Fillet has no dialog with a live preview.
- Line-to-line distance works for level and upright lines only.

## Addendum 3, 2026-11-14: sketch Mirror

- New constraint `Symmetric { a, b, line }` (two points, mirror images in a line) in
  `mc-sketch` and `mc-solver`. `Sketch::mirror(entities, line)` copies lines, arcs and
  circles across a line and returns the ties (Symmetric per point, Equal per circle);
  points on the mirror line are shared.
- **Mirror** tool in the sketch ribbon: click shapes or drag a box round them, Enter
  (or "Pick the mirror line"), click the line. With shapes and exactly one construction
  line already selected it mirrors at once. One undo step. The copy follows the original.
- **Box selection**: a left drag that starts on nothing selects what is wholly inside.
- Checked by `a_mirrored_shape_is_a_copy_that_follows_its_original` (150 tests pass,
  layers and increment gates pass). Not looked at on screen and not operated by hand.
- Limits: the mirror line must be a line of the sketch (not an axis); symmetry marks
  are not drawn; no live preview before the line is clicked.


## 2026-11-14 (Gordon) - display modes, Select Other, and the Fusion notes

### What changed

- `crates/mc-view/src/lib.rs`: `occluded`, `split_visibility`, `pick_all`, `Run`, `Tri3`.
  A point on an edge is hidden when a triangle stands between it and the eye; a change of
  state is found by bisection, and a run shorter than a quarter of a sample is absorbed as
  noise (found by the 3/4-view check: a silhouette corner reported 0.05 mm of "visible").
- `crates/mc-ui/src/viewport.rs`: `draw_bodies` honours three modes; `ensure_visibility`
  caches the runs by a camera+model signature and keeps the old answer while the camera is
  moving; `face_hits_under` backs Select Other; `edge_stroke` keeps hover and selection.
- `crates/mc-ui/src/lib.rs`: `DisplayMode` (Shaded visible edges by default, Shaded all
  edges, Wireframe) in the View menu, plus a line telling the user to right-click for what
  is behind. The mode is a view setting, never saved into the document.
- Right-click on the model now offers **Select other... (N behind)**: every face under the
  pointer, nearest first, with its distance; clicking picks it.
- `C:\Users\<you>\MakerCAD\tools\ocr_screenshots.py`: local OCR of the PM's Fusion
  screenshots through Windows' own engine (`winsdk`), writing `ocr\*.txt` and
  `ocr\INDEX.md` beside the images.
- `docs/FUSION_NOTES.md`: what those pages specify, page by page, and the gap table against
  the program - including the sketch state rule (blue when under-constrained, black when
  fully defined) that we do not have yet.

### How it was checked

- `cargo test -p mc-view` - 18 pass, 5 of them new: the 3/4 view of a box shows 9 edges and
  hides 3; from straight above the top ring is visible and the bottom ring is not; an edge
  half behind a body splits into visible/hidden/visible with the hidden run 20 mm within
  1.5 mm; a point lying on its own face is not called hidden; Select Other lists the front
  face before the back.
- `cargo test -p mc-ui` - 34 pass, 3 new (headless): every mode draws without panicking; the
  cache fills, holds while nothing moves, and follows the camera; a mode never changes the
  model's volume.
- Pi template, visible runs: 300 pieces. 387 ms for the first computation in a debug build
  before the sampling cap; 390 ms worst frame after it (the cache fill).
- Smoke run, debug build, `--template --frames 60`: exit 0, first frame 4692.6 ms, mean
  update 19.25 ms, worst 390.1 ms.

### Known limits

- Hidden edges are **dropped**, not drawn dashed or faint. Fusion has a dashed option; this
  does not.
- The runs are recomputed when the camera stops, not every frame: during an orbit the
  picture can be up to one mouse-move out of date. Deliberate, and the reason the mean
  update stays at 19 ms.
- The mode is not remembered between restarts.
- No release-build timing. Every number above is a debug build.
- The app was running while this was built, so the new binary exists only in
  `target\gordon-build\debug\`; `target\debug\mc-desktop.exe` is still the old one because
  Windows locks a running executable.


## 2026-11-14 (Gordon, later) - typed sizes while drawing

### What changed

- `crates/mc-ui/src/sketcher.rs`: `SizeBox` and `sketch_click_sized`. After the first
  click of Rectangle, Centre Rectangle, Circle or Line, a small box follows the pointer
  showing the size it would be: W/H, D, or L/A. Typing takes over from the pointer; Tab
  moves to the next box; Enter places the shape at the typed size and leaves **driving
  dimensions** behind (HDistance + VDistance, Diameter, or Distance). Escape closes the box
  and keeps the first point. One shape is still one undo step.
- `crates/mc-ui/src/viewport.rs`: the boxes are drawn at the pointer with their labels.
- `docs/FUSION_NOTES.md`: corrected. It claimed the sketch had no blue/black defined state;
  it does (palette says "fully constrained" or "N degrees of freedom", entities go from
  `p.sketch` to `p.sketch_locked` at 0 DOF), so the note now says so and records that the
  three rules are to be kept true rather than added.

### How it was checked

- `cargo test -p mc-ui typing_a_size_places_the_rectangle_and_drives_it_with_dimensions`:
  one click, type 60, type 40, Enter -> a 60 x 40 rectangle, at least two constraints left
  behind, and exactly one undo step.
- `cargo test --workspace`: 160 passed, 0 failed (was 159).
- Smoke run `--demo --frames 40`: exit 0, mean update 15.89 ms, worst 189.1 ms.

### Known limits

- The boxes are a strip of two TextEdits, not Fusion's paired boxes with a Tab arrow; Tab
  cycle is handled by us, not by egui's focus order.
- A typed Angle on a Line sets the direction only; it is not left as an angle dimension
  (the solver's Angle constraint needs two lines).
- Negative sizes are refused with a message rather than flipping the shape.
- Still debug timings only.
