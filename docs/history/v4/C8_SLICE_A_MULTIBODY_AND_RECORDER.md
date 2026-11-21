---
title: "C8 Slice A — Multi-Body Display + Human Interaction Recorder"
project: MakerCAD
author: Claude (kernel lane)
verifier: Crush / Kris
status: implemented, gate-green
date: 2026-08-23
tags: [makercad, C8, C10, multi-body, recorder, handoff]
---

# C8 Slice A — Multi-Body Display + Interaction Recorder

Handoff note for Crush (game/ lane was touched with Kris's explicit authorization
on 2026-08-23 while Crush is on cooldown). Two independent changes shipped here.

## 1. Root cause of "evidence shows one shape not two"

Not a rendering glitch — an architectural limit. The display only ever showed
the **history tip**: `makercad_extension.cpp::_show_full()` did `int i = n - 1`
and rendered that single node. Two extrudes = two history nodes, but only the
last was ever drawn. Both shapes existed in the kernel; the first was invisible.
The history model is also a single linear pipeline with one `tip()`, so
"persistent bodies that interact" had nowhere to live.

## 2. Slice A — multi-body display (kernel + boundary lane, `src/`)

Concept: a **live body = any history node whose result is a real shape AND is
not consumed as an input by a later node.** Because `in[k] < i` always,
consumption only points backward, so one pass finds the "leaf" results — the
shapes that currently coexist. Backward-compatible: a pure linear chain still
yields exactly `{tip}`; two independent extrudes yield both.

Changes:
- `src/core/history_engine.h/.cpp` — new `std::vector<int32_t> live_nodes() const`.
- `src/boundary/makercad_extension.h/.cpp` — `_show_full()` / `_show_proxy()` now
  render the **whole live set** into a pooled set of `(solid, edge)` MeshInstance3D
  slots (`BodySlot`, grown by `_ensure_slots`), instead of the single tip onto
  `this`. Each slot shades with `solid_mat` / edge ribbon shader. The legacy
  `solid`/`edge` children and `this`-mesh are kept but stay empty.

Known scope limits (future slices, unchanged behavior for single-body):
- Picking / measure / focus still address a specific node (usually tip); picking
  across multiple bodies is a later slice.
- A body whose downstream feature failed is still not shown (matches old
  single-tip behavior — no regression).
- This is display-only. Operation-type (extrude Join/Cut/Intersect vs New Body,
  retiring the `_negative_extrude` shell hack) is **Slice B / C10**, not done here.

## 3. Human Interaction Recorder (game/ lane)

A foundational, general telemetry system (Kris's ask: robust enough for agents
to learn from human behavior, not a camera one-off).

- `game/scripts/interaction_recorder.gd` — NEW autoload singleton `Recorder`
  (registered in `game/project.godot` `[autoload]`). Append-only,
  schema-versioned (`v:1`) JSONL at
  `user://interaction_log/<timestamp>_<label>.jsonl`. Inert until `start()` —
  the headless smoke scene never calls it, so it records nothing there.
  Envelope: `{v, t(ms since start), type, ...}`. Event types today: `session`,
  `camera` (with `cause`: orbit/zoom/focus/reset/pivot/align/pose), `pose_mark`.
  Add more (verb/pick/sketch) via `record(type, data)` — no schema bump.
- `game/lowfi_orbit_camera.gd` — `_apply(cause)` now logs every camera change via
  `Recorder.record_camera(...)` (epsilon-deduped). Added `get_state()` /
  `apply_state()` for pose serialization.
