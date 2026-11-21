---
title: "Handoff to Crush — 2026-08-23 session (Claude, kernel lane)"
project: MakerCAD
status: gate-green (build + 63/63 API smoke + visual boot)
tags: [makercad, handoff, C8, C10, C4a, recorder]
---

# Handoff to Crush — 2026-08-23

Claude held the ladder while Crush was on cooldown. Kris authorized editing
`game/` (Crush's lane) for this session; all such edits are additive and marked
with `Claude (C8-wrapup ...)` comments. Everything below is verified: `verify.ps1`
is green — build + **API smoke 63/63** + visual boot. (Session 2 on 2026-08-24
added the addendum below; read it — it supersedes parts of items 5 and 6.)

## What shipped (in order)

1. **C8 wrap-up** — evidence smoke test re-run; `test_evidence/c8_multi_sketch*`
   refreshed. (Original brief: `DELEGATION_C8_WRAPUP.md`.)

2. **Slice A — multi-body display** (`src/`): the display showed only the history
   tip, so two extrudes rendered as one shape. Fixed with
   `HistoryEngine::live_nodes()` (leaf = result not consumed by a later node) +
   a boundary `BodySlot` pool that renders every live body. Detail:
   `docs/C8_SLICE_A_MULTIBODY_AND_RECORDER.md` §2.

3. **Multi-body picking** (`src/`): `pick`/`pick_edge` iterate live bodies and
   return the nearest — faces/edges on ANY body are selectable now, not just tip.

4. **Slice B / C10 — operation type** (`src/`): extrude is NEW_BODY / JOIN / CUT /
   INTERSECT (`EXTRUDE.f[3]`) against a target body (`EXTRUDE.in[1]`); `CUT.f[0]`
   is the boolean mode. `ShapeStore::boolean(t,tool,mode)` (Fuse/Cut/Common, in
   the containment TUs). Boundary: `extrude_op`, `boolean_op`, `get_live_nodes`.
   **Tests T18.1–4** (JOIN=1500, CUT=500, INTERSECT=500, two-body count). The
   shell's `_negative_extrude` sign-flip is retired for a real `_extrude_mode`
   (Alt+E cycles).

5. **Camera** (`game/lowfi_orbit_camera.gd`): orbit fixed (pitch-limited ground
   barrier instead of the position clamp that made it feel stuck); **Pan** added
   (**SUPERSEDED by the session-2 addendum**: the `P` sticky toggle is gone —
   pan is Shift+LMB only); `_recenter_pivot` frames all
   bodies; `get_state`/`apply_state` for poses.

6. **Faces & edges selectable** (`game/`): LMB selects+highlights the nearest
   edge (or face) — no more auto-fillet. (**SUPERSEDED**: Shift+LMB is now pan,
   not force-face; plain LMB already falls back to a face pick.) E fillets / C
   chamfers the selected edge; F focuses a face. `_select_face` added.

7. **Human Interaction Recorder** (`game/scripts/interaction_recorder.gd`,
   autoload `Recorder`): schema-versioned append-only JSONL telemetry so agents
   can learn from human behavior AND so a weird moment reconstructs exactly.
   Event types: `session`, `camera` (cause-tagged), `state` (mode/selection/
   verb/pan/history), `verb` (name/value/node/op/live-body-count), `pose_mark`,
   `pick`. F8 marks a canonical camera pose; F7 re-applies it; evidence shots
   prefer a marked pose. Kris's stated direction: keep extending this (replay,
   more event types) — it's foundational, not a camera one-off.

8. **C4a — planegcs vendored** (`vendor/planegcs/`, Kris approved): the FreeCAD
   fetch (previously blocked) is unblocked and the source is vendored (commit
   `faf5fea`, 2026-08-22). See **Next up** — it is NOT yet built.

## Verify / run

```bash
powershell -ExecutionPolicy Bypass -File verify.ps1
```
Interactive shell (for the mouse-driven bits — orbit, pan, picking):
```bash
run_shell.bat
```
Headless auto-flow (regenerates evidence, exercises C8/C10):
```
Godot_..._console.exe --path game res://scenes/maker_shell.tscn -- --auto-steps
```
(Scene MUST be passed explicitly or the default test_api scene runs instead.)

## Session 2 addendum (2026-08-24) — after Kris's human trial

Gate is now **63/63** (was 59) + visual boot.

### Fixed: orbit/pan regression (from the human trial)
My first pan implementation was **modal** — a sticky `P` toggle plus a Shift
check on the orbit buttons. One stray `P` turned every drag into a pan, which is
what Kris hit ("it now pans instead of orbits"). Pan is now **modeless**, per
Kris's spec:
- **MMB/RMB drag → always orbits** (never pans, no mode can change that)
- **Shift+LMB drag → pans** (`_pan_dragging`, set only by that chord)
- The `P` toggle and `set_pan_mode()` are **gone**; `is_pan_mode()` now just
  reports whether a Shift+LMB pan drag is in progress (recorder introspection).
The shell already returns early on Shift+LMB in ORBIT so the event reaches the
camera.

