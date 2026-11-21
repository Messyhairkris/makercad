# MakerCAD v5 - build plan (pass 1, architecture)

2026-11-12, Fable pass 1. Target: one usable program that models a Raspberry Pi 4B
enclosure from the UI and exports STL, DXF, a cut list and its own log.
Nothing here was compiled or run; every statement about existing code comes from reading it.

## 0. What reading the code changed

| # | Found | Consequence |
|---|---|---|
| 1 | `cargo run -p xtask -- layers` does not exist: `xtask` is the lib `xtask-layers`. | The gate is `cargo run -q -p mc-cli -- layers` (and `cargo test -p xtask-layers`). |
| 2 | No JSON crate anywhere in the workspace. | `.makercad` needs a hand-written std-only JSON module (`mc_io::json`, about 250 lines). |
| 3 | `build.rs` links 13 toolkits; none holds fillet or offset. | Add `TKFillet` and `TKOffset` to the link list. Same OCCT install, no new dependency. |
| 4 | `shim/stub.cpp` defines 3 symbols; `lib.rs` declares 15. | The no-OCCT build cannot link today. The stub gains every verb (each returns 0). |
| 5 | `mc_shape_tessellate` skips `++face_id` when a face has no triangulation. | Triangle face ids drift from face-list order. Increment before `continue`. |
| 6 | `mc_shape_extrude` and `mc_shape_cut_hole` return 0 without `set_error`; `g_last_error` is never cleared. | Stale reasons. Every verb clears the error on entry and sets it on every failing path. |
| 7 | `mc_shape_free` blanks the slot but never reuses it. | Add a free list; a session of rebuilds otherwise grows the table forever. |
| 8 | `sancho_rules::Fact` is `{width, depth, height, hole_radius, stock}`. | It cannot say "that wall is 1.2 mm". `mc-assist` adds enclosure rules beside it (section 5). |
| 9 | `log_render` replaces non-ASCII with `?`. | Log templates are ASCII: `x` not `×`, `-` not an em dash. |
| 10 | `mc-ui` rebuilds on the UI thread in `App::new`, and `draw_part` rebuilds two `HashMap`s and emits one egui shape per triangle every frame. | Rebuild moves to a worker; per-face data is computed once per rebuild (section 5). |
| 11 | `Feature.kind` is `&'static str`. | It cannot be loaded from a file. The inline `Doc`/`Feature`/`Cmd` in `mc-ui` are deleted, not extended. |
| 12 | `DESIGN.md` has sections A-F, no section 11. | Decisions were taken from `DESIGN.md` and `docs/DECISIONS.md`. |
| 13 | The two crew modules are not in the scratch copy. | Their APIs are taken from the contracts; copy the files verbatim to `mc-assist/src/rules.rs` and `mc-log/src/render.rs`. |

## 1. Kernel additions

Rules for every verb: take `g_lock`; clear the error; `try { OCC_CATCH_SIGNALS ... }` with
`catch (Standard_Failure&)` and `catch (...)`; return 0 and `set_error` on failure. `build.rs`
adds the flag `/EHa`, and the shim calls `OSD::SetSignal(false)` once per thread, so an access
violation inside OCCT becomes a caught failure instead of a dead process (**assumed to work on
this MSVC build; step 1 proves it**). A result is accepted only if `BRepCheck_Analyzer` calls
it valid.