- `game/scripts/maker_shell.gd`:
  - `_ready()` calls `Recorder.start("maker_shell")`.
  - **F8** marks the current view as a canonical pose → logs `pose_mark` and
    writes `user://camera_pose.json`.
  - **F7** re-applies the last marked pose.
  - `_evidence_shot_settled()` — like `_evidence_shot` but awaits a drawn frame
    (so a same-step camera move is reflected; the original `_evidence_shot` is
    unchanged, preserving the preview-before-commit shots' timing).
  - The C8 evidence step prefers a human-marked pose (`Recorder.load_pose()`),
    else a document-centered auto framing. This is the agent-learning loop:
    the human's chosen framing drives the shot.

## 4. Verification

- `verify.ps1`: build SUCCESS, API smoke **55/55**, visual boot OK (re-run after
  every change here, including the autoload).
- Multi-body proven: `test_evidence/c8_multi_sketch.png` now shows the large L
  body (a **non-tip** body, nodes 0-3) AND the small second extrude (node 6)
  simultaneously — previously only the tip rendered.
- Recorder self-test: auto-flow run produced a valid session JSONL with
  `session` + `camera` events (causes tagged correctly).

## 4b. Follow-on work (2026-08-23, Kris-directed, same session)

Multi-body picking, C10, camera, and a richer recorder — all gate-green (59/59).

- **Multi-body picking** (`src/boundary`, mine): `pick` / `pick_edge` now iterate
  `live_nodes()` and return the nearest hit's body, so faces/edges on ANY body
  are selectable — not just the tip. (Slice A had left picking tip-only.)
- **C10 operation type** (`src/`, mine): `EXTRUDE.f[3]` = 0 NEW_BODY / 1 JOIN /
  2 CUT / 3 INTERSECT with target body in `EXTRUDE.in[1]`; `CUT.f[0]` = boolean
  mode. New `ShapeStore::boolean(t, tool, mode)` (Fuse/Cut/Common, still in the 2
  containment TUs). Boundary: `extrude_op(...)`, `boolean_op(...)`,
  `get_live_nodes()`. Serialization already round-trips f[3]/in[1]. Tests
  **T18.1-4** prove two-body count, JOIN=1500, CUT=500, INTERSECT=500.
- **Shell C10 wiring** (`game/`): the old `_negative_extrude` sign-flip is
  retired; `_extrude_mode` (Alt+E cycles NEW BODY/JOIN/CUT/INTERSECT), target
  auto-picked by `_pick_boolean_target` (newest live solid). Toast + param title
  show the op.
- **Camera** (`game/lowfi_orbit_camera.gd`): orbit no longer clamps POSITION
  (which slid the camera and felt broken) — it clamps PITCH to keep the camera
  above ground, so orbit is smooth. Added a **Pan** capability (`set_pan_mode`,
  `_pan`): Shift+drag always pans; **P** toggles sticky pan. `_recenter_pivot`
  now frames ALL live bodies, not just the tip. `get_state`/`apply_state` for
  poses.
- **Faces & edges selectable** (`game/`): LMB selects the nearest edge (or face
  if no edge; Shift+LMB forces face) and highlights it — it no longer auto-starts
  a fillet. `E` fillets / `C` chamfers the selected edge; `F` focuses a face.
  New `_select_face` shows a centroid marker.
- **Recorder as a debug/telemetry tool** (`game/`): new event types beyond
  camera — `state` (mode, sel_node/edge/face, active verb, pan, sketch count,
  history size) logged at every transition; `verb` (name, value, result node,
  extrude_mode, live-body count) on each commit; `pick` (kind, node, index) on
  selection. So a "something weird happened" moment reconstructs to the exact
  instance. Schema envelope unchanged (`v:1`).

## 5. For Crush on return

- Nothing in `game/` was refactored — only additive hooks + one new file + the
  autoload line + the F7/F8 arms + the C8 evidence framing. Your C8 auto sequence
  logic is otherwise intact.
- Suggested next: Slice B / C10 (operation-type: New Body / Join / Cut /
  Intersect + target body, retiring `_negative_extrude`), then wire the
  `curved_lines_2d` sketch tools (Slice D) and pipe verb/pick events into the
  Recorder for a richer behavior corpus.
