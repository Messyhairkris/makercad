# MakerCAD v5 - handoff to Gordon

Written 2026-11-14 by Claude (Fable), who built passes 2 through 2c, for Gordon, who
carries the program on. The PM is Kris. Read this file first, then `docs/HANDOFF.md`
(the pass-1 handoff: decisions, reference catalogue) and `docs/INCREMENT_2.md` (what
each pass changed, how it was checked, known limits).

Everything here is either a fact checked on 2026-11-14 or marked as an opinion.

---

## 1. Where the program stands

A person can now do this by hand, and the PM has: Create Sketch, pick a plane, draw a
centre rectangle on the origin, dimension it with the D tool, add a construction line,
draw a cutout, Mirror it, Finish Sketch, Extrude. The PM recreated Fusion's wall-outlet
cover in the running app on 2026-11-14 and said it is "starting to take shape" but
"could take even more from SolidCraft and CADCraft".

Checked state on 2026-11-14:

| Check | Command | Result |
|---|---|---|
| Tests | `cargo test --workspace` | 150 passed, 0 failed |
| Layer law | `cargo run -q -p mc-cli -- layers` | `layers ok: 57 dependency edges checked` |
| Increment gate | `python gates/increment.py` | `OK: the increment is on disk and its suites pass` |
| Smoke run | `target\debug\mc-desktop.exe --template --frames 120` | exit 0 (last measured in pass 2: mean update about 13 ms, debug build) |

Tests per crate: mc-assist 8, mc-doc 20, mc-io 11, mc-kernel 25, mc-sketch 25,
mc-solver 15, mc-ui 30, mc-view 13, xtask 3.

Not verified by anyone: release-build timings; the questions screen and the
end-of-session screen under real use; anything on a machine other than the PM's.

## 2. How to build, run and look

Folder: `C:\Users\<you>\MakerCAD\makercad_v5`. It is **not a git repository**.
A copy of the sources from before pass 2 is at
`C:\Users\<you>\MakerCAD\makercad_v5_backup_2026-11-13_pre-pass2`. Opinion:
make it a git repository (or take another dated copy) before your first edit, and ask
the PM before you do either.

```
cargo run -p mc-desktop                 the app
cargo run -p mc-desktop -- --template   opens the Raspberry Pi enclosure
cargo run -p mc-desktop -- --demo       opens a sketch in progress
```

With `--demo`, the environment variable `MC_DEMO=extrude` opens the Extrude dialog and
`MC_DEMO=plane` opens the Create Sketch plane pick. These exist so a screen can be
captured and looked at without clicking.

The first start after a build can sit on a blank window for tens of seconds while the
OpenCASCADE DLLs load. That is not a hang.

Mouse: left button picks and places; right drag orbits; middle drag or Shift + right
drag pans; wheel zooms. Keys: E extrude, F fit, Esc back out, Enter accept; in a
sketch L, R, C, A, S, D (dimension), X (construction), Delete.

## 3. Rules that are not up for discussion

These come from the PM and from `docs/HANDOFF.md`. Do not relitigate them.

1. **Layer law.** `xtask/src/lib.rs` holds the table. Nothing below layer 5 depends on
   egui, eframe or wgpu. Only `mc-kernel` names OpenCASCADE, through the plain-C shim
   `crates/mc-kernel/shim/shim.cpp`. Run the layers command after any Cargo.toml change.
2. **No new dependencies** beyond serde, serde_json, eframe, egui (0.37.0).
3. **Do not touch** `.git`, `target/`, `gates/`, or `docs/ATTRIBUTION.md` except to add
   an attribution row for code actually taken.
4. **Licences.** SolidCraft and CADCraft are MIT OR Apache-2.0: code may be taken with
   an attribution row. **lamina is AGPL-3.0: concepts only, no code, no linking.**
   So far nothing has been copied from any of them.
5. **Target machine**: i5-6600, Intel HD 530, 8 GB, no working Vulkan
   (`WGPU_BACKEND=dx12,gl`, set in `mc_ui::run`). Everything is drawn with egui's 2D
   painter on purpose; there is no GPU pipeline and no depth buffer.
6. **ASCII only** in anything the log renderer touches (status text, log entries).
7. **Old files must still load.** New fields on saved types get `#[serde(default)]`.
8. **Every change to the document is one undo step**, and a refused command leaves the
   document exactly as it was and says why in the status line.
9. Sancho gives deterministic advice from rules first, and can be muted.
10. Prefer less code. If time or correctness forces a cut, cut from the bottom of a
    list, never from the top.

## 4. Map of the code

Lines are as of 2026-11-14.