### Doc 62 found — and it changes the C12 plan
Located in the vault (not the repo):
`wiki/projects/makercad/62 - Fusion 360 Sketching Gap Analysis.md` (status:
decided, 2026-08-18).

**C12 mirror/pattern is DECLINED by doc 62, not merely unblocked.** Its
"Deliberately NOT copied" list reads: *splines, curvature-continuity
constraints, sketch patterns/mirror — overkill for low-spec bracket-class
parts.* I did **not** build C12. If mirror/pattern is still wanted it needs a
fresh decision from Kris/Crush that supersedes doc 62 (note the doc scopes this
to **sketch** mirror/pattern; body-level mirror was never ruled on either way).

### Done: C10's remaining half — sketch planes / face-sketching
Doc 62's exact kernel ask was *"create_sketch accepting plane descriptor
{origin, x_axis, z_axis} for face-sketching"* — which is also Kris's trial
request (sketches stuck on one plane). Now implemented:
- **Kernel**: `SKETCH.f[0] == 3` selects a custom plane held in
  `HistoryNode.plane[9]` = {origin, x_axis, z_axis}; `execute()` routes to
  `ShapeStore::make_sketch_wire_plane`. Plane is part of the node hash.
- **Boundary**: `create_sketch_on_plane(PackedFloat32Array plane)` (9 floats,
  validates finite/non-degenerate/non-parallel) and `create_sketch_on_face(node,
  face)` (captures the frame from a planar face; curved faces refused).
- **Persistence**: the plane descriptor is now serialized/deserialized.
  **`SCHEMA_VERSION` bumped 2 → 3** — without it a face sketch silently reloaded
  onto the XY plane. v1/v2 files still load (T16.6 covers it). **Flagged:**
  T16.5 asserted the literal `"version": 2`, so I updated that one assertion to
  3; its actual subject (ids + constraints round-trip) is unchanged.
- **Shell**: with a planar face selected, **K** now sketches *on that face* —
  the ortho sketch camera aims down the face normal, and screen↔plane mapping
  uses the frame (`_sketch_frame`). The shell passes the kernel the exact frame
  it previewed with, so drawn == built. The ground-plane path is untouched
  (zero regression risk); auto-flow still completes end-to-end.
- **Tests T19.1–4**: sketch-on-face extrudes to an exact volume (48), lands on
  the face frame (not the ground), round-trips through save/load, and refuses
  bad descriptors/curved faces.

### Still open (deliberately not started)
- **Sketch environment quality** (Kris: "improve the overall quality of sketch
  environment and tools soon"). Doc 62 sequences this as **shell lane**: C8
  session rework, **C9 rectangle/circle/arc entities**, C11 constraint UX. That
  is Crush's lane and the natural next block.
- **Construction planes** (offset/work planes) — the kernel now accepts any
  plane descriptor, so this is a shell-side plane-definition UX, no kernel ask.
- **planegcs** — still vendored-but-not-built (unchanged from below).

## Next up (the ladder)

- **C4a — build makercad_solver.dll.** Source is vendored; PIMPL wall API is
  `src/core/sketch_solver.h` (planegcs-free). planegcs is NOT standalone — it
  needs **Eigen** (`vendor/eigen/`), **Boost** (Graph + Math, header-only), and
  **FreeCAD shims** (`FCConfig.h`, `Base/Console.h`, `boost_graph_adjacency_list.hpp`).
  Full plan + dependency surface: `vendor/planegcs/NOTICE.md`. Build it as its
  OWN DLL so a failure can't break the 63-test gate. Then wire the stored C4b
  constraints → GCS, solve, write positions back, report DoF for blue/black
  constrained state. **License TODO before shipping:** add full LGPL-2.1 +
  BSL-1.0 + MPL-2.0 texts and surface in About/licenses (DEPENDENCIES.md).
- **C10 finish** — face-sketch placement is **DONE** (session-2 addendum). Still
  open: a shell UX for choosing the boolean target body (currently auto-picked as
  the newest live solid by `_pick_boolean_target`).
- **C9 sketch entities** (doc 62, shell lane) — rectangle + circle as polyline
  generators (no schema change); true arcs would be a later kernel ask. This is
  the highest-value next block for "sketch environment quality".
- **C12 — mirror + pattern.** **DECLINED by doc 62** ("Deliberately NOT copied:
  ... sketch patterns/mirror"). Do not build without a fresh ruling from
  Kris/Crush. `ShapeStore::boolean` (fuse) is in place if that ruling changes.
- **Selection polish** — full-face highlight shading (currently a centroid marker).

## Lane / hygiene notes for Crush
- `src/` changes are Claude's kernel lane; `game/` was edited with Kris's explicit
  one-session authorization — review those diffs. Nothing was refactored; the C8
  auto-flow logic is intact apart from additive hooks.
- `verify.ps1` stays pure ASCII (PS 5.1 / Windows-1252). OCCT stays in the 2
  containment TUs; the solver will get its own containment TU behind
  `sketch_solver.h`.
- Memory (Claude's): `makercad-build-plan`, `interaction-recorder-direction`.