| C signature | OCCT class | Returns | Fails when |
|---|---|---|---|
| `int mc_shape_box_at(double x,double y,double z,double dx,double dy,double dz)` | `BRepPrimAPI_MakeBox(gp_Pnt,...)` | handle | a size <= 0 |
| `int mc_shape_cylinder(const double* axis6,double r,double h)` (origin, direction) | `BRepPrimAPI_MakeCylinder(gp_Ax2,r,h)` | handle | r or h <= 0, zero direction |
| `int mc_shape_extrude_on(const double* frame9,int n,const double* xyb,double dist)` (origin, u, v; vertices `x,y,bulge`) | `BRepBuilderAPI_MakeEdge`, `GC_MakeArcOfCircle`, `MakeWire`, `MakeFace`, `BRepPrimAPI_MakePrism` along `u x v` | handle | n < 2, wire open or self-crossing, dist = 0 |
| `int mc_shape_compound(const int* hs,int n)` | `BRep_Builder::MakeCompound` | handle | unknown handle |
| `int mc_shape_common(int a,int b)` | `BRepAlgoAPI_Common` | handle (volume 0 if disjoint) | boolean not done |
| `int mc_shape_transform(int h,int kind,const double* p7)` (0 translate xyz; 1 rotate: point, direction, radians; 2 mirror: point, normal) | `gp_Trsf` + `BRepBuilderAPI_Transform(copy=true)` | handle | zero direction |
| `int mc_shape_bbox(int h,double* out6)` | `BRepBndLib::AddOptimal` into `Bnd_Box` | 1, fills min xyz, max xyz | void shape |
| `int mc_shape_shell(int h,double t,const int* open,int n_open)` | `n_open>0`: `BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin(S, faces, -t, 1e-3)`. `n_open=0`: `BRepOffsetAPI_MakeOffsetShape::PerformByJoin(S, -t, 1e-3)`, then `BRepAlgoAPI_Cut(S, inner)` | handle | t too large, offset self-intersects, result invalid or not smaller |
| `int mc_shape_fillet(int h,const int* edges,int n,double r)` | `BRepFilletAPI_MakeFillet` | handle | radius too large for a neighbouring face, edge index unknown |
| `int mc_shape_chamfer(int h,const int* edges,int n,double d)` | `BRepFilletAPI_MakeChamfer::Add(d, edge)` | handle | as fillet |
| `int mc_shape_split_plane(int h,const double* plane6,int* below,int* above)` (point, normal) | `BRepBuilderAPI_MakeFace(gp_Pln)`, `BRepPrimAPI_MakeHalfSpace`; above = `Common`, below = `Cut` | 2 both sides, 1 plane missed the body, 0 failed | boolean not done |
| `int mc_shape_face_count(int h)`; `int mc_shape_face_info(int h,int i,double* out8)` | `TopExp_Explorer` (same order as tessellate), `BRepAdaptor_Surface`, `BRepGProp::SurfaceProperties` | type (0 plane, 1 cylinder, 2 other), area, centroid xyz, outward normal xyz (zero if not a plane) | index out of range |
| `int mc_shape_edge_count(int h)`; `int mc_shape_edge_info(int h,int i,double* out11)` | `TopExp::MapShapes(EDGE)` cached in `ShapeEntry`, `BRepAdaptor_Curve` | type (0 line, 1 circle, 2 other), length, midpoint, start, end | index out of range |

Bulge is DXF's: `tan(angle/4)`, positive = counter-clockwise; arc midpoint =
chord midpoint + `(bulge/2) * (dy, -dx)`. Fillet and chamfer take the `edge_info` indices.

Rust side (`mc-kernel/src/lib.rs`): new methods borrow (`fn cut(&self, tool: &Shape)`,
`fuse`, `common`, `shell`, `fillet`, `chamfer`, `split_plane`, `transformed`, `bbox`, `faces()
-> Vec<FaceInfo>`, `edges() -> Vec<EdgeInfo>`), because the rebuild cache keeps every
intermediate body alive. The consuming methods stay for the 7 existing tests. Add
`KernelError::Failed(String)` carrying `last_error()`. `Tri`, `FaceInfo`, `EdgeInfo` are plain data.

**Named fallback for closed shell:** if `MakeOffsetShape` fails its step 1 test, `Shell` with
`open: None` is refused in words and the template's feature 3 becomes an `Extrude` Cut of the
inset rectangle (one line in `templates.rs`).

## 2. `mc-doc` data model