| File | Lines | What it holds |
|---|---|---|
| `crates/mc-sketch/src/lib.rs` | 788 | `Sketch { points, entities, constraints }`, `Geom` (Line, Arc, Circle), `Constraint` (incl. `Symmetric`), `rectangle`, `slot`, `fillet_corner`, `mirror` |
| `crates/mc-sketch/src/profile.rs` | 492 | closed-region finding: `Seg`, `Loop`, `Profile { outer, holes, depth }`, `find` |
| `crates/mc-sketch/src/snap.rs` | 276 | snaps in CADCraft's priority order |
| `crates/mc-solver/src/lib.rs` | 898 | own Levenberg-Marquardt solver: `dof`, `solve`, `try_add`, `set_dimension`, `drag` |
| `crates/mc-kernel/src/lib.rs` + `shim/shim.cpp` | 1073 + 1065 | OpenCASCADE behind integer handles: extrude profiles, fillet, chamfer, shell, mirror, edges, faces, triangles |
| `crates/mc-doc/src/lib.rs` | 1562 | `Document` of `Feature`s, `FeatureKind`, `Plane`, `Frame`, `rebuild`, picks resolved by place |
| `crates/mc-io/src/lib.rs` | 631 | save/load, STL, DXF, true-size SVG cut template, cut list, log |
| `crates/mc-view/src/lib.rs` | 528 | `Camera`, `project`, `ray`, `unproject_plane`, `pick`, `pick_hit`, `pick_polyline` |
| `crates/mc-assist/src/lib.rs` | 342 | Sancho's rules |
| `crates/mc-ui/src/lib.rs` | 1566 | `App`, `Cmd`, undo, toolbar ribbon, Browser, timeline, `edit_kind` |
| `crates/mc-ui/src/sketcher.rs` | 1891 | sketch session, tools, Dimension box, constraints, Mirror, box select, most tests |
| `crates/mc-ui/src/tools.rs` | 306 | `Stage` (Idle, PickPlane, Extrude), Create Sketch, Extrude dialog, standard views |
| `crates/mc-ui/src/viewport.rs` | 912 | all drawing and model picking |
| `crates/mc-ui/src/screens.rs` | 495 | opening screen, new-project questions, end-of-session log |
| `crates/mc-ui/src/theme.rs`, `machine.rs` | 210, 178 | colours (Dreamy, Pastel, Workshop); performance profile (Light, Standard, Full) |

Empty shells waiting for work (7 lines each): `mc-engine`, `mc-fold`, `mc-geom`,
`mc-id`, `mc-log`, `mc-sheet`.

How a command flows: a panel pushes a `Cmd`; `App::run` runs it after the frame is
drawn; a document change goes through `App::edit` or `App::sketch_edit`, which push one
undo step and rebuild. Follow that pattern; do not mutate the document from drawing code.

How to test the UI without a window: the tests at the bottom of `sketcher.rs` build an
`App`, then either call the same functions the panels call, or feed real pointer events
through `ctx.run_ui` (see `frame` and `mouse_click` there). The second kind is what
found the lost-click bug. Use it for anything the mouse does.

## 5. Reference material

- Local clones (shallow, 2026-11-12): `C:\Tools\REAForge\work\makercad-2026-11-12\repos\`
  with `solidcraft`, `cadcraft` and others. Do not search the whole clone with a broad
  pattern; it times out. Go to the files.
- SolidCraft's own user guide, the most useful thing in the catalogue:
  `solidcraft\docs\book\src\guide\ui-tour.md`, `sketching.md`, and
  `solidcraft\docs\book\command-guides\solid.extrude.md`. The current interface follows
  these three files.
- The PM's Fusion help screenshots:
  `C:\Users\<you>\Documents\ShareX\Screenshots\2026-11\Fusion - (1..45).png`.
  Pages 1 and 13-25 are the window; 26-45 are the tutorial "Fully defining a sketch".
  You are the crew's vision member; read them yourself rather than trust a summary.
- Plasticity (`https://doc.plasticity.xyz/`): only the table of contents loaded for me.
  Nothing has been taken from it. The page under `plasticity-essentials/plasticity-interface`
  returned 404; open any URL yourself before relying on it.

## 6. The PM's newest request: display modes (do this first)

In the PM's words: the model "always shows all lines through the object", and the user
"should be able to toggle between this version where all faces and edges are visible,
wireframe, or a more traditional model that only shows what is visible (unless the mouse
hovers over and is right clicked, then it should show the options for the user to select)".

What is true today: `App::draw_bodies` in `viewport.rs` paints every triangle sorted far
to near, then draws **every** kernel edge on top, including edges on the far side. There
is no depth buffer, so nothing hides them.

Build three modes, chosen from the View menu and saved in the app (not the document):

