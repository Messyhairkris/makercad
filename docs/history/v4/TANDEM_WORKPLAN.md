---
title: "MakerCAD — Tandem Work Plan (Crush + Claude)"
project: MakerCAD
status: active
updated: 2026-08-11
tags: [makercad, tandem, coordination]
---

# Tandem Work Plan — Crush & Claude (non-colliding assignments)

**Coordination protocol (vault as message board):**
- Task briefs live in `wiki/projects/makercad/` (numbered docs). Each agent posts completion + blockers as a dated note in `wiki/projects/makercad/messages/` (file per day: `YYYY-MM-DD - <agent>.md`).
- Claude's file-level context: read `C:\Users\<you>\MakerCAD\makercad_v4\ARCHITECTURE.md` + `DELEGATION_SKETCH_NODE.md` (repo copy).
- Gate: Crush reviews every Claude delivery (build + smoke test + screenshot verification) before it counts as done. No claim without a cold-run result on the reference machine.
- Ownership rule: **never edit files outside your lane** (below); hand off through the messages folder.

## Lanes (file ownership)

| Owner | Lane |
|---|---|
| **Claude** | `src/core/*`, `src/boundary/*` (C++/OCCT kernel additions) |
| **Crush** | `game/*` (scenes, GDScript harness, UX stubs), `build.ps1`, docs, procedures, research, verification |
| **Kris** | product decisions, final acceptance, message arbitration |

## Claude queue (kernel, sequential)

| # | Task | Brief | Exit |
|---|---|---|---|
| C1 | **Sketch node** (workplane profile → extrude) | [[20 - Delegation Brief - Sketch Node (for Claude)]] | new smoke tests pass, Crush-verified |
| C2 | **Picking API**: face/edge pick through boundary (`IntCurvesFace_Intersector` port from old extension) feeding `SelectionIntent::INDEXED` | to be drafted after C1 | click → correct index, per-edge fillet works |
| C3 | **File I/O**: STEP/STL export ops + `.makercad` history serialization (nlohmann, already vendored) | after C2 | round-trip open/edit/save |
| C4 | **planegcs spike** (design only): FreeCAD sketch solver embed feasibility notes | after C3 | written spike report, no code commitment |

## Crush queue (parallel, non-colliding)

| # | Task | Output | Status |
|---|---|---|---|
| R1 | Sketch interaction research brief: Fusion 360 sketch workflow (top-down snap, click-click dimensions, Tab nav) + ThreeJS_CAD/LoCAD patterns, distilled into Phase C interaction spec | `wiki/projects/makercad/30 - Phase C Interaction Spec.md` | queued |
| R2 | Sketch-mode test stub: 2D-snap camera behavior prototype in `game/` (orthographic top-down on plane pick, 2D snapping grid) | verified test scene | queued |
| R3 | **Verify C1 on arrival**: rebuild, extend smoke test, auto-capture screenshot evidence, update HUMAN_TEST_PROCEDURE | verification note | waiting on C1 |
| R4 | Packaging/dieline data-model distillation from FoldCAD docs (`PACKAGING_ENGINEERING.md`) into Phase D spec | `wiki/projects/makercad/40 - Packaging Data Model.md` | queued |
| R5 | Regression harness: scripted build→smoke→capture pipeline so each Claude delivery gets verified in one command | `verify.ps1` | queued |
| R6 | Selection/UX decision capture: keep design doc + DECISIONS log current from Kris's feedback | updated docs | continuous |

## Dependencies & ordering

- C1 → R3 → C2 (picking needs sketch extrusion solid to click).
- R1/r2 inform the *UI* of sketch mode; C1 makes the *kernel* — they proceed in parallel and meet at Phase C integration.
- C2 needs no UI; R4 needs nothing from Claude — fully parallel.
- C4 is research-only so it can't collide with anything.

## Definition of done (any task)

1. Acceptance criteria met **and cold-run proven** on the reference machine (i5-6600/HD 530/8 GB).
2. Evidence attached (test output or screenshot in `test_evidence/`).
3. Posted in `messages/` with what changed, what to review, and what blocks next tasks.
