---
title: "DELEGATION C8-WRAPUP — Local-Access Smoke Test + C8 v1 Verification"
project: MakerCAD
status: briefed
assignee: Claude
verifier: Crush
updated: 2026-08-19
tags: [makercad, delegation, C8, smoke-test]
---

# Delegation Brief C8-WRAPUP — Smoke Test + Verification

**Context.** Crush is entering a cooldown. Claude (kernel lane) has been granted local filesystem/shell access for the first time. C8 v1 multi-sketch support was just shipped shell-side (`game/scripts/maker_shell.gd`) and verified ad hoc, but the formal gates are not yet rerun. This brief is a graduated test of Claude's local access before any larger assignments.

**Why graduated:** run the steps in order. Stop at the first failure, report what happened verbatim (command + full stderr), and do not improvise fixes on test steps 1-3.

## Phase 0 — Access smoke test (reporting only, no edits)

1. List directory `C:\Users\<you>\MakerCAD\makercad_v4\game\scripts` and confirm `maker_shell.gd` exists.
2. Read lines 1-15 of `verify.ps1` and quote them back.
3. Check this file exists: `C:\Users\<you>\AppData\Roaming\Godot\app_userdata\MakerCAD v4\c8_multi_sketch.png`

If all three pass: continue. If any fail: stop and report.

## Phase 1 — Controlled write

4. Copy `C:\Users\<you>\AppData\Roaming\Godot\app_userdata\MakerCAD v4\c8_multi_sketch.png` into `C:\Users\<you>\MakerCAD\makercad_v4\test_evidence\c8_multi_sketch.png`.
5. Produce `c8_multi_sketch_small.jpg` next to it (downscaled, matching how the c6 shots in `test_evidence/` were downscaled; Python + PIL is fine, Python is at `C:\Users\<you>\AppData\Local\Programs\Python\Python314\python.exe`).
6. Look at an existing c6 `_small.jpg` first and match its approximate dimensions/quality so the evidence set stays consistent.

## Phase 2 — Full verification gate

7. Run: `powershell -ExecutionPolicy Bypass -File C:\Users\<you>\MakerCAD\makercad_v4\verify.ps1`
8. Confirm all three gates pass: kernel build, 55-test API smoke, visual boot.
9. If the API smoke reports anything other than 55/55, paste the failing test names verbatim. Do NOT patch code to make a gate pass without flagging it as a proposal first — verification lane reports, Crush/Kris decide.

If Phase 2 is green, Claude's local access is validated and future briefs can skip the smoke-test phase.

## Phase 3 — Standing kernel asks (background priority, only after Phase 2 green)

These are already briefed elsewhere; restating so they stay on the ladder:

- **C10** — extrude semantics: cut vs. unite as a first-class kernel concept (currently negative-extrude is a shell-side sign flip on `cad.extrude`), plus face-sketch placement.
- **C12** — mirror + pattern ops (doc 62 ruling, mirror/pattern verdict).
- **C4a** — planegcs solver DLL, still blocked on the FreeCAD git fetch; re-attempt fetch, report if still blocked.

## Hard constraints (unchanged from prior briefs)

- OCCT confined to the 2 containment TUs; floats/ints/packed arrays/Dictionaries at the boundary.
- `game/` is Crush's lane. Phase 1-2 above touch only `test_evidence/`; do not edit `game/scripts/maker_shell.gd` or `verify.ps1`.
- `verify.ps1` must remain pure ASCII (PS 5.1 reads no-BOM files as Windows-1252).
- Godot console exe for headless: `C:\Users\<you>\Godot\Godot_v4.6.2-stable_win64_console.exe`; scene must be passed explicitly as `res://scenes/maker_shell.tscn` or the default test_api scene runs instead.

## Report back

Reply in this format:

```
Phase 0: PASS/FAIL (per step)
Phase 1: PASS/FAIL (files written, sizes)
Phase 2: PASS/FAIL (gate results, test count)
Phase 3: status note per item (not started / in progress / blocked + why)
```
