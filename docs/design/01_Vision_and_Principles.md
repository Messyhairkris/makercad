# 01 — Vision and Principles

## 1.1 What MakerCAD is

MakerCAD is a **project tool with a CAD core**. Most CAD programs assume the designer is one link in a long chain: someone else buys material, runs the machines, plans the schedule and writes the manual. MakerCAD assumes **the designer is the whole chain**. Usually that is one person, sometimes a few, with a specific room, a specific pile of stuff, a specific set of tools, a specific computer and a deadline.

So the program's job is wider than geometry. It helps that person:

1. **Say what they want** in their own words, and keep that intent visible for the whole project.
2. **Account for what they have**: tools, materials, salvage, space, money, time and computer.
3. **Design inside that reality**, using real dimensions from a tape, a photo or a scan, at the right scale, with live feedback on material use and waste.
4. **Plan the build**: order, cuts, templates, schedule and risks.
5. **Build it and record what happened**: session interviews, photos, video, and a scan of the result compared with the design.
6. **Publish it** as instructions someone else could follow, or as a private record.

The tagline we've been circling: **"Design with what you have."**

## 1.2 Who it is for

| Persona | Typical project | What they need most |
|---|---|---|
| **The renter-maker** | Furniture for an odd alcove; storage | Scan or measure the space; material use; plain-language help |
| **The small-business founder** | Product packaging, displays, jigs | Production-grade dielines; print and cut exports; repeatability |
| **The outdoor or soft-goods maker** | Backpack, bike bags, covers | 3D-to-pattern flattening; seam allowances; sewing order |
| **The space-constrained dreamer** | Fold-away bedroom, boat-style interiors | Motion and fold states; clearance; budget; staged build |
| **The fixer** | Replacement battery cover, knob or bracket | Fast measuring; photo scale; a quick 3D print or laser cut |
| **The organiser** | Drawer inserts, pegboards, shop storage | Quick Build; sheet nesting; reuse of offcuts |

Secondary (from the 2026-08 walkthrough): VFX and animators, game developers. They are served later through **workflow packs** (doc 06 §6.14), not by bending the core.

## 1.3 What makes it different

1. **Questions before tools.** A new project starts with a conversation, not an empty viewport (doc 05).
2. **Inventory is a design constraint.** "15 pine 2x4s at 3 ft" is a material budget the model is checked against live (doc 08).
3. **Real space first.** Scans and measurements become clean, editable reference geometry: walls, openings, frames, furniture (doc 09).
4. **Human scale is always visible.** Sancho stands in the scene at true size: a person beside a dresser, or a giant beside a bench-top part (doc 07).
5. **The build is part of the project.** Build order, cut templates, projector mode, schedule, session log, as-built comparison (doc 10).
6. **The computer is a tool in the inventory.** The program knows the hardware it runs on and adapts. It also tells you when a task needs more than you have (doc 12).
7. **Environments load as needed.** A pencil-sharpener-plus-box project loads Solid, Sketch, Fold and Motion, and nothing else (doc 06).

## 1.4 The ten principles

Each principle has a test, so it can be checked rather than argued.

| # | Principle | Test |
|---|---|---|
| P1 | **Context, not restriction.** Inventory and tools inform; they never forbid. | Every inventory warning has a "design anyway" path, and the log records the choice. |
| P2 | **Honest numbers.** Material use, waste, cost and time are computed, never guessed. Estimates say they are estimates. | Every displayed number traces to a command or to a user entry. |
| P3 | **Everything is a command.** UI, keyboard, Sancho, log, CLI and MCP all go through one registry. | No feature exists that the CLI cannot reproduce. |
| P4 | **Light by default, rich on request.** Nothing loads until the project needs it. | A sketch-only session never initialises the kernel's boolean engine, the scan stack or physics. |
| P5 | **The floor is the potato.** Works on the reference low-end machine (i5-6600, HD 530, 8 GB, no usable Vulkan). | Every release passes the reference-machine run; numbers go in STATUS. |
| P6 | **Forgiving surface, rigorous core.** Friendly tools and words on top; exact B-rep and solved constraints underneath. | Quick Build output is ordinary timeline features, editable in Solid. |
| P7 | **Intent is never lost.** The first description stays visible and editable, and every major change asks "why?" | The project's intent card is one click from anywhere; the log shows how the idea evolved. |
| P8 | **Production-grade output.** Exports are good enough to send to a die-cutter, a CNC shop, a print shop or a pattern printer. | Golden exports open cleanly in the standard downstream tools. |
| P9 | **Fail honestly.** A broken feature says why, in words, and offers a fix. A skipped prompt shows as incomplete. | No silent failure, no fake "done". |
| P10 | **Your data is yours.** Local-first files; AI is optional and labelled; nothing leaves the machine unless the user sends it. | With the AI switched off, the program makes zero network calls, and that is tested. |