```rust
pub const FILE_VERSION: u32 = 1;

pub struct Document {
    pub version: u32,
    pub name: String,
    pub created: u64,              // unix seconds, std::time only
    pub model: Model,              // the only part undo touches
    pub materials: Vec<Material>,
    pub log: Vec<LogEntry>,        // append-only; an undo adds an entry, never removes one
    pub voice: Voice,              // On | Silent | Mute
}
pub struct Model {
    pub params: Vec<Param>,        // ordered; an expr may name earlier params only
    pub features: Vec<Feature>,    // the timeline, top to bottom
    pub bodies: Vec<BodyMeta>,     // what the user said about a body; never geometry
    pub next_id: u32,              // ids are never reused
}
pub struct Param { pub name: String, pub expr: String, pub note: String }
pub struct Num(pub String);        // "26" or "wall*2"; evaluated at rebuild

pub struct Feature { pub id: u32, pub name: String, pub suppressed: bool, pub kind: FeatureKind }
pub enum FeatureKind {
    Extrude { plane: PlaneRef, profile: Profile, dist: Num, op: Op, target: Target, grid: Grid },
    Shell   { target: Target, thickness: Num, open: Option<FaceSel> },
    Fillet  { target: Target, edges: EdgeSel, radius: Num },
    Chamfer { target: Target, edges: EdgeSel, dist: Num },
    Split   { target: Target, plane: PlaneRef },
}
pub enum Op { New, Add, Cut, Split }            // Split = split by the sketch profile
pub enum Target { All, Body(BodyId) }
pub struct BodyId { pub feature: u32, pub part: u8 }
pub struct Grid { pub nx: u32, pub ny: u32, pub dx: Num, pub dy: Num }   // 1 x 1 = one copy

pub enum Side { Top, Bottom, Front, Back, Left, Right }
pub enum Axis { X, Y, Z }
pub enum PlaneRef {
    Base { side: Side, offset: Num },
    Face { body: BodyId, face: FaceSel, offset: Num },
}
pub enum FaceSel { Extreme(Side), Near { point: [f64; 3], normal: [f64; 3] } }
pub enum EdgeSel { Parallel(Axis), Near(Vec<[f64; 3]>) }   // Near = picked edge midpoints

pub enum Profile {
    Rect    { cx: Num, cy: Num, w: Num, h: Num, r: Num, ring: Num },
    Circle  { cx: Num, cy: Num, r: Num },
    Polygon { pts: Vec<[f64; 2]>, dims: Vec<EdgeDim> },
}
pub struct EdgeDim { pub edge: usize, pub len: Num }

pub struct BodyMeta { pub id: BodyId, pub name: String, pub visible: bool,
                      pub material: Option<u32>, pub printed: bool, pub qty: u32 }
pub struct Material { pub id: u32, pub name: String, pub kind: MatKind,   // Sheet | Board | Filament
                      pub thickness: Option<f64>, pub density: f64 }      // mm, g/cm3
pub struct LogEntry { pub id: u32, pub t: u64, pub feature: Option<u32>,
                      pub what: String, pub why: Option<String>, pub auto: bool }
```

Everything derives `Clone, Debug, PartialEq`.

- `Rect` is one variant for rectangle, rounded rectangle and slot (`r = h/2`). `ring > 0`
  makes it a hollow frame of that width. "Hole" and "Slot" are UI tools that write an `Extrude`.
- **Plane frame, one rule:** a plane is a unit normal `n` and a distance `d`. `u = +x` when
  `|n.z| > 0.99`, else `normalize(z cross n)`; `v = n cross u`; origin `n*d`. Normals: Top +z,
  Bottom -z, Front -y, Back +y, Left -x, Right +x. So Top sketches are in world x,y; Front is
  (x, z); Right is (y, z); Left is (-y, z). A `Face` plane uses the face's normal and
  `d = centroid.n + offset`.
- `Extrude` sweeps the profile from its plane along `n` by `dist`; negative goes inward.
- Expressions: `+ - * / ( )`, unary minus, `min(a,b)`, `max(a,b)`, parameter names.
  `mc_doc::eval(&str, &[(String, f64)]) -> Result<f64, String>`, about 120 lines.

**Why it survives save/load and undo.** The document is owned plain data. No kernel handle,
no face or edge index, no pointer. Selections are geometric (`Extreme`, `Near`, `Parallel`)
and are resolved again on each rebuild, so a reloaded or edited tree finds the same face.
Ids come from `next_id` and are never reused, so a `Target` or log link cannot point at the
wrong thing. Geometry, meshes and failure reasons live in `Built` (section 4) and are never
saved. Undo is `Vec<Model>` snapshots (cap 100) in `mc-engine`; a model is a few kilobytes.

**Commands** (`mc_doc::Cmd`, with `apply(&mut Document, Cmd) -> Result<String, String>`
returning the log line): `AddFeature`, `SetFeature(id, FeatureKind)` (every dimension edit),
`RenameFeature`, `RemoveFeature`, `MoveFeature(id, index)`, `Suppress(id, bool)`,
`SetParam(name, expr)`, `RemoveParam`, `SetBody(BodyMeta)`, `InsertTemplate(String)`,
`LogNote{what, why}`, `EditLog{id, what, why}`, `SetVoice`.