| Mode | Faces | Edges |
|---|---|---|
| Shaded, all edges (today's look) | shaded | every edge |
| Shaded, visible edges (make this the default) | shaded | only the parts not behind a face |
| Wireframe | none | every edge |

Opinion on how, given the rules in section 3:

1. In `mc-view` (layer 5, pure, no egui) add a function that splits a polyline into its
   visible runs: for each sample point, cast the ray from the eye and call the point
   hidden when a triangle is hit nearer than the point by more than a small tolerance.
   `pick_hit` already does the ray-triangle work.
2. Cache the result in the app. Recompute only when the camera or the built model
   changes; never per frame while nothing moves. While the camera is moving on the Light
   profile, falling back to all edges and refining on stop is acceptable.
3. Right-click over the model in the visible-edges mode opens a **Select Other** list of
   everything under the pointer along the ray, nearest first, hidden faces and edges
   included; hovering a row lights the thing up, clicking picks it. This is SolidCraft's
   "Select Other" (`ui-tour.md`, the Viewport paragraph). The existing right-click menu
   is `response.context_menu` at the end of `App::viewport`; put the list in it.

Checks to write before the code (the functions are pure, so these are ordinary tests):

| Case | Expected |
|---|---|
| A 60 x 40 x 20 box seen from the default 3/4 view | 9 edges wholly visible, 3 wholly hidden |
| The same box seen straight from the top | the 4 top edges visible; the 4 bottom edges hidden (they lie exactly behind the top ones: decide and state which way the tolerance falls) |
| An edge half behind a second body | split into one visible run and one hidden run at the crossing, within 1 mm |
| Wireframe | no triangles painted; edge count equals the kernel's |
| Pointer over the box's front face, Select Other | the list holds the front face and the back face behind it, in that order |
| Mode changed, then the app restarted | not required to persist yet; say so in the known limits |

Timing to report, not assume: milliseconds to compute visible runs for the Raspberry Pi
template (`--template`) on the Light profile.

## 7. The rest of the backlog, in the order I would take it

Each line names the gap, where the reference is, and a check that would catch a wrong
answer. Write the check first.

**Sketching (SolidCraft `sketching.md`, CADCraft `snap/` and command line)**

1. *Value boxes while drawing.* After the first click of Line, Rectangle or Circle, boxes
   for length/angle, width/height, diameter follow the pointer; typing fills the first,
   Tab moves on, Enter places; each typed value becomes a dimension. Today sizes are
   added afterwards with D. Check: rectangle, type 60, Tab, 40, Enter gives a 60 x 40
   rectangle with two dimensions and one undo step. The Dimension tool's `DimBox` in
   `sketcher.rs` is the pattern to grow from.
2. *Trim (T) and Offset (O).* Check for trim: a circle drawn across a rectangle's top
   edge, two clicks, leaves a notch and exactly one profile. Check for offset: a 60 x 40
   rectangle offset 4 inward gives a 52 x 32 rectangle held by one dimension.
3. *Profiles remembered by their curves, not by index.* `FeatureKind::Sketch.regions`
   stores indices into `sketch.profiles()`. Editing the sketch can renumber them, so an
   extrusion can silently take a different region. SolidCraft's rule is in
   `sketching.md` under "Profiles". Check: extrude region A, add an unrelated circle to
   the sketch, rebuild, volume unchanged.
4. *Mirror about the sketch axes*, a live preview before the line is clicked, and a mark
   drawn for symmetry. Today the mirror line must be a drawn line.
5. *Dimension placement.* Dimension lines sit at a fixed 22 pixel offset
   (`draw_constraints`). Let the user drag the text, and let its position choose
   aligned / horizontal / vertical as in `sketching.md`.
6. *Distance between two lines* works only when both are level or both upright
   (`offset_dimension`). A general point-to-line distance constraint in `mc-solver`
   removes the limit.
7. *CADCraft's command line*: type `L`, click, type `@5<45`, Enter. Direct distance
   entry is the part makers will feel.

**Modelling (SolidCraft command guides)**

8. *Box, Cylinder, Hole placed by pointing*: click a plane or face, drag or type, with
   a dialog and live preview like Extrude. Today they appear at the origin with fixed
   numbers. Fusion's walkthrough (screenshots 21-23) shows the flow.
9. *Fillet, Chamfer, Shell as dialogs with a live preview*, selection inside the dialog,
   OK / Cancel, one undo step. `Stage::Extrude` in `tools.rs` is the pattern.
10. *A drag arrow on the extrusion* and symmetric / two-sided extents.
11. *Sketch on a slanted face.* `Plane::from_face` accepts only faces that look along
    X, Y or Z, and Back, Left and Bottom open rotated half a turn. A general frame
    (origin, u, v) stored on the feature fixes both; keep the six named planes loading.
12. *Picks that survive edits.* Fillet and shell remember an edge or face by a place in
    space within `PICK_TOLERANCE` (1 mm). A change upstream that moves the place loses
    the pick. This is the topological naming problem; SolidCraft is the reference.

**Window (SolidCraft `ui-tour.md`, Fusion screenshots 1, 13-17)**

13. *A real ViewCube* in place of the four view buttons, clickable faces, edges, corners.
14. *Command search (S).* Type a few letters, Enter runs the command. Every action is
    already a `Cmd`, so this is a table of names over what exists.
15. *Timeline*: right-click a step for Edit / Delete / Suppress; a marker that rolls the
    model back (`Document::rebuild_until` already exists); play and step.
16. *Browser*: eye icons to show and hide bodies and sketches; right-click menus.
17. *Workspace switcher* where Fusion has DESIGN. The PM's vision is environments as
    modules loaded per project. `machine.rs` lists them: solid and sketch are ready;
    fold, motion, scan, print, cnc, circuit are not built. `brief.environments` already
    records which a project uses.

**The maker's side, which is the reason the program exists** (from the PM's vision;
none of it is in the reference programs, so it is where MakerCAD is its own thing)