## 1.5 What we keep from earlier versions and what we drop

The PM is right that v1–v5 should not be the foundation. They still taught real lessons. These are the ones that survive, each with its source.

### Keep: lessons with evidence behind them

| Lesson | From | Why it survives |
|---|---|---|
| **The capability probe and named graphics backends** | v5 STATUS 2026-11-12: Vulkan **segfaults** on the HD 530 and wgpu did not fall back | A measured failure on the target machine. SolidCraft is also wgpu; it must be checked on that machine before anything else. |
| **CPU raster as the rendering floor** | v4: five failed renderer attempts; v5 B4 | SolidCraft's `viewport` crate already has a CPU rasteriser; we make it the guaranteed fallback. |
| **Status discipline: no claim without a command and a number** | v4 → v5 STATUS | It cost the most to learn. Keep it. |
| **The project log is a view of the document, not an exporter** | v5 C1 | Reconstructing what you did afterwards is the hard part of publishing. |
| **Sancho's contract: rules, then voice, then optional mind; persona as data; never blocks; one-click silence** | v5 C2 | Good design, independent of the codebase. |
| **Lamina-style construction techniques: stacked, interlocked, radial, curve-rib, folded** | v5 D | Concepts only; lamina is AGPL. We re-implement from description. |
| **Unfold doctrine: spanning tree of the dual graph, union-area overlap test** | v5 decisions | Correct and documented. |
| **The linter-to-toolbox idea** (description words load tools) | 2026-08 walkthrough | Central to "questions before tools". |
| **Splat measurement probe, environment calibration, design budget, fit/clearance as first-class features** | UX specification §26–28, §50, §52 | Good UX thinking that never got built. |
| **Source / canonical / display separation; staged import; partial success is a feature** | v4 Universal Geometry proposal §8, §4.2, §30 | Right for scans and big imports. |

### Drop

| What | Why |
|---|---|
| Godot / GDExtension (v4) | The renderer became the project. |
| OpenCASCADE as the primary kernel (v5) | 83 OCCT plus 332 third-party DLLs; C++ shim, global mutex, thread-unsafe. solidcore removes all of it. |
| Our own sketch solver wrapper (planegcs DLL) | SolidCraft's solver plus the CADCraft constraint work covers it. |
| `cdylib` modules (v5 B3) | Rust has no stable ABI; a cdylib plugin breaks on every compiler bump. We use WASM components or sidecar processes instead. |
| Our own document and timeline (`mc-doc`) | SolidCraft's `doc` crate is further along. |
| The fixed "Sheet vs Brep" split | It becomes a property: a body can be **stock-bound** (doc 08) whatever its geometry type. |

### Salvage from v5 code

Little code carries over cleanly. Worth porting as code or tests: the camera clamp tests (`mc-view`), the layer-gate test cases (as extra cases for SolidCraft's `xtask layers`), the `sketch_shapes` test cases, and the dieline FEFCO 0201 template definitions once they exist. Everything else is reference only. Doc 14 §14.6 has the full list.

## 1.6 North star

> A first-time maker opens MakerCAD, describes the odd alcove in their bedroom, scans it with their phone, says they have fifteen 3-ft 2x4s, a jigsaw and a drill, and needs it in three weeks. Forty minutes later they are looking at a dresser that fits the alcove to the millimetre, uses 13 of their boards with 9% waste, a cut diagram for each board, a build order for six evenings, and Sancho standing in the room next to it, slightly worried about the top drawer's runners.

Every document in this set should make that paragraph more true.
