# 10 — Plan, Build and Publish

## 10.1 From design to build steps

Build steps are **derived, not typed**. Three sources feed the step generator (`mk-plan::derive`):

1. **Commands with `build_step` metadata** (doc 03 §3.5): `stock.assign` gives a *cut* step, `solid.hole` gives *drill*, `joint.dado` gives *route/cut dado*, `fold.crease` gives *score*, `soft.seam` gives *sew*, `finish.apply` gives *finish*.
2. **Stock layouts** (doc 08): the cut list *per board*, in the order that keeps setups together (all 33″ crosscuts, then all 15″).
3. **Assembly structure**: the assembly tree plus joint dependencies and access rules give the **assembly order**.

### Ordering rules (data, `mk-plan/order.toml`)

- **Precedence:** cut before drill; drill before assemble when access is lost after assembly; sand faces before assembly where they become inaccessible; dry-fit before glue-up; finish interior faces before assembly if they're hard to reach.
- **Batching:** group identical setups (same blade height, same fence) and minimise tool changes.
- **Waits:** glue cure (from the glue's data, e.g. 30 min clamp / 24 h full), finish dry time between coats, filament print time. Waits become **lag** on dependencies, and the scheduler puts other work into them.
- **Access:** if a screw's driver path is blocked by a later part, that screw goes before the part (ray test along the fastener axis).
- **Sewing order:** pockets and appliqués on flat panels first; zippers before closing seams; inside seams before binding; topstitch last (doc 06 §6.7).
- **Folding order:** for packaging: score, then cut, then pre-fold, then glue the manufacturer's joint, then flat-pack. The hand-assembly order comes from the fold sequence that avoids collisions.

The result is a **step graph** (a DAG). Each step has: title (from the log template), parts involved, tools (from capability matching against inventory), materials, duration estimate, safety notes, an auto-generated **step thumbnail** (exploded view of just this step, rendered by the CPU rasteriser when idle, 512 px), and tips.

## 10.2 Scheduling (PlanCraft core)

The step graph becomes a **PlanCraft project** (`plan/plan.plancraft` inside the `.maker` file):

| MakerCAD | PlanCraft |
|---|---|
| Build step | Task (with duration in working minutes, three-point estimate when uncertain) |
| Phase (Prepare stock / Cut / Assemble carcass / Drawers / Finish) | Summary task |
| "Carcass dry-fit OK", "Ready to finish", "Done" | Milestones with deliverables |
| The maker (and helpers) | People resources, with the work windows from the Quest as the calendar |
| Tools from inventory | **Equipment** resources (calendars: borrowed tools, makerspace hours) |
| Materials to buy | Tasks "Buy …" with a cost; lead time as duration |
| Glue cure / dry time | Lag on links (elapsed time, e.g. `24eh`, not working time) |
| Deadline | Project deadline; critical path |

**Views we show** (redrawn in our UI kit): **Timeline** (Gantt, simplified), **Board** (To do / Doing / Done, which also sets % complete) and **Calendar**. The other PlanCraft views are hidden unless the user switches to "Advanced plan" (team projects, small business).

**Re-planning:** design changes re-derive the steps, and a diff is applied to the plan that preserves manual edits (pinned tasks, notes, % complete). Changed steps are highlighted: "3 steps changed because you switched the top to plywood."

**Deadline health:** after each session, compare actual against plan (from the session interview and board moves). If the critical path misses the deadline, Sancho offers (once): simplify (drop a feature, which the design shows), add a work window, extend the deadline, or buy instead of make (e.g. pre-cut panels).

## 10.3 Time estimates that get better

- **Base rates:** per technique and tool (e.g. crosscut 2x4 with a circular saw: 1.5 min including marking; pocket hole: 1 min; sew 1 m straight seam: 4 min; score and fold carton: 30 s per crease). The table is `mk-plan/rates.toml` and is editable.
- **Skill multiplier** from intent (beginner ×1.8, comfortable ×1.2, experienced ×1.0).
- **Personal calibration:** actual durations from sessions update a per-user multiplier per technique, using a robust running median. Estimates say when they're calibrated: "≈ 3 h (based on your last 4 projects)".

## 10.4 The Journal

The log is a second view of the project, recorded as it happens (kept from v5 C1, adjusted).

**Record types** (`log/journal.jsonl`, append-only; edits are new records that supersede old ones):

| Record | Created by | Contents |
|---|---|---|
| `intent` | Quest, intent edits | Goal text, criteria; every revision with "why" |
| `decision` | "Why?" button; Sancho fit-solver choices; inventory overrides | Question, options considered (with thumbnails), choice, reason |
| `step` | Derived (§10.1) | Ordered instructions; editable text; photos attach here |
| `session` | Session interview (doc 07 §7.7) | Did, worked, didn't (with cause chips), next, media, mood |
| `media` | Drops, phone transfer | Photo or video (linked), caption, which step/part it shows |
| `finding_override` | Sancho | Which rule, user's reason |
| `as_built` | Compare (doc 09 §9.8) | Deviation map, callouts |
| `outcome` | Project conclusion | Final photos, what you'd change, would you build it again, tips for builders |

**Prompt moments** (data in `mk-log/prompts.toml`): project creation; first material chosen; first fit to space; design change after material picked ("what changed and why?"); first physical test-fit; session end; conclusion. Skipped prompts become **open pages**. They are visible and gentle, never punitive (doc 07 §7.7).

**Media handling:** photos import with EXIF time, so they auto-attach to the step active at that time if the plan board was in use. Video is linked by default and can be trimmed to clips by in/out points stored as metadata (no re-encoding on potato).

## 10.5 The export pack

One pipeline (`mk-export`) with many writers. The user picks a **preset** (Build at home / Send to CNC shop / Print shop dieline / Sewing pattern / Share project / Publish guide) or ticks items individually.

| Output | Formats | Notes |
|---|---|---|
| **Blueprints** | PDF (vector), DXF/DWG | Drawings environment: orthographic + section + exploded per step, dimensions in user units, title block with project name, revision and date |
| **Cut list** | PDF, CSV | Per material: part, qty, dimensions, grain, stock piece assignment |
| **Cut diagrams** | PDF, SVG | Per board or sheet: parts, kerfs, cut sequence numbers (guillotine order), offcuts labelled "keep" |
| **Bill of materials and shopping list** | PDF, CSV | Owned vs. to buy; costs; supplier columns |
| **Build guide** | PDF, single-file HTML (images inlined), Markdown | Steps with thumbnails, tools, materials, time, safety, tips. Doubles as the published guide |
| **1:1 templates (print)** | Tiled PDF (A4/Letter/A3) | Overlap margins, registration crosshairs, page grid labels (A1, A2…), **a 50 mm / 2″ test square on every page**, and "check scale" instructions. Grain line, labels, notches |
| **1:1 templates (plotter)** | Single large-format PDF (A0/36″ roll), SVG, HPGL later | Pattern and dieline plotting |
| **Projector templates** | Projector mode (§10.6), plus a PDF/PNG pack at the projector's native resolution | "With scale": a ruler band and test square included |
| **Dielines** | PDF with spot-colour layers, DXF with named layers | CUT / CREASE / PERF / BLEED / ART / DIMENSIONS |
| **Laser / plotter files** | SVG (stroke colour = operation), DXF | Kerf compensation optional; per-material settings note |
| **CAD exchange** | STEP, IGES, 3MF, STL, OBJ, glTF, URDF, DXF/DWG, `.sc` | Via SolidCraft `io` and CADCraft `dxf`/`dwg` |
| **Share bundle** | `.maker` packed (media embedded) | Opens anywhere; the recipient's own inventory is diffed on open |
| **Build log** | HTML/PDF/Markdown | §10.7 |

**Golden-file tests** exist for every writer (doc 03 §3.10). Production outputs (dielines, patterns, DXF) are also checked by opening them in common downstream tools during release QA.

## 10.6 Projector mode

For makers with a projector over a cutting table (common in sewing and leathercraft, and increasingly in woodworking):

1. **Calibrate:** project a 4-corner pattern, lay a known reference (a calibration mat, or a tape-measured rectangle) on the table, and drag the projected corners onto the reference corners. That gives a **homography** from design-mm to projector pixels. A second check projects a 100 mm square to measure. The calibration is saved per projector and per table height.
2. **Project:** a full-screen, black background, high-contrast lines window on the projector display. Shows pattern pieces, cut lines or a board's cut layout at 1:1. It can move and rotate the projected layout to match where the material actually lies (drag or arrow keys, at 0.5 mm steps).
3. **Step-through:** project one step at a time ("cut piece 3: left side panel, cut 2"), with the step's instructions on the laptop screen.
4. **Line styles** for visibility on different materials (white on dark fabric, inverted on light), and adjustable line width.
5. **Potato-friendly:** 2D vector drawing only, no 3D, so it runs on anything that can drive a second display.

## 10.7 Publishing the build log

- **Assembled from the journal**, not written from scratch: intent and mood board make the introduction; decisions make the "design story"; steps with photos make the instructions; outcome and as-built make the results; the cut list, BOM and templates are attached.
- **Optional AI rewrite** (Mind on, conclusion only): it rewrites for flow in the **user's own voice**, using their own entries as the style sample. It is shown as a diff over the raw log, the raw entries stay, and every rewritten paragraph is ⚡-marked until accepted.
- **Formats:** single-file HTML (portable, offline), PDF, Markdown (for blogs and Git), and an **Instructables-style** step structure ready to paste.
- **Privacy pass before publish:** strip EXIF GPS, flag photos with faces or house numbers for review, and exclude private notes (a per-entry flag).
- **Publishing destinations** are exports only by default. Direct upload integrations, if ever added, are opt-in plug-ins.
- **"Build from this"**: a published `.maker` share bundle opened by someone else re-runs the material fit against *their* inventory and *their* tools, and re-plans the steps for *their* capabilities. This is the walkthrough's "digital version… take your personal inventory and tool list into account."

## 10.8 Acceptance

| Criterion | Test |
|---|---|
| Golden dresser: ≥ 15 derived steps in a valid order (no assemble-before-cut, no blocked screw) | `xtask corpus --plan dresser` |
| Plan respects work windows and 24 h glue cure; the critical path is reported | Same |
| Tiled PDF test square measures 50.0 mm ± 0.3 mm when printed at 100% on a reference printer | Physical QA (STATUS row) |
| Projector calibration: a 100 mm square projects within ± 1 mm at the table corners | Physical QA |
| A person who never saw the project builds the bench-scale golden project from the HTML guide alone | Usability test |
