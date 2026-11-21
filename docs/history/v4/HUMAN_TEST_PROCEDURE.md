# MakerCAD v4 — Human Test Procedure

**Version under test:** `makercad_v4/` (Claude's delivery, 15 files + CMakeLists + gdextension)
**Reference machine:** Intel i5-6600 / 8 GB RAM / Intel HD 530 (the spec)
**Rule (from the review doc): a result only counts if it was cold-run on THIS machine and recorded below.**

Fill in the **Result** column as you go: `PASS` / `FAIL` / `BLOCKED` + one line of notes.
If anything is BLOCKED, write down the exact error text — it goes into the debug loop.

Estimated total time: 30–45 min (build excluded; the build itself may take 20–60 min the first time because godot-cpp compiles from source on 4 cores).

---

## T0 — Prerequisites (5 min, one time)

| # | Check | How | Expected | Result |
|---|-------|-----|----------|--------|
| T0.1 | CMake | open any terminal, run `cmake --version` | prints 3.30.x line | |
| T0.2 | Ninja | file exists: `C:\dev\tools\ninja.exe` | exists (build.ps1 puts it on PATH for you) | |
| T0.3 | MSVC | file exists: `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat` | exists | |
| T0.4 | OCCT 7.9.3 | folder exists: `C:\OpenCASCADE\opencascade-7.9.3-vc14-64\cmake` | exists | |
| T0.5 | godot-cpp | folder exists AND is linked: `C:\Users\<you>\MakerCAD\makercad_v4\extern\godot-cpp\CMakeLists.txt` | exists (junction to `C:\dev\godot-cpp`, pinned `godot-4.5-stable`) | |
| T0.6 | Godot | file exists: `C:\Users\<you>\Godot\Godot_v4.6.2-stable_win64.exe` | exists | |

---

## T1 — Build the extension (scripted, 20–60 min first time)

**Steps:**
1. Open PowerShell.
2. Run: `powershell -ExecutionPolicy Bypass -File C:\Users\<you>\MakerCAD\makercad_v4\build.ps1`
3. Wait. Watch the output scroll. (First run compiles godot-cpp from source — this is most of the time.)

**Expected:**
- The script ends with `=== build.ps1 finished: SUCCESS ===`
- File exists: `game\bin\makercad.windows.release.dll`
- `game\bin\build_result.txt`... actually `build\build_result.txt` starts with `SUCCESS`
- `game\bin\` contains many OCCT DLLs (TK*.dll etc.)

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T1.1 | Script completes | `build\build_result.txt` contains `SUCCESS` | |
| T1.2 | Extension DLL built | `game\bin\makercad.windows.release.dll` exists, size > 500 KB | |
| T1.3 | Runtime DLLs present | `game\bin` holds 30+ `.dll` files (OCCT + 3rd-party) | |

**If it FAILs:** copy the last ~20 lines of the PowerShell window plus `build\build_result.txt` into the debug session. Common causes: OCCT path mismatch (T0.4), missing ninja (T0.2), RAM exhaustion during godot-cpp compile — close browsers/other apps and rerun; the build is incremental.

---

## T2 — Extension loads in Godot (2 min)

**Steps:**
1. Open Godot: `C:\Users\<you>\Godot\Godot_v4.6.2-stable_win64.exe`
2. Import → browse to `C:\Users\<you>\MakerCAD\makercad_v4\game\project.godot` → Open
   *(build.ps1 already ran `--import`, which is required once so Godot discovers the extension — if you ever skip build.ps1, run the project in the editor once before judging failures.)*
3. Watch the **Output** panel at the bottom

**Expected:**
- Project opens without a red error banner
- Output shows **no** lines containing `Can't open dynamic library` / `126` / `MakerCADExtension` errors
- Bottom-right corner of the editor says **Compatibility** (renderer), not Forward+ or Mobile

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T2.1 | Project opens | no error banner | |
| T2.2 | GDExtension registered | no "126"/"failed to load" in Output | |
| T2.3 | Renderer | editor shows "Compatibility" | |

**If T2.2 fails with Error 126:** a runtime DLL is missing from `game\bin`. Re-run step "Copying OCCT runtime DLLs" by just re-running `build.ps1` (it is idempotent).
**If the class is missing with NO error at all:** the editor import never ran — see the note above. (Root cause: Godot only discovers extensions during editor import, recorded in `.godot\extension_list.cfg`.)
**If the editor can't load it but a direct run can:** a `.gdextension` debug key points to a DLL that doesn't exist — the editor uses the debug feature tag. `game\makercad.gdextension` currently maps both keys to the release DLL on purpose.

---

## T3 — Automated API smoke test (2 min, no visual judgment needed)

> **STATUS on reference machine (2026-08-11, Claude/Crush run): 13/13 PASS** — this section record belongs in the results table; re-run to confirm before/after any code change.
>
> **Visual verification adds (2026-08-11, auto-captured):** T4 box + edges PASS; T5 cut renders solid (CULL_DISABLED fix — earlier invisible/shakey top/bottom faces resolved); T6 fillet renders solid; T7 orange ghost proxy PASS — also during a full drag orbit (opaque-grid fix killed the transparency-sorting flicker Kris reported at 10:55); T8 undo PASS; FPS 57–60 throughout; ESC quit PASS. PNG evidence in `test_evidence/`. Remaining subjective check for the human: judge edge-ribbon definition (2.5 px) at native resolution.

**Steps (choose one):**
- **Headless (preferred):** in PowerShell run
  `& "C:\Users\<you>\Godot\Godot_v4.6.2-stable_win64_console.exe" --headless --path "C:\Users\<you>\MakerCAD\makercad_v4\game" "res://scenes/test_api.tscn" --quit-after 10`
  *(the scene path is required: the project main scene is the visual test now)*
- **In-editor:** open `scenes\test_api.tscn` and press **F6**, read the Output panel

**Expected:** lines like `[PASS] T3.x: ...` and a final summary `API SMOKE TEST: 13 passed, 0 failed`

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T3.1 | create_box | node 0 returned | |
| T3.2 | history size | 1 after one op | |
| T3.3 | box routes BOX | route == 1 | |
| T3.4 | boolean_cut | node 2 returned | |
| T3.5 | cut routes MESH | route == 0 | |
| T3.6 | fillet_all_edges | node 3 returned | |
| T3.7 | edge segments | non-empty, multiple of 6 | |
| T3.8 | undo_to(3) | history back to 3 | |
| T3.9 | extrude on solid | -1 (documented scope choice) | |
| T3.10 | cut w/ invalid node | -1, no crash | |
| T3.11 | route_info invalid | empty dict, no crash | |
| T3.12 | set_detail | runs without crash | |

A `FAIL` line is self-diagnosing — report its T-number and text.

---

## T4–T9 — Human visual tests (10–15 min)

**Steps:** press **F5** (play) — the project's main scene IS the visual test (`scenes\visual_check.tscn`).
*(If you pressed F5 before and got a blank grey window, that was the logic-only API test scene running as main; fixed — F5 now opens this scene.)*
Press **SPACE** to advance through the steps. On-screen text repeats each check — useful checks below.

| # | Test | Exactly what to look for | Result |
|---|------|--------------------------|--------|
| T4 | Parametric box | One solid 4×2×3 box. All faces shaded (no black/invisible sides), dark outline on every edge. Hold MMB, orbit around it, check all sides. | |
| T5 | Boolean cut | A notch/hole cut through the box, still solid, crisp edge lines around the cut. No flickering surfaces (z-fighting), no missing faces inside the cut. | |
| T6 | Fillet all edges | Sharp edges become visibly rounded. Surface stays closed — no holes, spikes, or exploded triangles. | |
| T7 | Low-fi proxy | While holding MMB/RMB and dragging, the detailed shape is replaced by a bright ORANGE translucent ghost box; releasing the button instantly brings the detail back (with fillets/cuts intact). The ghost is SUPPOSED to look box-like — it is a bounding-box stand-in, not a regression. Try fast circular drags. | |
| T8 | Undo | After SPACE, fillet disappears (back to sharp-edged cut shape), hole still present. Display updates by itself — no stale geometry mixed in. | |
| T9 | Performance | Read the FPS counter (top-right) while orbiting and zooming hard for ≥10 s. PASS = steady 30+ FPS, no second-long stalls. Note the lowest number you see. | |

**Camera behavior (2026-08-11 update):** orbit pivots on the main mass center (AABB center); the camera cannot dive below the ground grid (hard floor at y=0.2); a ground grid with minor/major lines is always visible. ESC quits the test window. On-screen step text is now wrapped/short — all steps readable. **If a visual test fails:** note WHICH step and WHAT you saw. Exact symptom descriptions are what make rendering bugs fixable — this follows the staged pipeline from the Phase 2 expert review.

---

## T10 — Exit + cleanup (1 min)

| # | Step | Expected | Result |
|---|------|----------|--------|
| T10.1 | Close the game window | no freeze, no "application stopped responding" | |
| T10.2 | Close the Godot editor | no crash dialog | |
| T10.3 | Record summary below | — | |

### Summary (fill in)

- Date/tester: ______________
- Build result (T1): **SUCCESS on reference machine 2026-08-11 00:21 (first build, ~13 min)**
- API smoke: **13/13 passed (2026-08-11, automated run)**, re-confirm after changes
- Visual: **T4 ✓ T5 ✓ T6 ✓ T7 ✓ T8 ✓ confirmed via auto-capture 2026-08-11** — human spot-check edges at native res; min FPS seen: 57
- Worst failure found: winding/culling on tessellated top/bottom faces (T5) — FIXED via CULL_DISABLED solids + edge depth nudge (locked readability decision in the design doc)
- Next action per procedure: failures go back into the debug loop with their T-numbers; a full PASS run unblocks Part 6 Phase B of the v4 roadmap.

---

## Appendix A — What each API maps to (quick translator)

1 real OCCT pipeline runs under this UI: `create_box → boolean_cut → fillet_all_edges → undo_to`.
`get_route_info` returns the ShapeAnalyzer verdict: 1 = BOX (Godot primitive, no tessellation), 2 = CYLINDER, 0 = MESH (tessellated + edge ribbons). `set_interaction_state` is the drag proxy (driven automatically by the camera script).

## Appendix C1 — Sketch node tests (headless, no human check needed)

Run with the same T3 headless command; these print alongside the T3 lines.

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T11.1 | create_sketch(0) + 4-pt rect + close | node 0, close true | |
| T11.2 | close with <3 points | false (validation) | |
| T11.3 | extrude sketch (XZ plane, sweep +Y 10) | node ≥ 0, route MESH (0), edge segments non-empty | |
| T11.4 | triangle extrude + readback + undo | history_size 6, exact point round-trip, undo_to(0) → 0 | |
| T11.5 | NaN coords | add false, close false, no crash | |
| T11.6 | in-plane sweep (degenerate) | node pushed, NO_SHAPE result (empty route_info) | |

## Appendix C2 — Picking API tests (headless, no human check needed)

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T12.1 | pick at box face center | face ≥ 0, correct node | |
| T12.2 | pick at empty screen corner | `{}` | |
| T12.3 | pick_edge nearest edge midpoint → per-edge fillet | exact edge index; single-edge fillet segment count ≠ ALL_EDGES count | |
| T12.4 | picks are read-only | history_size unchanged | |
| T12.5 | pick after undo_to | node == restored tip | |

## Appendix C3 — Edit / redo / persistence tests (headless)

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T13.1 | `set_node_params` on a box with a fillet downstream | box resizes 4→8, downstream node still builds | |
| T13.2 | over-large fillet radius | `has_shape` false + non-empty `broken_reason` | |
| T13.3 | `undo_to` then `redo` | node restored, redo_depth decrements | |
| T13.4 | new op after undo | redo stack cleared | |
| T13.5 | `serialize`/`deserialize` | history size, sketch points, edge segments all match | |
| T13.6 | `save_project`/`load_project` via `user://` | same round-trip through a real file | |
| T13.7 | bad JSON / wrong format / missing file | all rejected, existing model untouched | |

## Appendix C5/C6 — Export + chamfer tests (headless)

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T14.1 | `export_stl` binary | file > 84 bytes (header + facets) | |
| T14.2 | `export_stl` ascii | starts `solid`, contains `facet` | |
| T14.3 | `export_step` ap214 | `ISO-10303-21` header naming 214 | |
| T14.4 | `export_step` ap214=false | AP203, no stale 214 schema | |
| T14.5 | bad node / empty path / shapeless node | all refuse, no crash | |
| T15.1 | `chamfer_all_edges` | builds a shape | |
| T15.2 | per-edge vs all-edges chamfer | different segment counts | |
| T15.3 | picked edge index drives chamfer | same intent indices as fillet | |
| T15.4 | over-large chamfer | `has_shape` false + reason | |
| T15.5 | chamfer node save/load | round-trips intact | |

**Slicer check (human, one-time):** open `user://t14_export.stl` in a slicer
(or Windows 3D Viewer) — it should load as a closed 20×10×5 mm filleted box.
That is the bracket → slicer path end to end.

## Appendix C4b — Point identity + constraint storage (headless)

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T16.1 | ids across insert + delete | every surviving id unchanged, new id unused | |
| T16.2 | `add_constraint` | stored, `solved:false`, serialized state changes | |
| T16.3 | constraint on unknown point id | refused | |
| T16.4 | delete a constrained point | only its constraints drop | |
| T16.5 | v2 round-trip | ids and constraints preserved exactly | |
| T16.6 | v1 file (no `pt_ids`) | loads, ids assigned 0..n-1, geometry builds | |
| T16.7 | malformed v2 (count / dupe / dangling ref) | all refused, model intact | |

## Appendix C7 — Measure + camera framing (headless)

Assertions are against a 20×10×5 box, whose exact properties are known
(volume 1000 mm³, surface area 700 mm², 12 edges totalling 140 mm).

| # | Check | Expected | Result |
|---|-------|----------|--------|
| T17.1 | `measure_solid` | volume 1000, area 700, size [20,10,5] | |
| T17.2 | `measure_edge` over all edges | 12 straight edges, total 140 mm | |
| T17.3 | `measure_face` over all faces | 6 planar, areas sum 700, unit normals | |
| T17.4 | `measure_angle` face pairs | 90° adjacent, 180° opposite | |
| T17.5 | `measure_distance` opposite faces | equals a box dimension | |
| T17.6 | `get_focus_target` | shape radius 11.456, face radius smaller | |
| T17.7 | bad index / node / mixed kinds / curved face | refuse; curved reports `planar:false` | |

## Appendix B — Rebuilding after code changes

Re-run the same `build.ps1`; it is incremental. Then repeat T2→T9. Only re-do T0 if the machine changed.
