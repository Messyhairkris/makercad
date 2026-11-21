# MakerCAD Next-Gen — Comprehensive Design Review & Roadmap

**Prepared by:** Crush (Senior Design/Engineering Review)
**Date:** 2026-08-01 — **v2 update:** incorporates Kris's product walkthrough (directed, plan-driven experience), corrected ground truth on the HAVEN demos and FoldCAD, 2D-sketch snap model, and the Three.js platform question.
**Scope:** Audit of the entire local MakerCAD ecosystem, all Learning Projects experiments, and three external reference projects (FuCad, box-pleating studio: BP Studio, LibreCAD). Produces a single recommendation: architecture, organization, and phased roadmap for the next version of MakerCAD — Fusion 360-style controls, OpenCASCADE under the hood, packaging/box design, on genuinely low-spec hardware.
**Target hardware (unchanged):** Intel i5-6600 / 8 GB RAM / Intel HD Graphics 530 (no Vulkan, OpenGL 3.3 via Compatibility renderer / ANGLE).

> All findings below come from reading the actual files and repositories listed in the Source Material appendix. Where sources contradict each other, both positions are stated and marked **[CONFLICT]**.

---

## Part 1 — Executive Summary

1. **You do not need to start over.** The hardest, most unusual problems of this project are documented in depth: OCCT tessellation pitfalls (face orientation, normals, `TopLoc_Location`), the ANGLE/HD 530 rendering trap (no `GL_LINES`), the coordinate-space invariant, the staged rendering validation pipeline, and the design of the parametric core (dirty-flag history, SelectionIntent, functional pipeline hashing). **[CORRECTED v2]** Kris's ground truth: every demo failed in some way, mostly defeated by the hardware's hard rules. The *documentation* is the lasting asset; the *rendering claims* (BoxMesh 60 fps, etc.) are unverified hypotheses. Plan for a platform/render spike, not an assumption that rendering is solved.
2. **What does not exist yet is a *product*.** Work is fragmented across 4+ overlapping prototypes (GDExtension at `C:\dev\makercad-extension`, LoCAD v1/v2, ThreeJS_CAD, FoldCAD), with contradictory status documents and no single integrated application. The next version must be a *convergence effort*, not a new build.
3. **Recommended architecture (v4):** OCCT 7.9.3 stays the B-rep kernel; a shell (Godot-vs-Three.js decided by spike — Part 7 Q1) renders and hosts interaction; a packaging mode (dieline + folding) lives inside the app; and a new **Directed Experience System** (Part 5.6) wraps it all — MakerCAD is *not* a tool you must already know how to use; it is a plan-driven guide that assembles the right tools per project and walks you through them.
4. **The Fusion 360 "feel" is the differentiator and is cheap to get right:** MMB orbit / Shift+MMB pan / scroll zoom, S-key shortcut search, right-click marking menu, bottom timeline, left browser tree, sketch-first workflow, click-click dimensioning with Tab navigation. Most of this was already prototyped and *proven* in ThreeJS_CAD (v1.3) — those patterns should be ported, not reinvented.
5. **Hard rule for v4: no new geometry backends, no wishful code reuse.** The value of ThreeJS_CAD, LoCAD, and FoldCAD is extracted lessons only — ThreeJS_CAD's interaction patterns and rendering budget, LoCAD's interaction doctrine, FoldCAD's packaging domain documentation (its code must not ship: it demonstrably cannot load or fold a real dieline). FuCad/BP Studio/LibreCAD are reference reading.

---

## Part 2 — Current State Audit (Local MakerCAD Ecosystem)

### 2.1 The authoritative plan: `MakerCAD_Project_Plan_v3.md`

Still the correct spine for v4. Key decisions that remain valid, and should *not* be re-litigated:

| Decision | Verdict |
|---|---|
| OCCT 7.9.x prebuilt (never compile from source on this machine) | Keep. Now on 7.9.3 (per Phase1_Addendum) — fine. |
| Godot 4.6.2 Standard + Compatibility renderer | Keep, non-negotiable for HD 530. |
| GDExtension (C++) with **no OCCT types across the boundary** (IDs/plain data only) | Keep. This rule enables future IPC migration and has been verified PASS in Phase 1. |
| Linear history with **dirty-flag partial rebuild O(k)**, functional pipeline hash, mesh cache keyed by hash | Keep. This is the correct v1 answer to rebuild-cost explosion. |
| SelectionIntent + HeuristicMatcher (0.85 confidence) + graceful "broken" state + Fix-flow UX | Keep. Honest alternative to the unsolved topological naming problem. |
| Freeze History as the 100+ op escape valve | Keep. |
| planegcs (from FreeCAD source) as sketch constraint solver, limited constraint set | Keep for Phase 4+; nothing better exists that is embeddable and LGPL. |
| AI as privileged client, Phase 7, same operation API | Keep. The already-defined MCP tool schema in the plan is good. |
| Low-fidelity drag mode (bounding box during interaction) | Keep — required, not optional, on HD 530. |

One update needed: plan says build system "CMake + Ninja"; Phase 2 docs show NMake in `rebuild_clean.bat`. Harmonize on **CMake + Ninja** everywhere and fix the build scripts.

### 2.2 The real GDExtension — status