**Polygon dimensions, the named fallback (no solver, no matrix library).**
`mc_sketch::set_edge_length(pts, edge, len)`: moves the edge's end vertex along the edge and
translates every later vertex by the same delta; the closing edge absorbs the change. Closed
form, one dimension. With two or more `dims`: `mc_sketch::relax_lengths`, sequential
edge-length relaxation (Gauss-Seidel, at most 50 passes, tolerance 1e-6 mm); if it does not
converge the feature is broken with "these dimensions fight each other". `mc-solver` stays a stub.

## 3. The Pi 4B enclosure template

`mc_doc::templates::pi4b() -> (Vec<Param>, Vec<Feature>)`, inserted by
`Cmd::InsertTemplate("pi4b")`. After insertion these are ordinary features and parameters.

Parameters (mm). A = assumed, not from the Raspberry Pi mechanical drawing.

| Name | Value | | Name | Value | |
|---|---|---|---|---|---|
| `board_l` | 85 | | `wall` | 2 | A |
| `board_w` | 56 | | `clr` (board to wall) | 0.5 | A |
| `pcb_t` | 1.6 | A (nominal) | `corner_r` | 3 | A (board corners are 3) |
| `hole_dx` | 58 | | `standoff_h` | 3 | A |
| `hole_dy` | 49 | | `standoff_d` | 6 | A |
| `hole_in` | 3.5 | | `pilot_d` (M2.5 into plastic) | 2.2 | A |
| `comp_h` (tallest part, USB stack) | 16 | A | `split_z` | 14 | A |
| `headroom` | 1.4 | A | `lip_h` | 2 | A |
| | | | `lip_clr` | 0.2 | A |

Derived parameters: `b = wall+clr` (2.5, board origin in x and y);
`L = board_l+2*b` (90); `W = board_w+2*b` (61);
`H = 2*wall+standoff_h+pcb_t+comp_h+headroom` (26); `zt = wall+standoff_h+pcb_t` (6.6, PCB top);
`li = wall/2+lip_clr/2` (1.1); `lo = wall/2-lip_clr/2` (0.9).

Board frame: origin at the corner by USB-C and microSD, x along the 85 mm edge. Connector
centres along the edges are from the drawing; every opening size and height is assumed.

| # | Name | Feature | Numbers |
|---|---|---|---|
| 1 | Body | Extrude New, Top 0 | Rect c(`L/2`,`W/2`) `L` x `W`, dist `H` |
| 2 | Corners | Fillet, `Parallel(Z)` | `corner_r` (4 edges of a plain box) |
| 3 | Hollow | Shell, closed | `wall` |
| 4 | Base / Lid | Split, Top `split_z` | below = Base, above = Lid |
| 5 | Lip ridge | Extrude Add to Base, Top `split_z` | Rect `L-2*li` x `W-2*li`, r `corner_r-li`, ring `lo`, dist `lip_h` |
| 6 | Lip rebate | Extrude Cut from Lid, Top `split_z-1` | Rect `L-2*lo` x `W-2*lo`, r `corner_r-lo`, dist `lip_h+1.2` |
| 7 | Standoffs | Extrude Add to Base, Top `wall` | Circle c(`b+hole_in`,`b+hole_in`) r `standoff_d/2`, dist `standoff_h`, grid 2 x 2 at `hole_dx`,`hole_dy` |
| 8 | Screw pilots | Extrude Cut from Base, Top `wall+standoff_h+1` | Circle r `pilot_d/2`, dist `-(standoff_h+2)`, same grid (leaves 1 mm of floor) |
| 9 | USB-C | Extrude Cut All, Front 1, dist `-(wall+2)` | Rect c(`b+11.2`,`zt+1.6`) 13 x 7, r 1 |
| 10 | micro-HDMI x2 | same plane | Rect c(`b+26`,`zt+1.5`) 9 x 5, r 1, grid 2 x 1 at 13.5 |
| 11 | Audio | same plane | Circle c(`b+54`,`zt+3.1`) r 3.5 |
| 12 | USB-A x2 | Extrude Cut All, Right `L+1`, dist `-(wall+2)` | Rect c(`b+9`,`zt+8.25`) 14.5 x 16.5, r 0.5, grid 2 x 1 at 18 |
| 13 | Ethernet | same plane | Rect c(`b+45.75`,`zt+7`) 16.5 x 14, r 0.5 |
| 14 | microSD | Extrude Cut All, Left 1, dist `-(wall+2)` | Rect c(`-(b+28)`,`wall+standoff_h-1`) 13 x 3, r 0.5 |
| 15 | Vents | Extrude Cut from Lid, Top `H+1`, dist `-(wall+2)` | Rect c(`L/2-18`,`W/2`) 2 x 24, r 1, grid 9 x 1 at 4.5 |

