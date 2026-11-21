<!-- Unified by Fable (claude-fable-5-2) on 2026-11-12 from plan.md, STATUS.md,
     DECISIONS.md and ATTRIBUTION.md. Reviewed and filed by Gordon.
     Open inconsistencies are listed in runs/crew/special_teams/UNIFY_NOTES.md. -->

# MakerCAD v5 - unified design

2026-11-12. Unified from the plan of record (its section 11 supersedes anything earlier in it), docs/STATUS.md, docs/DECISIONS.md and docs/ATTRIBUTION.md. Workspace: `C:\Users\<you>\MakerCAD\makercad_v5\`. `makercad_v4\` is frozen reference: read it, never build it.

## A. What it is

MakerCAD v5 is a Rust desktop CAD program that keeps OpenCASCADE 7.9.3 as its B-rep kernel behind a single crate. It is for a hobby maker on a low-end Windows machine (reference: i5-6600, HD 530, no Vulkan, 8 GB) who is often working alone. It takes that person from an idea to a built object: design, convert to flat parts that can be cut, check that they assemble, nest them on owned stock, and write the plan. Two things set it apart: a project log that grows with the design and ends as an Instructables-like article, and Sancho, a helper character who lives in the design space.

Not in scope: sweep/loft parity, assemblies with joints, FEA, cloud accounts, a component marketplace, sheet metal proper (folding is the dieline module).

## B. Architecture

### B1. Layers

Dependencies point down only. The gate is a command, not a convention.

| Layer | Crates | Role |
|---|---|---|
| L0 | `mc-geom`, `mc-id` | Maths, planes, 2D polygons (boolean, offset, area), meshes, units; stable ids, content hashes, schema version |
| L1 | `mc-sheet` | Laminate model: layer stack, regions, thickness, kerf, grain; display mesh, cut list, four construction techniques |
| L1 | `mc-sketch` | Entities, shape generator, snaps, dimensions, constraint store |
| L1 | `mc-solver` | planegcs behind its own DLL; reachable only through this crate |
| L1 | `mc-kernel` | The OCCT wall (B2) |
| L1 | `mc-fold` | Dieline templates, hinge graph, fold kinematics, flat pattern, unfold |
| L2 | `mc-doc` | Parameters, unit-aware expressions, feature timeline, incremental rebuild, `broken_reason` |
| L3 | `mc-io` | `.makercad` save/load and migrations; STEP, STL, 3MF, OBJ, DXF, SVG, PDF |
| L3 | `mc-log` | The project log (C1) |
| L3 | `mc-assist` | Sancho (C2) |
| L4 | `mc-engine` | Session, command registry, undo/redo, selection, module loader |
| L5 | `mc-view` (wgpu and CPU backends) | Swappable renderer |
| L5 | `mc-ui`, `mc-cli`, `mc-mcp` | egui desktop shell, headless CLI, MCP server: three front ends, one engine |
| shared | `mc-ui-kit`, `mc-theme`, `mc-text` | Used by hosts; unreachable from lower layers |

Rules that hold across layers:

- **Every action is a command with a schema** in one registry. UI, CLI, MCP server, log and Sancho all call the same commands.
- **Two modelling truths.** `Sheet` (2D layers with thickness; cheap; the maker default) and `Brep` (OCCT; real 3D parts, imports, fillets). The user picks per body; the toolbox hides which is in use.
- **Units.** Stored as f64 millimetres; the Maker persona sees inches by default.
- **Rebuild.** Each feature hashes its inputs; a change dirties that feature and those after it. History can be frozen past about 100 operations.
- **Honest failure.** A feature that cannot build sets `broken_reason` (human), `broken_detail` (technical) and `fix_suggestion`, shows red in the timeline, and its children keep their ids.
- **Naming.** B-rep selections use `SelectionIntent` (geometric signature, 0.85 confidence, graceful degrade). Sheet regions are named by layer and region id, stable by construction.
- **Preview is push/undo of the real document.** During a drag the display drops to a bounding proxy. There is no second model.

### B2. The one OCCT wall

- `mc-kernel` is the only crate that names OCCT (proven: STATUS, M1 part 1).
- The bridge is about 40 verbs taking and returning plain data: ids, f64 arrays, enums, strings. No OCCT type appears in a Rust signature.
- Every call goes through `trait Kernel`, so an out-of-process sidecar can be added later without touching the engine. It is not built now.
- Bad input is refused before the kernel is touched (`BadDimension`). Without OCCT the workspace still builds, and the kernel test fails with a sentence naming the path.
- Picking resolves in Rust (BVH over the display mesh), then asks the kernel for the canonical face or edge by ray.
- Licence: LGPL-2.1 plus the OCCT exception, dynamic DLLs, user-swappable. planegcs follows the same model in its own DLL.

### B3. What loads on demand

| Thing | Loads when | If absent |
|---|---|---|
| OCCT DLL set | First Brep operation; a sheet-only session never loads it (design, not yet proven) | Brep commands unavailable |
| `mod-package` (dieline, fold, artwork) | First use of its commands | Its commands are absent from the toolbox, not broken |
| `mod-draft` (drawing sheet, hatches, plot, PDF) | First use | Same; no PDF export |
| `mod-origami` (flat-foldability, layer order) | First use | Same |
| `mod-render` (offline pretty pictures) | First use, and only offered if the profile allows | Same |
| Sancho's 3D node | Profile allows it | 2D portrait |
| Sancho's Mind (AI) | User enables it | Rules only, zero network |

The four `mod-*` are separate `cdylib`s; everything else is compiled in. `settings.toml` can pin modules on or off.

**Capability probe.** At first run, command `mc.system.probe`:

- tries DX12, then GL, then CPU, and records what actually initialises;
- runs a 2-second, 200k-triangle benchmark on the real viewport;
- reads RAM, core count and disk read speed of the OCCT DLL set;
- writes a profile (`potato`, `modest` or `capable`) that sets renderer backend, triangle budget, LOD aggressiveness, 3D Sancho or portrait, and whether `mod-render` is offered.

"Check again" in settings re-runs it and shows what changed. It runs headless from the CLI. It never blocks startup: a failed probe means `potato`.

### B4. The renderer floor

- `mc-view` exposes only a camera, a draw list of (mesh, material, transform, pick-id), 2D screen overlays and a render call. Nothing in the UX depends on which backend draws it.
- Default backend: wgpu on DX12, with wgpu's GL backend as fallback.
- **Floor: a CPU rasterizer for the same draw list.** It is slow and always correct. If the GPU path fails on the HD 530, the product loses shading, not function.
- Level of detail is by screen-space error. Frame budget: <= 200k visible triangles, <= 60 draw calls, no post-processing by default.

### B5. Targets and what is measured

A target is not a claim until STATUS holds a command and a number for it.

| Target (reference machine) | Command | Measured |
|---|---|---|
| Kernel box 60x40x20 is exact | `cargo run -q -p mc-kernel --example kernel_report` | 48000 mm^3, 8800 mm^2 |
| Layer gate reads the workspace | `cargo run -q -p mc-cli -- layers` | 49 dependency edges checked |
| Cold start < 3 s with no Brep body | none yet | unmeasured |
| Orbit 30 fps | none yet | unmeasured |
| RSS < 600 MB; idle < 120 MB; sheet-only < 250 MB | none yet | unmeasured |
| 20-operation history rebuilds in < 1 s | none yet | unmeasured |
| Commit-to-exact p95 < 400 ms on the bracket | none yet | unmeasured |
| >= 80% of maker-path commands make no OCCT call | none yet (engine command log) | unmeasured |
| Rules answer >= 60% of findings with no model call | none yet | unmeasured |
| Any command reachable in <= 2 keystrokes | none yet | unmeasured |

## C. The two differentiators

### C1. The project log (`mc-log`)

The log is a second view of the document, stored in the `.makercad` file beside the timeline. It is not an exporter.

**Records**

| Record | Fields | Source |
|---|---|---|
| intent | goal, space, constraints, budget, skill, photos | Creation prompt; editable for the life of the project |
| decisions | id, at_feature, question, options, chosen, why, photo | The "Why?" button, with the options on screen |
| steps | id, from_commands, title, body, photo, tips, warnings, time_est | Command log templates |
| materials | name, kind, sheet, thickness, grain, stock_size, qty, cost | User and prompts |
| tools | name, kind, required_for | Derived |
| cuts | cut list | Derived from the sheet model |
| outcome | photos, what_worked, what_didnt, revisions | Close and conclusion prompts |

**How entries arrive**

1. Every command in the registry carries a log template ("Extrude {profile} by {distance}"), so the technical history writes itself.
2. Prompts fire at the moments below. The set is data in `crates/mc-log/prompts.toml`, not code, and will change after real use.
3. Derived content is computed, never typed: cut list, bill of materials with costs, tool list, step order from feature dependencies, time estimate, risk callouts.
4. An overridden Sancho finding is recorded as a log entry.

**Prompt schedule v1**

| Moment | Questions | If skipped |
|---|---|---|
| Project creation, before any tool appears | What are you making? Where will it live, what must it fit? What do you have to work with? | Toolbox stays locked: the linter needs the text |
| First material chosen | What is it, and its real dimensions? Why this one? | Design budget cannot be checked |
| First part fitted to the space | Does it fit, measured how? Anything you learned? | Fit is unverified |
| Any design change after a material was picked | What changed and why? | The most valuable entry is lost |
| First physical test-fit | Photo. What surprised you? | Log has no reality check |
| Close, or save with changes | What did you do? What went well? What needs changing? | Project cannot be marked complete |
| Conclusion | Final photos. Anything a builder should know? Then the optional AI rewrite | No publishable article |

**Rules**

1. The creation description is the first entry and stays editable. The distance between the first idea and the final solution is the point of the log.
2. Skipping is allowed and visible. A project with any skipped prompt shows **Incomplete** in the project list and in the export header. It is never silently "done".
3. The AI rewrite is optional and runs only at conclusion. The style sample is the user's own earlier entries, not a house voice. It is a suggestion over the log: raw entries stay and the diff is visible.
4. Export uses the same pipeline as every other format: Markdown, single-file HTML with images inlined, PDF (via `mod-draft`), and a share bundle (design, log, cut list). There is no second exporter.
5. Step thumbnails come from the CPU rasterizer at 512 px, generated when idle.

**Acceptance (M4):** the finished bracket exports an HTML log with >= 10 steps and a cut list; `mc-log::render` matches its golden files byte for byte; a person who never saw the project can build from the HTML.

### C2. Sancho (`mc-assist`)

Named after Sancho Panza: the companion on the road, there so the user is not alone with the project. He is part Clippy, part tamagotchi, part alert system. He has three layers, each able to be absent:

| Layer | What it does | Cost | Failure |
|---|---|---|---|
| Rules (`mc-assist::rules`) | Pure function: project state in, findings out. Offline, deterministic, tested | none | none |
| Voice (`mc-assist::persona`) | Wording, tone, timing, mood, relationship state (time since last session, past successes) | none | can only annoy; has a mute |
| Mind (`mc-assist::ai`) | Optional AI: answers questions, proposes parameters, calls MCP tools | money, latency | degrades to rules only |

**The contract**

1. **Persona is data.** `crates/mc-assist/personas/sancho.toml` holds name, portrait, tone, idle clip and praise thresholds. Core code never contains the name. The user can rename, re-skin or replace him.
2. **Every finding has** a kind, `severity`, `why`, `fix`, and a link to the command that fixes it.
3. **Kinds:**
   - `forbidden`: physically will not work (a 40 mm screw through 18 mm stock).
   - `denied`: breaks a rule the user or project set (hand tools only, but the step needs a table saw).
   - `not_allowed`: wrong for this material or process (laser-cutting PVC).
   - `tip`: useful, not urgent.
   - `praise`: a milestone completed.
4. **Assembly checks are rules, not AI guesses.** Before cutting, every part is checked as a physical object: too thin, severed by a slot, held by nothing, larger than the sheet. Most carry a one-click fix. The check list is the concept taken from lamina; the geometry tests are ours.
5. **He speaks only when:** (a) there is a forbidden, denied or not_allowed finding; (b) he is asked; (c) a milestone is reached; (d) the user turned on coach mode. There is no idle chatter by default. Tips wait in the panel unless coach mode is on.
6. **Silencing is one click and total.** `Silence`: voice off, findings still show in the panel. `Mute`: nothing. `Off`: node removed, a settings toggle. The choice is remembered per project and globally. A muted Sancho never animates and costs no frame.
7. **He never blocks.** Findings are advisory; the user can proceed, and the log records the override. Exception: true safety items (PVC under a laser, mains wiring) need one explicit confirm.
8. **The Mind acts only through the command registry**, the same commands `mc-mcp` exposes. It never touches geometry directly.
9. **He is honest about AI.** Mind answers are marked. When the Mind is off or unreachable he says so in one short line, once, and continues on rules. Mind off means zero network.
10. **Presence.** He stands in the scene at true scale (a 5'6" figure beside the desk is the dimension line), under 8k triangles, with prebaked clips and no lip-sync. The probe may select a 2D portrait instead.

**Acceptance (M4):** on the golden project Sancho flags 5 seeded forbidden conditions with 0 false flags; mute costs 0 fps; rules cover >= 60% of findings with no model call.

## D. Build order

Every exit criterion is run cold on the reference machine and entered in STATUS with its command.

| M | Delivers | Exit criterion | State |
|---|---|---|---|
| M0 Skeleton | Workspace, layer gate, window with grid, orbit and ground barrier, CLI stub, capability probe | Window opens in < 3 s; idle RSS < 120 MB; layer gate fails on a planted bad edge; `cargo test --workspace` green | Gate and tests proven; window, start time, RSS unmeasured |
| M1 First solid | Kernel bridge with box, extrude, cut; timeline and rebuild; STL out | 20-operation history rebuilds in < 1 s; STEP to STL round trip; volume matches analytic to 1e-6 | Box measured exact; rest unmeasured |
| M2 Sketch | Sketch tools, snaps, dimensions, top-down sketch mode, command line | Rectangle, circle, arc, slot, polygon from clicks give closed profiles; ported `sketch_shapes` cases pass; a new user builds the bracket (sketch, extrude, fillet, hole) in < 10 min | Not started |
| M3 Sheet + fold | Laminate model, cut list, the five construction techniques, `mod-package` with FEFCO 0201 and fold slider, DXF and SVG out | Box template gives correct panel count and crease lengths; fold 0-100% with 0 panel penetrations; DXF opens in a plotter workflow; technique criteria not set in the inputs | Not started |
| M4 Differentiators | `mc-log` end to end; `mc-assist` rules (including assembly checks), persona, optional Mind; `mc-mcp` | The C1 and C2 acceptance lines | Not started |
| M5 Handoff | This document plus `walkthrough.md` | A contributor continues from one document with 0 decisions reopened | This document |

Risk sits in M1 (OCCT integration), M2 (UX) and M4 (product).

The five techniques, each a plan generator over any body in the document:

| Technique | Makes | Module |
|---|---|---|
| Stacked | Parallel slices held by dowels, pegs or spacers | `mc-sheet::stacked` |
| Interlocked | Two slotted families, egg-crate | `mc-sheet::interlocked` |
| Radial | Half-slices fanned around an axis, locked by rings | `mc-sheet::radial` |
| Curve | Ribs that follow a bend | `mc-sheet::curve` |
| Folded | The surface unfolded flat; papercraft nets, 13 joint types | `mc-fold::unfold` |

Unfold doctrine (decided):

- Build a spanning tree of the mesh's dual graph.
- Attach a neighbour only if the union's area equals the sum of the areas.
- Strategies are `flat`, `strip`, `area`, `auto`; prefer fewest panels, then most compact.
- Simplify the mesh before unfolding.
- Number seam joints on both panels and engrave the joint type.

## E. Start here

**Run first, in this order**

1. Set `MC_OCCT_DIR` to the OCCT 7.9.3 install (reference: `C:\OpenCASCADE\opencascade-7.9.3-vc14-64`).
2. `cargo test --workspace` - expect every crate to pass.
3. `cargo run -q -p mc-cli -- layers` - expect `layers ok`.
4. `cargo run -q -p mc-kernel --example kernel_report` - expect `linked: true`, version 7.9.3, and the box at 48000 mm^3 and 8800 mm^2.

If step 4 fails with a sentence naming a path, OCCT was not found; that failure is deliberate.

**Read, in this order**

1. This document.
2. `docs/STATUS.md` - what is proven, with the command. Nothing enters it without a cold run on the reference machine.
3. `docs/DECISIONS.md` - 14 decisions, each with what would reverse it.
4. `docs/ATTRIBUTION.md` - before copying anything from anywhere. Lamina is AGPL-3.0: re-implement from its described concepts; never copy, vendor or link it.

**The three rules the codebase is built to keep**

1. **Layers point down.** The gate fails the build on an upward edge. Fix the edge, never the gate.
2. **One wall.** Only `mc-kernel` names OCCT; only plain data crosses.
3. **Everything is a command.** If the UI can do it, the CLI, the MCP server, the log and Sancho do it through the same registry entry.

## F. What could go wrong

| # | Risk | Early signal |
|---|---|---|
| 1 | The renderer becomes the project again, as it did in v4 | A week passes with no window time or fps in STATUS; workaround code appears in `mc-view` instead of a switch to the CPU floor |
| 2 | OCCT weight defeats the potato: 83 OCCT and 332 third-party DLLs sit beside the binary, and deferred load is unproven | The first cold-start or RSS measurement misses its target with no Brep body in the session |
| 3 | Breadth before depth: techniques, fold, origami, drafting and nesting pull effort from M1 and M2 | Commits land in `mod-*` or technique modules while STATUS gains no new M1 or M2 rows |
| 4 | The differentiators annoy: Sancho gives false flags, or Incomplete reads as punishment | Any false flag on the golden project; testers leave Sancho Off; most test projects end Incomplete |
| 5 | Licence contamination from lamina | A contribution whose structure or naming tracks lamina's source; an ATTRIBUTION row that lags the code |