18. Material budget drives the design: stock listed in the questions (for example
    fifteen pine 2x4s, three feet long) checked against the parts, waste shown.
19. Board-level cut layout, blueprints, and an order of assembly, beside the existing
    cut list and true-size cut template.
20. A mood board that holds images. Today inspiration is text.
21. The end-of-session log published as a build guide; later, build photos and video.
22. Scan data and point clouds for "where must it fit"; later, scan-compare.

**Housekeeping**

23. `docs/ATTRIBUTION.md` and `docs/DECISIONS.md` still name planegcs as the solver.
    The solver is our own. Left for the PM because of rule 3; raise it with him.
24. Round holes look eight-sided on the Light profile (`Profile::deflection` is 0.5 mm).
    Opinion: scale the deflection to the part's size instead of a fixed number.
25. The Draw row in the right panel duplicates the ribbon.

## 8. Traps I fell into, so you need not

- **egui 0.37 is not the egui in most examples.** `eframe::App::ui(&mut self, ui, frame)`;
  `egui::Panel::top("id").show(ui, ..)`; `ctx.run_ui(input, |ui| ..)` for headless
  frames; `ctx.egui_wants_keyboard_input()`; `ui.scope_id()`; `ui.close()`;
  `egui::CornerRadius::same(u8)`. Check a name in the registry source before using it:
  `%USERPROFILE%\.cargo\registry\src\*\egui-0.37.0\src`.
- **egui stops calling it a click after 6 pixels of movement.** That silently ate clicks.
  Use `App::clicked_loosely` for anything picked in the viewport, and test with a slip
  (`mouse_click(.., 9.0)`).
- **The left button must never move the camera.** It did, and a slipped click tilted
  the view.
- **A headless frame must clear its textures**: `out.textures_delta.clear()`, or the
  test panics on drop.
- **`apps/mc-desktop/src/main.rs` has CRLF line endings**; the other sources are LF.
  A text replace that assumes LF will miss.
- **Use-after-realloc in the shim.** `mc_shape_split` held a pointer into the shape
  table across a call that could grow it. Copy the handle first. Suspect the same in
  any new shim function that creates a shape while holding an entry.
- **A tangent at a shared endpoint is a double root** and constrains nothing to first
  order; the solver states it as "radius square to the line" there. Any new constraint
  whose residual has zero slope at the answer will under-count in the same way.
- **`HDistance` and `VDistance` residuals use an absolute value**, so a value of 0 (used
  for "these two points are lined up") works, but the sign of the offset is not held.
- **A test that expects the solver to slide a shape may see it stretch instead.** Both
  satisfy the constraints. Assert what the constraints guarantee.
- **Do not judge a screen from memory of the code.** Capture it and look. The capture
  script used so far lived in a temporary folder and is gone; it called `PrintWindow`
  on the app's window from PowerShell. Sancho filling the whole background orange was
  only found by looking.
- **The brief's slot volume (1963.5) was wrong**; the analytic value is used and noted
  in `INCREMENT_2.md`. Check a number in a brief before building a test on it.

## 9. How to report

After each piece of work, add a dated section to `docs/INCREMENT_2.md` (or start
`INCREMENT_3.md`) with three headings: **What changed**, **How it was checked**,
**Known limits**. Say what was not verified in plain words. The PM tries the app by
hand after each pass; give him a short path to click through, and ask him to quote the
status line beside "Sancho" when something is refused.

A note on working with the crew harness: its code tasks run Python and JavaScript
through hidden tests, not Rust, so this work happens in the repository itself with
`cargo test` as the check. The crew's one rule still holds: write the check before the
code, and do not call anything done that the check has not passed.