Result: Base 90 x 61 x 16 (walls to 14, ridge to 16), Lid from z 14 to 26, 0.2 mm lip gap each
side and on top. The ridge and the lid skin are 0.9 mm thick; that is deliberate and exempt
from the wall rule.

**The order differs from the brief's (split last), on purpose.** The fillet runs on a plain
box and the shell on a box with four round corners, where OCCT is most reliable, and the
uniform wall follows the corners. The ports are cut after the lip so the ridge does not
bridge the USB and Ethernet openings. Port and vent tools start 1 mm outside the wall so no
tool face lies on a body face. The brief's order (shell, cutouts, standoffs, vents, fillet,
split) still works by hand in the UI and is the hand test in step 7; a lip added after the
ports needs the two side ports cut again.

## 4. The rebuild pipeline

`mc_doc::rebuild(doc: &Document, cache: &mut Cache, progress: &mut dyn FnMut(usize, usize, &str)) -> Built`

```rust
pub struct Built { pub bodies: Vec<BuiltBody>, pub outcomes: Vec<Outcome>,   // one per feature
                   pub overlaps: Vec<(BodyId, BodyId, f64)>, pub ms: f64, pub reused: usize }
pub struct BuiltBody { pub id: BodyId, pub tris: Arc<Vec<Tri>>, pub faces: Vec<FaceInfo>,
                       pub edges: Vec<EdgeInfo>, pub bbox: [f64; 6], pub volume: f64,
                       pub explode: [f64; 3] }
pub enum Outcome { Ok, Warn(String), Suppressed,
                   Broken { reason: String, detail: String, fix: String } }
```

1. **Resolve.** Evaluate params in order, then every `Num` of every feature. A bad
   expression breaks that feature only ("`wal` is not a parameter").
2. **State.** A state is `Vec<(BodyId, Arc<Shape>)>`. Each feature maps a state to a state.
   A body a feature does not touch keeps the same `Arc`.
3. **Cache.** `key[i] = fnv1a(key[i-1], resolved numbers and selectors of feature i)`
   (`mc_id::fnv1a`, std-only). `Cache` holds `(key, state, outcome)` per feature. Rebuild
   walks from the top, reuses every entry whose key matches, truncates at the first mismatch
   and computes from there. A prefix cache: no eviction policy, no dependency graph.