**Location authority problem:** `MakerCAD\makercad-extension\` is a stub containing only `demo\phase2_test.gd`. The living code is at `C:\dev\makercad-extension\`. **[Action: move/merge into the MakerCAD tree — see Part 5.]**

What actually exists there (verified from source headers and build files):

- **Build:** CMake 3.27+, `/MT` static runtime matching godot-cpp (pinned `godot-4.5-stable`, API-compatible with 4.6.2 — the `4.6.2` tag does not exist; document this), MSVC v143. `win_clean.h` include-first convention + `/wd2589` for the min/max macro war with OCCT. Vendored but unused `nlohmann/json` (params currently hand-parsed — tech debt to pay down in v4).
- **Registered classes:** `MakerCADExtension` (MeshInstance3D), `CameraController`, `WorkplaneGrid`, `WireframeVisualizer`.
- **Internal engine (PIMPL throughout):** `ShapeStore` (OCCT containment boundary, hash-indexed), `HistoryEngine` (dirty-flag rebuild, undo/redo depth 50, preview mode, JSON serialization), `HistoryNode` + `SelectionIntent`, `OperationRegistry`.
- **OCCT ops implemented:** create_box, extrude (`BRepPrimAPI_MakePrism` on face), boolean_cut (`BRepAlgoAPI_Cut`), boolean_union (`BRepAlgoAPI_Fuse`), fillet (`BRepFilletAPI_MakeFillet` — **fillet-all-edges; per-edge SelectionIntent targeting is a known gap**), wire/face builders, `BRepCheck_Analyzer` validity, STEP export (AP203/AP214), ray picking (`IntCurvesFace_Intersector`), tessellation (0.5 mm / 0.5 rad deflection), `ShapeAnalyzer` primitive detection (box/cylinder/sphere/cone/torus → route to Godot primitives).
- **Phase status — paper vs. reality [CORRECTED v2]:** on paper: Phase 1 complete; Phase 2 complete after expert review; Phase 3 code-complete but untested; Phase 4 STEP export coded. **Kris's ground truth (2026-08): every demo has failed in some way, mostly defeated by the hardware's hard rules.** Treat all phase gates below ALMOST-certainly not met in practice until re-proven. The code inventory above (classes, ops, build system) remains valid and reusable; the passing/failing status of every scene and claim is unverified. Direct-API wrappers in `makercad_extension.cpp` are still TODO stubs (union/subtract/intersect, fillet/chamfer wrappers, IGES, physical properties).

### 2.3 Rendering on Intel HD 530 — documented approaches, UNVERIFIED (v2 correction)

The session logs record five failed approaches (SurfaceTool, raw ArrayMesh, CSGPolygon3D, CULL_BACK, GL_LINES) and then claim breakthroughs (BoxMesh solids at 60 fps, CylinderMesh wireframes). **Kris's ground truth overrides this: none of the demos work.** The following should be treated as *hypotheses to re-test*, not doctrine:

- **Solids (hypothesis):** built-in Godot primitives (`BoxMesh`) + `StandardMaterial3D` with `CULL_DISABLED`, `SHADING_MODE_PER_PIXEL`, with OCCT as computational truth and `ShapeAnalyzer` routing primitives vs. tessellated ArrayMesh fallback.
- **Wireframes/edges (hypothesis):** never GL lines (hard 1-px cap under ANGLE ES 3.0); thin `CylinderMesh` per edge with dynamic zoom-based scaling (`clamp(1.0 + (ratio-1.0)*0.6, 1.0, 1.3)` past distance 60).
- **Grid (hypothesis):** `material.render_priority = -1`; **never write `gl_FragDepth`**; no geometry shaders under GLES3/ANGLE.
- **Budget:** ≤ ~200k triangles/frame for 30 FPS; low-fidelity bounding-box mode during any drag.
- **Regression discipline:** the 8-stage validation pipeline (Expert Review §3–4) — hardcoded Godot-only cube first, culling-last, picking last. This is now mandatory for any rendering regression in v4.

### 2.4 The Godot product shell: `maker-cad-program`

Currently a thin project: LoCAD v2 installed inside it (one scene, `MainController.gd` + `OrbitCamera.gd` + `STLExporter.gd`), plus 4 addons (`gdscript2all`, `advanced_radial_menu`, `stl-io`, `godot_mcp`). Working: box creation (B), select/highlight, orbit/pan/zoom, STL export. **Not:** move tool, extrude tool, integration with the GDExtension. The README is a leftover addon README — cosmetic but telling: the shell has no identity yet.

### 2.5 Documentation debt **[flagged]**

- `Important file locations.txt` labels **all 11 demo scenes "Does not work"**, contradicting HAVEN breakthrough docs. **[RESOLVED v2]** Kris confirms: all demos failed. The "Does not work" labels were accurate; the HAVEN "breakthrough" claims were not. Historical lesson for v4 process: *a solution claim is not real until it survives a cold re-run on the reference machine, recorded in STATUS.md with the test command.*
- Three copies of LoCAD exist (`v1/`, `locad_v2/`, `project/` duplicates) + an empty `Learning~Projects/` junction. High risk of divergence mistakes.
- Multi-agent session logs (Codey, Haven, Otherguy) are excellent individually but scattered. v4 needs one canonical HANDOFF + STATUS file per repo (FoldCAD's AGENTS.md/HANDOFF.md discipline is the model to copy).

---

## Part 3 — Learning Projects Audit (what each experiment taught us)

### 3.1 FoldCAD — packaging/dieline/fold engine prototype
`Learning Projects\FoldCAD\` · Vanilla JS + Three.js r128 (CDN, zero build) · Status: rebuild scaffold (Phase 0–1), 1,807-line v3 prototype archived with broken core.

**Verdict [CORRECTED v2]: study only — zero code reuse.** Per Kris: FoldCAD does not work *even slightly* — it looks polished but cannot load or fold a single real dieline. Its SVG import is a fake (an `<img>` backdrop, not a parser). The salvageable asset is 100% documentation: `PACKAGING_ENGINEERING.md` is production-grade (material taxonomy SBS 0.3–0.6 mm / E-B-C-flute / kraft; CITO crease formulas `B = c×1.5 + d` paperboard, `B = c×2.0 + d` corrugated; bend radii; FEFCO 0201 flap rules; grain direction; delamination physics). `INSPIRATION.md` correctly identifies OrigamiSimulator as the hinge-kinematics reference and templatemaker.nl SVG as the import format.

**Lessons for v4:**
- dieline 2D mm (X-right/Y-down) → 3D Y-up coordinate convention, documented explicitly; hinge-graph folding (panels rotate, children follow) — *kinematic only*, FEA/cloth sim banned.
- v3's failure modes to avoid: single 25-prop mutable global state; fake SVG import (an `<img>` backdrop!); hardcoded exports; wrong fold-sign math on bottom flaps. All traced to "UI before engine." The packaging module of v4 gets its *fold engine first, UI second*.
- FoldCAD's own `ROADMAP.md` 9-phase task list is directly reusable as the packaging module's plan.

### 3.2 LoCAD — low-spec interaction research (Godot/CSG)
Status: v1 archived at ~60% (orphaned InteractionController, good architecture skeleton: DependencyGraph, HistorySolver with immutable snapshots, typed Resource params); v2 = minimal runnable (boxes/select/orbit/STL); **chair benchmark 0/6**. Dormant since May 2026.

**Lessons for v4:**
- *"Users forgive missing features. They do not forgive broken interaction."* — this should be printed at the top of v4's design doc.
- CSG (Godot native) is a dead end as the kernel — it's the visual fallback only. The typed-params/DependencyGraph/HistorySolver design study from v1 is absorbed into MakerCAD's HistoryEngine (already superior).
- `Godot_Fork_Feasibility_Report.md`: plugin/addon path over engine fork — validated; the editor viewport is compiled out at export, forget embedding it.
- The **chair benchmark** (sketch seat → extrude → revolve legs → pattern → fillet → STL) is adopted as v4's end-to-end acceptance test.

### 3.3 ThreeJS_CAD ("makercad-threejs") — the most advanced working prototype
React 18 + TS + Vite + Three.js r160 + Zustand; CSG in a Web Worker (`three-bvh-csg`), BVH raycasting (~0.3 ms), fillet/chamfer, face push/pull, sub-object selection, context menu, perf monitor, Intel HD 530 budget validated (~45 fps, ~20 draw calls). ~50 source files, ~25 docs, v1.3 (May 2026).

**Lessons for v4:**
- This project *proved the UX concepts*: Autodesk-style right-click context menu, click-click dimension dialog with Tab navigation, sub-object (face/edge/vertex) selection, ghost previews. Port the interaction patterns to Godot; do not port the stack.
- Web-Worker CSG is the web analog of v4's "async tessellation/rebuild thread" — same principle: keep the UI thread clean on 4 cores.
- `EXPERT_INTEGRATION_PLAN.md` render optimizations (EdgesGeometry over wireframe, Lambert previews, on-demand rendering cutting GPU load ~80%, geometry disposal) map 1:1 onto Godot equivalents.
- `PROPOSED_IMPROVEMENTS_V1.3.md` (ThreeJS version + MakerCAD-root version) contains the suggestive-interface research (Igarashi Chateau thumbnail suggestions, Rossignac feature taxonomy) — adopt as v4 Phase-6+ UX ideas, not v1 scope.

### 3.4 RustCAD
Empty directory. Never started. **[Action: delete or repurpose; Rust + OCCT bindings (`occt-rs` immaturity) would fight the "no new languages" rule anyway.]**

---

## Part 4 — External Reference Review

### 4.1 FuCad (FadyFaheem/FuCad)
**What it is:** an early-stage fork of FreeCAD that keeps OCCT, the document model, Python API, and `.FCStd` format intact, and reworks only the UX along Fusion 360 lines — a *unified, task-driven interface instead of workbench-per-discipline*. Stack: OCCT + Coin3D (Open Inventor scene graph) + Qt + Python. No releases; build from source.

**Lessons for v4:**
1. **Validates the thesis from an independent direction:** the geometry kernel is not the product; the *curated, unified UX* is. FuCad and MakerCAD converge on the same insight — don't rebuild OCCT, re-*present* it.
2. MakerCAD's strategic framing ("not a CAD engine, a simplified parametric modeling experience built on OCCT") is exactly FuCad's architecture in miniature. Keep FreeCAD source as the OCCT reference implementation (already in the v3 plan) and **watch FuCad for UX patterns** worth borrowing: how they collapse workbenches into task flows, their toolbar organization, how they keep `.FCStd` compat while simplifying the UI.
3. Cautionary contrast: FuCad inherits FreeCAD's full weight (Python runtime, Qt, hundreds of MB). It will *never* run on an HD 530 / 8 GB as comfortably as MakerCAD targets. That is our moat; don't erode it with feature envy.

### 4.2 Box Pleating Studio (bp-studio) — the strongest packaging reference
**What it is:** browser app for super-complex origami crease-pattern design (box pleating, GOPS stretching gadgets), v0.7.15, MIT license. Stack: Vue 3 + **PixiJS 7 for all 2D rendering** + Bootstrap, rsbuild, PWA (workbox), mocha + Playwright tested, runs on everything including weak hardware, heavy computational geometry with input/collaboration from Jason Ku (MIT origami-math researcher) and Robert Lang's theory.

**Lessons for v4's packaging mode:**
1. **A 2D canvas engine (PixiJS) handled all interactive graphics;** no 3D was needed for the core design surface. For MakerCAD's dieline editor, the equivalent in-Godot choice is a 2D Control/CanvasLayer surface (not a squinted 3D view) — BP Studio proves 2D-first is the right interaction model for foldable layouts.
2. The mathematical core (hinge assignment, crease validity, layer ordering advice from Ku/Lang research) is exactly the class of algorithms FoldCAD's docs already catalog. BP Studio is evidence these algorithms are *tractable on weak hardware* when the data structures are clean.
3. Process lessons worth copying outright: Playwright E2E tests + mocha unit coverage on a hobbyist CAD-adjacent app; PWA packaging for distribution; i18n from day one (vue-i18n); BrowserStack for low-spec compatibility testing — MakerCAD's equivalent is simply *testing on Kris's own machine*, which is the spec.
4. It is **not** a packaging/CAD tool per se (origami crease patterns, no materials/FEFCO/dieline domain). FoldCAD's PACKAGING_ENGINEERING.md remains the domain source; BP Studio is the *interaction and engineering-quality* reference.

### 4.3 LibreCAD — the low-spec survival reference
**What it is:** mature 2D CAD (QCAD community fork), Qt 6.4+/C++, GPLv2, 30+ translations, libdxfrw for DXF/DWG, ESRI shapefile import, CLI converters (`dxf2pdf/png/svg`).

**Lessons for v4:**
1. **Proof of the target market:** LibreCAD thrives precisely on machines like the target. Lean UI, no GPU assumptions, decades of low-spec discipline. When in doubt about a performance-sensitive choice, ask "what would LibreCAD do" — usually: less.
2. **Study, don't integrate:** useful internals to read — their entity/command architecture, layer/block system, snap system, and the deliberate separation of kernel (`librecad/src/lib`) from UI. Their use of small focused libraries (libdxfrw, muparser) over monolithic deps mirrors MakerCAD's "no new dependencies" instinct (FoldCAD AGENTS.md).
3. **CLI batch converters are a cheap, lovable feature:** `makercad.exe step2stl in.step out.stl` costs almost nothing once STEP/STL I/O exists (it does, in code). Steal this idea.
4. LibreCAD's DXF IO + BP Studio/FoldCAD's SVG lineage converge: **dieline import/export for packaging should target SVG + DXF** (laser cutters/plotters are the physical output path for boxes).

---

## Part 5 — The v4 Product: Definition & Architecture

### 5.1 Product statement
**MakerCAD v4** is a single application: parametric solid modeling with Fusion 360-style controls (OCCT under the hood), plus a packaging/box-design mode (dieline + foldable 3D preview), running at 60 fps UI / 30+ fps modeling on an i5-6600 + HD 530 + 8 GB.

**The product difference (Kris's walkthrough, 2026-08):** Fusion 360 is a tool — it assumes you already know the model you want and how to build it. **MakerCAD is a guide.** The user arrives with a *project* ("build a desk for this weird alcove"), not an engineering plan. MakerCAD turns the project's own description into a curated toolset, walks the user through capturing their reality (scans, materials, tools, hardware), helps them model inside their actual space, and finishes with printable build instructions — then closes the loop by letting finished projects be published so others can build them. Learning, completing, and sharing plans is the product; CAD is the medium.

### 5.2 Consolidation — the single most important v4 action
Today: 1 real extension (outside the project tree), 1 stub extension dir, 3 LoCAD copies, 1 empty RustCAD, 1 archived web prototype, 1 archived FoldCAD prototype, and contradictory status docs. v4 begins with a **monorepo**:

```
MakerCAD/                        (single git repo)
├── app/                         Godot 4.6.2 project (Compatibility) — THE product
│   ├── addons/                  godot_mcp, stl-io (keep); prune unused
│   ├── scenes/  scripts/        GDScript UI/interaction layer
│   └── docs_in_app/
├── extension/                   C++ GDExtension (moved from C:\dev\makercad-extension)
│   ├── src/  third_party/  CMakeLists.txt
│   └── tests/
├── docs/
│   ├── PLAN.md                  (v3 plan, updated to v4)
│   ├── STATUS.md                (single live status — all other status docs archived)
│   ├── HANDOFF.md               (single live handoff, FoldCAD-style)
│   ├── DECISIONS.md             (ADR log — decision, date, rationale)
│   ├── packaging/               PACKAGING_ENGINEERING.md + FoldCAD analysis (imported)
│   └── archive/                 (read-only: v1/v2/v3 plans, session logs, HAVEN/CODEY logs)
├── benchmarks/                  chair benchmark script, cannon test, perf harness
└── Learning Projects/           FROZEN (reference only, header README says so)
```

Rules: one STATUS.md, one HANDOFF.md; old copies moved to `docs/archive/` on day one. `Learning~Projects/` junction deleted. `C:\Users\<you>\MakerCAD\makercad-extension\` stub deleted after merge. Everything else is read-only history.

### 5.3 Layer architecture (unchanged from v3 plan, now enforced)

```
┌──────────────── Godot 4.6.2 Compatibility — Shell ───────────────┐
│ Fusion UX layer: sketch-first tools, S-key toolbox, marking menu, │
│ timeline (history panel), browser tree, ViewCube, gizmos          │
│ Packaging mode: 2D dieline editor (Control nodes) + 3D fold view  │
└────────────── GDExtension API — IDs + plain JSON only ───────────┘
┌──────────────── C++ Extension (extension/) ───────────────────────┐
│ HistoryEngine · ShapeStore · OperationRegistry · Tessellator      │
│ HeuristicMatcher · FoldEngine (hinge kinematics, new) · Exporters │
└──────────────────────── OCCT 7.9.3 only below here ──────────────┘
```

- **FoldEngine placement:** hinge-graph folding needs no B-rep; it operates on the dieline's panel graph (mesh-level). Implement it in the extension (C++, testable, reuses ShapeStore for optional thick-shell export back to OCCT) — NOT as GDScript (FoldCAD v3 taught: fold math in the UI layer rots).
- **Packaging → solid bridge:** "wrap product" workflow = dieline folded → export folded box as STEP/STL; and reverse-simple case: select a solid → bounding-box dieline auto-nest (deferred, Phase 9+).

### 5.4 Fusion 360 control mapping (explicit contract)

| Fusion 360 | MakerCAD v4 (Godot) | Notes |
|---|---|---|
| MMB drag orbit / Shift+MMB pan / wheel zoom | Proven pattern in LoCAD v2 + extension `CameraController` (re-verify — Part 2.2 correction) | Add Fusion's orbit-around-selection, pivot on empty-click |
| F-key Home / ViewCube top-right | ViewCube exists as class in LoCAD v1; port to extension | Click face/edge/corner snaps |
| Left Browser tree | Scene/History browser panel, driven by `HistoryEngine::get_history_view()` | Broken-op red state + Fix flow per v3 plan |
| Bottom timeline | History timeline with dirty/broken/frozen glyphs | Freeze History = right-click op |
| S-key shortcut search box | Command palette (fuzzy), feeds AI tool schema too | Same JSON registry serves both |
| Right-click marking menu | `advanced_radial_menu` addon already installed in the shell | Wire to context-sensitive ops (ThreeJS_CAD ContextMenu patterns) |
| Sketch → Extrude → Fillet verb toolbar | Toolbar tabs MODEL / PACKAGE / EXPORT (3 modes, not FreeCAD's ~15 workbenches — FuCad lesson: collapse, don't multiply) | |
| Direct push/pull on faces | Face pick (`IntCurvesFace_Intersector` — code exists, re-verify) → param dialog + ghost preview | Preview-debounce ~150 ms per v3 plan |
| Touch/pen | Godot native; 3-finger pan was in Phase 2 acceptance | |
| (no Fusion equivalent — sketch view snap) | **Sketch mode snaps the camera to a true top-down orthographic view of the chosen sketch plane** (workplane normal facing user); exit sketch returns to 3D perspective. The program is otherwise "mostly 3D" — 2D is a mode you enter, not a place you live | Kris requirement, v2 walkthrough |
| Viewport camera pivot (Kris, 2026-08-11) | Orbit pivot = selected point/edge/face when something is selected; else **center of the main mass** (AABB center); else **ground-origin (0,0,0)**. The ground plane is a **hard barrier** — the camera may never pass below it (game-mechanic collision, exploit Godot physics since we're in Godot). Implemented in the test shell's orbit camera (`game/lowfi_orbit_camera.gd`), carry into the product camera controller | Kris requirement, v4 visual-test feedback |
| Ground grid | Visible reference grid with minor/major lines doubling as the camera barrier and skid reference; shader-drawn plane (fwidth lines), no depth hacks | Kris requirement, v4 feedback |

**Readability rule (Kris, 2026-08-11 — do not re-litigate):** edges must be clearly defined and face sides strongly differentiated (deeper key/ambient contrast, thicker edge ribbons). This was the previous generation's biggest pitfall; the bar is "looks obviously readable on the HD 530", set at v4 Phase C and locked — no endless re-tweaking. Borrow proven recipes (ThreeJS_CAD, CADaver, godot-line3d), measure once, move on.

**Readability decisions (locked 2026-08-11):** (1) solid faces render `CULL_DISABLED` — watertight-solid rendering must be immune to per-face winding quirks; cost is bounded by the 200k-tri budget; winding/normals fixes in `triangulate()` remain correct-by-construction but are no longer load-bearing. (2) Edge ribbons take `width_px=2.5` plus a small clip-space depth nudge so ribbons always win over coplanar faces. (3) Low-fi proxy is bright-orange translucent — it must be unmistakable as a stand-in. (4) Product lighting: strong warm key (1.4) + cool weak fill (0.4) + dark ambient — this is the side-separation recipe, do not redesign per scene. Kris's remaining request "interiors of solids must read as solid, outer form dominates" is satisfied by (1)+(4): interiors are drawn but never opaque-dominant because the exterior form always occludes correctly from external views.

### 5.5 Performance contract (carry over, re-verify before Phase A ends)

30+ fps viewport, ≤200k tris/frame (ANGLE), <600 MB RAM, <3 s cold start, <2 s 50-op rebuild, low-fidelity bounding-box drags, ≥0.85 SelectionIntent confidence, no GL_LINE primitives, render-priority grid, no `gl_FragDepth`. (The primitive-vs-tessellated rendering strategy is itself an open spike result — Part 7 Q1/Q2.)

### 5.6 The Directed Experience System (v2 — the actual product layer)

Fusion 360's UX table (5.4) describes *how tools feel*. This section describes *which tools exist on screen and why* — the layer that makes MakerCAD a guide instead of a toolbox. Default persona: **Maker** (woodworking/workshop projects); the architecture must generalize (see personas, below).

#### 5.6.1 Project intake flow
1. **Start menu:** New project / Continue existing. No blank canvas with 200 icons.
2. **Name + describe:** the user names the project and *describes it in plain language* (goal, space, constraints, what they have, units preference — e.g. inches for most US users).
3. **Toolbox linter:** a local, rule-based text analyzer scans the description for keywords/phrases ("wood", "plywood", "reclaimed", "scan", ".splat", "drawer pulls", dimensions, file formats) and assembles a **project Toolbox** — only relevant tools appear in the sidebar. Everything else is hidden by default; experts can pin tools as always-available once they know their workflow.
4. Extended example (Kris's walkthrough): "Custom desk Project" → linter loads woodworking tools, dimensioning in inches, scan/splat import tools, material inventory, hardware modeling. "Reclaimed" + "drawer pulls" → prompts for board photos (for texture-mapped preview) and pull models (manufacturer link, existing STL/OBJ, or guided photo-modeling).

#### 5.6.2 Reality capture: environment, inventory, tools
- **Environment:** splat files or 3D models load as the modeling backdrop, displayed **in a generic room at correct real-world scale** when no environment exists. Scale correction is a first-class tool: enter one known measurement, or draw/fit a reference box.
- **Splats are not geometry** (no edges like photogrammetry): an open research problem (Part 7 Q4) — the user *inhabits* the splat as context while modeling real parts against it. Survey the field before designing.
- **Material inventory:** user lists materials on hand (plywood sheets, reclaimed boards, etc.) with real dimensions. The **materials estimator** checks the design against inventory as it evolves ("this rough-in uses more wood than you own") and parametric controls can snap design dims (board thickness) to inventory dims. Designing beyond inventory is allowed but clearly outside the "use what you have" path.
- **Tool & workspace inventory:** captured *with* materials, early in intake (Kris correction: not at the end) — saws, drills, printers, bench size drive feasibility of joinery and build plans later.
- **Hardware/part library:** parts (e.g. vintage pulls) are modeled once via a guided photo walkthrough (top/sides/front/back/bottom) — or imported (STL/OBJ) — then stored in a **local part database** for reuse, export, and sharing. Right-click a library part → "Use Part" in the design. Experimental, explicitly AI-assisted model generation may live here and nowhere else.

#### 5.6.3 The Assistant character
An animated 3D character lives in the workspace — RPG-NPC style. Default ~5'6" tall, **always rendered at true scale relative to the model**, grounding the user in the scene's real dimensions (a human standing beside your desk communicates scale better than any dimension line). It introduces itself, explains tools on request, walks guided flows (photo modeling, scale correction), and is the voice of the lint/prompting system. Keep it cheap: simple rig, low-poly, prebaked animation clips — on an HD 530, the assistant is a node, not a render feature.

#### 5.6.4 From model to built object
When the design satisfies budget and inventory, MakerCAD generates the **build plan**: BOM, build steps in required order, timeline, flagged issues (tool gaps, tight fits), all printable as instructions to build the object by hand.

#### 5.6.5 Sharing & the community loop
On completion, the user is prompted for photos and corrections; the project publishes as a **buildable plan** (design + inventory + tool list + steps). Others load it *personalized*: their own inventory/tools are substituted before MakerCAD tells them how to build it. Part library items share independently of projects.

#### 5.6.6 Personas (extensibility contract)
| Persona | Output | Notes |
|---|---|---|
| **Maker** (default) | Buildable plans, BOM, printable instructions | The walkthrough above |
| **VFX / Animator** | Scene assets, splat-aware props, scale references | Reuses environment + assistant + library; drops build plan |
| **Game developer** | Export-optimized meshes, real-scale props | Units + library + export focus |
| *(future niches)* | — | Persona = curated toolbox + intake prompts + output pipeline. The linter, toolbox, assistant, and sharing layers are persona-agnostic from day one. |

**Design rule:** never special-case the Maker persona into core code. Intake linter rules, toolbox manifests, and output templates are data files loaded per persona.

#### 5.6.7 Sketch mode (2D) inside a mostly-3D program
Per Kris: outlines/shapes are often drawn in 2D. When the user picks a sketch plane, the camera **snaps to top-down orthographic** on that plane and drawing tools behave 2D (snaps, dimensions); leaving sketch mode restores the 3D view. This is the *only* standing 2D mode besides the packaging dieline editor — the rest of the program is 3D-first.

---

## Part 6 — Phased Roadmap

Phases are gated by *exit criteria*, not feelings. Estimated as sessions, not days.

| Phase | Deliverable | Key work | Exit criteria (all measurable) |
|---|---|---|---|
| **A. Consolidation & Ground Truth** | Monorepo, one status doc, clean build | Move extension into repo; archive stale docs; fix Ninja build scripts; delete stub/junction/RustCAD | `ninja` clean build; cannon test (cylinder_wireframe + one OCCT box) renders correctly **on the HD 530 machine**; STATUS.md exists and everything contradictory is archived |
| **B. Phase 3 shakedown** | Actually-working parametric core | Port history/ops/undo into `app/`; test the untested Phase 3 code; replace hand-rolled JSON with vendored nlohmann; close fillet-per-edge gap via SelectionIntent | 20-op scripted history rebuild <1 s; fillet targets a *specific* edge; undo/redo across 50 steps stable; ghost preview debounced & cancellable |
| **C. Fusion UX layer** | Feels like Fusion 360 | Command palette (S), marking menu, timeline, ViewCube, orbit-around-selection, sketch tools (line/rect/circle/arc) on grid, click-click dimensions w/ Tab | A new user models a bracket (sketch→extrude→fillet→hole) in <10 min; chair benchmark steps 1–2 pass |
| **D. Packaging mode — dieline** | 2D dieline editor | Panel/crease taxonomy (cut/fold/perf/glue), parametric templates (RSC/FEFCO 0201, tuck-end, tray), CITO crease-width math, SVG export | Draw FEFCO 0201 with correct flap rules + material-aware bend radius; SVG opens in a plotter workflow |
| **E. Packaging mode — fold engine** | Live 3D fold preview | Hinge-graph kinematics in extension; fold slider; mountain/valley validation; folded → STL/STEP export | Any dieline from D folds without panel penetration; folded box exports load in a slicer |
| **F. File I/O & resilience** | Real files, real recovery | `.makercad` v1+2 (dielines), migration chain, STEP/IGES/STL/3MF, broken-op Fix flow UX | Open/edit/save round-trip; intentionally break an op → Fix flow recovers with params intact |
| **G. Hardening & perf** | Shippable beta | Async rebuild thread, freeze history UI, RAM audit, cold-start audit, full staged-validation rerun | All v3 performance contract numbers measured and green on reference hardware |
| **H. AI client (optional Phase 7 follow-through)** | AI panel | Existing MCP tool schema, claude-sonnet-4, privileged-client discipline | AI builds the chair benchmark from text |

FuCad/BP Studio/LibreCAD are re-consulted at C (UX patterns), D/E (folding + PWA-style simplicity), and G (low-spec discipline) respectively.

---

## Part 7 — Risks & Open Decisions

1. **[RESOLVED v2]** HAVEN-vs-"Does not work" conflict: Kris confirms all demos failed. New process rule — no claim enters STATUS.md without a cold re-run on the reference machine. Consequence: Phase A is a *re-baseline*, not a shakedown.
2. **Per-edge fillet via heuristic matching is unproven.** It's the linchpin of usable modeling UX. Budget explicit spike time in B; fallback = fillet-all with radius zeroing on excluded edges (ugly but functional).
3. **Scope creep into Fusion parity** (sweeps, assemblies, sheet metal proper). Defense: the v3 mantra — "ship a dumb but stable CAD"; sweeps/lofts stay deferred regardless of temptation.
4. **Packaging scope:** flat-plane hinge kinematics ≠ 3D-thick folding ≠ dieline nesting. v4 does kinematics + FEFCO templates only; nesting/optimization is BP-Studio-grade research territory — defer.
5. **Q1 — Platform: Godot Compatibility vs Three.js (WebGL).** Kris is open to Three.js given ThreeJS_CAD proved ~45 fps on the target. Honest tradeoffs — **Godot keeps:** existing C++ GDExtension investment (ShapeStore/HistoryEngine compile today), native OCCT linkage at full speed, desktop distribution, touch/pen, the team knowledge base. **Godot risks:** ANGLE path is exactly where demos kept dying; Godot shell work so far has not produced a stable demo. **Three.js wins:** proven on-hardware framerate in THIS repo's history, huge modern graphics path (and WebGPU option off Intel), easy sharing/PWA (matches 5.6.5), mature 2D overlays. **Three.js costs:** OCCT must run as WASM (compile once on a fast machine; ~30-60 MB binary; rebuild cost on 8 GB is real) or as a local native service with the web shell as client; macOS-level C++ tooling needed; Electron/Tauri if desktop packaging required. **Recommendation: decide by spike, not preference** — Phase A builds the SAME trivial acceptance scene in both shells (orbit + select + one editable parametric box + grid + 200k-tri stress mesh) and measures cold start, fps during drag, RAM, and build iteration time on the reference machine. Decision recorded in DECISIONS.md; the losing shell's assets are archived, not maintained in parallel. The OCCT kernel, history model, and JSON API boundary are identical either way — that investment is platform-proof by design (the v3 boundary rule pays off here).
6. **Q4 — Splats as modeling context.** Splats have no edges/B-rep; how the user "inhabits" a splat while modeling exact parts is unsolved. Survey the field in Phase A/F (photogrammetry-CAD hybrids, VR splat editors, Blender GS plugins). v4 commits only to: splat loads as backdrop, scale-correction tools, occlusion/off option while sketching. Anything more needs its own design pass.
7. **Assistant character scope risk.** An animated NPC can eat a GPU budget and a schedule. Cap it: low-poly (<10k tris), prebaked clips, no lip sync, deletes cleanly behind a settings toggle. If the performance contract degrades, the Assistant ships as a 2D portrait avatar instead.
8. **Sharing/community infra is out-of-scope-but-designed-for.** Personalized re-planning against another user's inventory is a local-data problem (good: no server). Distribution itself (hub/registry) is deliberately not in v4 phases; format the shareable plan as a single self-describing package so a hub can be added later.
9. **Open decisions to log in DECISIONS.md during Phase A:** 1 unit = 1 mm confirmed? float→double policy at zoom (Expert Review §6)? flat vs averaged normals at shared verts? camera DI for multi-viewport? Godot 4.5-vs-4.6.2 godot-cpp pin? Persona/manifest file formats (5.6.6 design rule)?

---

## Part 8 — Recommended First Two Weeks (concrete)

1. Create `docs/STATUS.md`, `docs/HANDOFF.md`, `docs/DECISIONS.md`; archive every other status/session doc; announce in-repo that these three are canonical.
2. Move `C:\dev\makercad-extension` → `MakerCAD/extension/`; delete `MakerCAD\makercad-extension\` stub, `Learning~Projects` junction, empty `RustCAD/`; freeze `Learning Projects/` with a README header.
3. Unify build on CMake + Ninja; fix `rebuild_clean.bat` (currently NMake); verify DLL lands in `app/bin/`.
4. Define the Phase A acceptance scene (orbit + select + editable parametric box + grid + stress mesh) and its measurement script; run it on the reference machine; log numbers in STATUS.md (this replaces "run the cannon test" and seeds the Q1 decision).
5. Swap hand-rolled JSON → vendored nlohmann; add the v3 `MakerLog` structured logging if still absent.
6. Sketch the Three.js spike scaffold (identical acceptance scene) — only if Q1 remains open after ground truth in Godot is measured.
7. Spike: per-edge fillet via SelectionIntent (2–3 sessions, timeboxed).
8. Start chair benchmark automation stub in `benchmarks/`.
9. Cold-run LoCAD v2 to verify its claimed working interaction (orbit/pan/zoom/select/STL) — same "unverified until re-run" rule as the extension demos; delete never-finished v1 skeleton references from active docs.
10. Draft `docs/personas/maker.md` (linter keyword map + toolbox manifest + intake prompts) as the first concrete artifact of the Directed Experience System — data, not code.
11. Kris's walkthrough is saved at `MakerCAD/Product_Walkthrough_Maker_Use_Case.md` — treat it as the product-requirements source for phases C2 and G; move it into `docs/` when the monorepo lands.

---

## Appendix — Source Material Reviewed

**Local (read in full or surveyed from source):** `MakerCAD_Project_Plan_v3.md`, `MakerCAD_Phase2_ExpertReview.md`, `HAVEN_SESSION_SUMMARY.md`, `HANDOFF_NEXT_HAVEN.md`, `INTEL_HD_530_RENDERING_SOLUTION.md`, `Phase1_Addendum.md`, `EXPERT_REVIEW_REQUEST.md`, `BUILD_INSTRUCTIONS.md`, `LOCAD_V2_INSTALLED.md`, `PROPOSED_IMPROVEMENTS_V1.3.md`, `Important file locations.txt`, `Research_Findings/Orion_Research_Report.md` (19 GDExtension projects surveyed; CADaver + interactive_grid + PathMesh3D + godot-line3d most relevant), extension source at `C:\dev\makercad-extension` (CMake, register_types, all class headers), `maker-cad-program` project tree, FoldCAD (9 docs incl. PACKAGING_ENGINEERING.md, ARCHITECTURAL_ASSESSMENT.md, ARCHIVE), LoCAD (plans, fork feasibility reports, three copies), ThreeJS_CAD (~25 docs, package.json verified), RustCAD (empty).

**External:** FuCad README (github.com/FadyFaheem/FuCad), Box Pleating Studio README + package.json (github.com/bp-studio/box-pleating-studio, MIT, Vue3+PixiJS7+rsbuild, v0.7.15), LibreCAD README (github.com/LibreCAD/LibreCAD, GPLv2, Qt 6.4+).

**Product vision (v2):** Kris's "A Maker using MakerCAD" use-case walkthrough (2026-08), preserved verbatim at `MakerCAD/Product_Walkthrough_Maker_Use_Case.md`.

*Document ends. Next action per this review: Phase A, task 1.*


---

## Appendix - Universal Geometry Program integration (2026-08-12)

External proposal `makercad_v4/MakerCAD_Universal_Geometry_Program_Proposal.md`
(Googie + Chad) reviewed in full. Of 41 discrete architectural ideas: 18
already implemented in v4, 17 adopted, 6 modified/deferred with reasons.

- The full comparison lives in the vault:
  `wiki/projects/makercad/60 - Universal Geometry Proposal - Comparison and Design Updates.md`
- **Phase E added to this roadmap**: "Universal Geometry Platform" - format
  adapters (STEP first), MIR supersedes the `.makercad` part doc at the D/E
  boundary, persistent cache, spatial chunks, camera-driven streaming, worker
  jobs. Proposal phase gates A-G adopted.
- v4 remains authoring-first; the viewer lane is Phase E content. Renderer
  doctrine (<=200k tris) stands until the benchmark harness justifies
  screen-space-error adaptivity.
- Dependency/licence ledger created: `makercad_v4/docs/DEPENDENCIES.md`.
- UI map (modals, orbit-to-selection, honest placeholders, capability cards):
  vault doc `61 - UI Map`.