4. **Evaluate one feature.**
   - Extrude: lower the profile to `x,y,bulge` (`mc-sketch`), build one tool per grid cell
     (`cylinder` for a Circle, `extrude_on` otherwise; a ring is outer cut by inner), join
     the cells with `compound`, then **one** boolean per target body. A body whose bbox does
     not meet the tool's is skipped with no kernel call. New adds a body `{feature: id,
     part: 0}`. Split replaces each target with `cut` (part `2k`) and `common` (part `2k+1`).
   - Shell, Fillet, Chamfer: resolve the selector against `faces()` / `edges()`, call the verb.
   - Split: `split_plane`; parts `2k` (below) and `2k+1` (above); `explode` = minus and plus the normal.
5. **Failure does not poison.** A failed feature gets `Outcome::Broken` and its output
   state **is its input state**. Later features run against the bodies as they were. A broken
   feature is cached like any other, so it is not retried each frame.
   Causes and words: kernel refusal (the reason is ours, `last_error()` goes in `detail`);
   target body gone ("the body this worked on no longer exists"); selector matched nothing
   ("no flat face faces that way any more"); result has zero volume. A Cut that touched no
   body is `Warn("that cut removed nothing")`, not broken.
   Example: `Fillet` -> reason "A 6 mm fillet does not fit on those edges", fix "Try a
   radius under 2 mm, or fillet before shelling".
6. **Finish.** Only for final bodies whose `Arc` changed since the last build: `triangles_faces(0.25)`,
   `faces()`, `edges()`, `bbox()`, `volume()`. With two or more bodies, `common` each pair
   whose bboxes meet and record volumes above 0.01 mm^3 in `overlaps`.

**Budget.** The template is 15 features and about 20 booleans, because a grid is one
boolean. An edit re-runs only the edited feature and those after it. A dimension is rebuilt
when it is committed (Enter or drag release), never per drag tick. The worker drops stale
requests and builds only the newest snapshot. Step 3 measures two numbers: `cold_ms` (empty
cache) and `edit_last_ms` (change the vent width). `edit_last_ms < 300` is asserted. `cold_ms`
is recorded; if it exceeds 300, in order: (a) move meshing off the final step for hidden
bodies, (b) raise deflection to 0.4, (c) accept it for file open only, where the status line covers it.

## 5. UI

`mc-ui` drops its `mc-kernel` dependency and gains `mc-doc`, `mc-io`, `mc-log`, `mc-assist`.
It holds a `mc_engine::Session` and never calls the kernel.

**`mc-engine::Session`**: `doc`, `undo`/`redo: Vec<Model>`, `built: Arc<Built>`, `status:
Option<Status>`, and one worker thread (`std::thread` + `mpsc`). `run(cmd)` snapshots, applies,
appends the log line, and sends the document clone to the worker. The worker owns the
`Cache`, reports `Progress{index, total, name}` before each uncached feature, and also runs
exports. **Anything over 200 ms shows in the status bar** ("Rebuilding 7/15: Lip ridge, 0.4 s");
while a job is pending the UI asks for a repaint every 100 ms. Nothing initialises at
startup except the window: the first frame paints before the first kernel call.

**Layout**

| Where | What |
|---|---|
| Top bar | File (New, Open, Save, Template > Raspberry Pi 4B case, Export > STL, DXF, Cut list, Log), tools, Add / Cut / New, distance field, plane chooser with offset, Explode slider |
| Left | Timeline: one row per feature, red with the reason on hover when broken, eye (suppress), delete. Under it Bodies: name, eye, material, printed, qty. Under it Parameters: name, expression, value |
| Right | Properties of the selected feature; under it the Log tab |
| Bottom | Sancho strip, then the status bar (job status, cursor u,v, active plane) |
| Centre | Viewport |

**Keys.** Kept: `1` rectangle, `2` circle, `3` Add/Cut, `F` frame, `Esc` cancel. New: `4`
polygon, `5` slot, `6` hole, `P` pick plane, `K` shell, `I` fillet, `J` chamfer, `T` split,
`Tab` type dimensions, `Enter` commit, `Backspace` drop last polygon point, `Delete` remove
feature, `M` cycle Sancho voice, `Ctrl+Z/Y/S/O/N`, `Ctrl+E` export STL.

**Sketching.** The active plane is App state (default Top, offset 0) drawn as a grid on that
plane. A click is `Camera::ray` intersected with the plane, in (u, v), snapped to 1 mm (Alt =
free). Rectangle: two clicks, or `Tab` then `W`, `H`, `Enter`. Polygon: click points; click the
first point or press `Enter` to close. Closing a profile writes an `Extrude` immediately with
the toolbar's op and distance, as today.

**Picking a face.** New in `mc-view`, pure maths: `Camera::ray(screen, w, h)`,
`ray_triangle` (Moller-Trumbore), `ray_plane`; `unproject_ground` becomes a call to them. The
UI scans every triangle of visible bodies (under 1 ms at 20k triangles, so no BVH), takes the
nearest hit, and reads `Tri.face` into `BuiltBody.faces`. Hover tints the face. `P` + click
sets the plane to `Face{body, Near{centroid, normal}}`. Edges: nearest `EdgeInfo` segment
within 8 px; `I` or `J` collects them into `EdgeSel::Near`, with an "all vertical" button for
`Parallel(Z)`.

**Editing a dimension after the fact.** Click the feature in the timeline. Properties shows
each `Num` as a text field that takes a number or an expression; `Enter` sends `SetFeature`
and the body rebuilds. Parameters edit the same way. A broken feature shows its reason and
fix above its fields.

**Split shows both halves.** Each body gets its own tint and a row in Bodies with an eye.
The Explode slider (0 to 30 mm, display only) moves each body along its `explode` vector. A
click selects a body and outlines it.

**Sancho** (bottom strip): a drawn portrait, one line of speech, the findings list, the
voice button (On / Silent = list only / Mute = nothing, no repaint cost). He speaks only on
Forbidden, Denied or NotAllowed. `mc_assist::findings(&Document, &Built) -> Vec<Finding>`
uses the crew's `Kind` and `Finding` and joins: `rules::check` per body (bbox as width,
depth, height; largest Circle Cut as `hole_radius`); every `Broken` outcome as Forbidden;
`Shell.thickness < 1.6` Forbidden "that wall is 1.2 mm, too thin to print"; `overlaps`
Forbidden "the lid will not fit: it overlaps the base by 14 mm^3"; `lip_clr < 0.15`
NotAllowed "0.1 mm of clearance is tighter than most printers hold"; a Sheet body whose
thinnest bbox side is not its material thickness, Denied. Limits are constants in
`mc-assist/src/limits.rs`.

**Log** (right tab): the entry list, each `what` and `why` editable, "Add note", "Render".
A new document opens with one question, "What are you making?", which becomes entry 1.
Every command appends its line with `auto: true`. `mc_log::to_markdown(&Document)` maps
entries to `render::Entry` and calls `render(name, &entries)`.

**Viewport cost.** On each new `Built`, once: per-face normals and the outline segments. Per
frame: project, sort, and emit **one** `egui::Mesh`, kept while the camera and build are unchanged.

## 6. File format

```json
{
  "version": 1,
  "app": "MakerCAD 0.1.0",
  "name": "Pi case",
  "created": 1794510000,
  "voice": "on",
  "params": [{ "name": "wall", "expr": "2", "note": "printed wall" }],
  "materials": [{ "id": 1, "name": "PLA", "kind": "filament", "thickness": null, "density": 1.24 }],
  "features": [
    { "id": 1, "name": "Body", "suppressed": false, "kind": "extrude",
      "plane": { "base": "top", "offset": 0 },
      "profile": { "rect": { "cx": "L/2", "cy": "W/2", "w": "L", "h": "W", "r": 0, "ring": 0 } },
      "dist": "H", "op": "new", "target": "all",
      "grid": { "nx": 1, "ny": 1, "dx": 0, "dy": 0 } },
    { "id": 2, "name": "Hollow", "suppressed": false, "kind": "shell",
      "target": { "body": [1, 0] }, "thickness": "wall", "open": null }
  ],
  "bodies": [{ "id": [1, 0], "name": "Case", "visible": true, "material": 1, "printed": true, "qty": 1 }],
  "next_id": 3,
  "log": [{ "id": 1, "t": 1794510000, "feature": null, "auto": false,
            "what": "A case for the Pi 4B on my desk",
            "why": "it must take the official power supply" }]
}
```

A `Num` is written as a JSON number when it parses as one, otherwise as a string.

**Version rule.** `version` is an integer and is required. Missing: refuse, "this is not a
MakerCAD file". Greater than `FILE_VERSION`: refuse, "written by a newer MakerCAD (file
version 3, this program reads up to 1)". Lower: run `migrate_1_to_2`, `migrate_2_to_3`, ... on
the parsed JSON value before decoding. Unknown keys are ignored. The version rises only when
a field changes meaning or a new one has no default. Save writes `name.makercad.tmp` then
renames, so a crash never leaves half a file.

**Exports (`mc-io`).** STL: binary, 84 + 50n bytes, visible bodies. DXF: R12 ASCII,
`POLYLINE`/`VERTEX` with bulge (group 42) and `CIRCLE`; a sketch exports exact arcs; a planar
face exports its boundary loops chained from the display mesh, so arcs on a face are short
segments. Cut list: `mc_sheet::cut_list(&[PartFact]) -> Vec<CutRow>` (name, material,
thickness, W, D, qty, printed, volume cm^3), written as CSV and appended to the log as a table.

## 7. Build order

Steps 2 and 4 need no kernel and can run beside step 1.

| # | Delivers | Gate | Number to record |
|---|---|---|---|
| 1 | Shim verbs, stub symbols, `TKFillet`/`TKOffset`, `/EHa`, fixes 5-7, Rust wrappers | `cargo test -p mc-kernel` | Closed shell of 90x61x26 at 2 = **34896 mm^3**; open top = **25092**; 10 mm cube, one edge filleted r 2 = **991.416**; chamfered 2 = **980**; 60x40x20 split at z 8 = **19200 + 28800**; cylinder r 5 h 10 = **785.398**; two-vertex bulge-1 profile r 1 dist 3 = **9.42478**; box has 6 faces, 12 edges, area 8800. Shell of a box with a hole cut returns a shape or a sentence, never a crash |
| 2 | `mc-geom` frame rule; `mc-sketch` profile lowering, `set_edge_length`, `relax_lengths`; `mc-id::fnv1a` | `cargo test -p mc-geom -p mc-sketch -p mc-id` | Front frame = (x, z); slot 2 x 24 r 1 lowers to 4 vertices |
| 3 | `mc-doc`: types, `eval`, `Cmd`/`apply`, `rebuild`, `Cache`, `templates::pi4b` | `cargo test -p mc-doc -- --nocapture` | pi4b: 15 outcomes Ok, 2 bodies, Base bbox z 0..16, Lid z 14..26, no overlaps; fillet radius 40 breaks feature 2 only and 2 bodies remain; `edit_last_ms < 300` asserted; `cold_ms` printed |
| 4 | `mc-io` json, save/load, STL, DXF; `mc-log` render wired; `mc-assist` findings; `mc-sheet` cut list | `cargo test -p mc-io -p mc-log -p mc-assist -p mc-sheet` | `load(save(doc)) == doc` for pi4b; STL length = 84 + 50n; a 1.2 mm shell yields the wall finding |
| 5 | `mc-engine::Session`, worker, undo; `mc-cli build` | `cargo test -p mc-engine`; `cargo run -q -p mc-cli -- build --template pi4b --out out` | Writes `pi4b.makercad`, `base.stl`, `lid.stl`, `cutlist.csv`, `log.md`; prints total ms |
| 6 | `mc-view` ray and pick; `mc-ui` on `Session`: viewport, timeline, properties, bodies, status bar; `mc-desktop --template`, `--open` | `cargo test -p mc-view`; `cargo run -q -p mc-cli -- layers`; `target\debug\mc-desktop.exe --frames 240 --template pi4b` | `layers ok`; mean update **< 16 ms** with the case on screen |
| 7 | Tools: plane pick, polygon, slot, hole, shell, fillet, chamfer, split, explode; Sancho strip; log tab; File menu | `cargo test -p mc-ui`; the hand test: build the case in the brief's order, 12 steps, written with the `test-pass` skill | Hand test passes on the reference machine |
| 8 | Cold run of everything; `docs/STATUS.md` and `TESTING.md` rewritten (the current `TESTING.md` describes a W/D/H/hole UI that no longer exists) | `cargo test --workspace`; `cargo run -q -p mc-cli -- layers` | Test count (23 today), `cold_ms`, `edit_last_ms`, first frame, mean update |

## 8. If time runs short

Drop in this order. What remains after all nine still models, saves, checks and exports the
enclosure from the template and edits it by parameter.

1. Embossed text plate: **already out**; it needs font outlines, which is a dependency.
2. DXF of a picked face (keep DXF of a sketch).
3. Polygon edge dimensions and `relax_lengths` (keep the polygon tool).
4. Chamfer in the UI (keep the verb).
5. Explode slider (keep the eye toggles).
6. `Op::Split` by profile (keep plane split).
7. Undo and redo (keep delete and edit).
8. The worker thread: rebuild on the UI thread, with the status painted one frame before.
9. Sketching on a picked face (the six base planes with an offset are enough for the case).

Never dropped: the step 1 verbs, Extrude / Shell / Fillet / Split, the template, parameters,
save and open, STL, cut list, Sancho's findings, the log.

## 9. Extras, one line each (strike any)

- E1 `mc-cli build --template pi4b --out DIR`: the whole chain headless; it is also step 5's gate.
- E2 Overlap check between bodies with `common`, so "the lid will not fit" is measured, not guessed.
- E3 Explode slider.
- E4 Save through a temp file and rename.
- E5 Cut list carries volume in cm^3 for printed parts.
- E6 `--template` and `--open` on `mc-desktop`, so the frame-time gate runs on a real model.

Not planned, the PM's call:

- One dependency (`serde_json`) would delete about 400 lines of hand-written JSON. The brief says no.
- OCCT is still mapped at process start. True on-demand load needs `/DELAYLOAD` link
  arguments; unproven on this install, so left out.
- Inches. Everything displays millimetres tonight; `DESIGN.md` wants inches by default for the Maker persona.
- A `Move` feature (turn the lid over for printing). The transform verb is there; the feature is not.
