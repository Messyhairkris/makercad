# MakerCAD — The Program We Should Have Built

**New Research set, 2026-11-19.** This is a clean-sheet design for MakerCAD built on six open-source Rust projects. Earlier MakerCAD versions are used only as a list of lessons, not as a base. Where an older idea is kept, the doc says where it came from and why it survived.

## The design in one paragraph

MakerCAD is a **fork of SolidCraft**, a Rust parametric CAD program like Fusion 360 that runs on its own pure-Rust kernel, solidcore. On top of it we put a **Project layer** that SolidCraft does not have: intent, inventory (tools, materials and salvage), space, deadline, inspiration, budget and log. The CAD side is split into **Environments**: Solid, Sketch, Quick Build, Space and Scan, Stock and Cut, Fold, Soft Goods, Motion, Plan, Output and more. Each environment is a module the project activates when it needs it and unloads when it doesn't. The sketcher is enriched with **CADCraft/DraftCraft** drafting tools. **SketchCraft** supplies the push/pull "Quick Build" way of modelling. **PlanCraft's** core crate runs the schedule. **SurfaceCraft** (on the same solidcore kernel) turns scans into editable geometry. **Sancho**, a companion character shown at true human scale, connects all of it. He is rules first, personality second, and an optional AI third. Every action is a command, so the UI, Sancho, the build log and outside AI agents (MCP) all drive the same engine.

## Reading order

| # | Document | What it decides |
|---|---|---|
| 01 | [Vision and Principles](01_Vision_and_Principles.md) | Who it is for, what makes it different, the ten principles, and what we keep from v1–v5 and what we drop |
| 02 | [Source Projects Assessment](02_Source_Projects_Assessment.md) | Each of the six repos: stack, licence, maturity, what we take and how. Includes the brand-licence finding |
| 03 | [Architecture](03_Architecture.md) | The fork, crate map, layers, kernel decision, command registry, `.maker` project file, module system |
| 04 | [Project Model and Inventory](04_Project_Model_and_Inventory.md) | Data model for project, intent, tools, materials, salvage, space, deadline, mood board |
| 05 | [New Project Flow](05_New_Project_Flow.md) | Opening screen, the question flow, the interpreter, scale selection, first five minutes |
| 06 | [Environments](06_Environments.md) | Full catalogue: purpose, source, cost, commands, minimum viable version, load rules |
| 07 | [Sancho](07_Sancho.md) | The companion: layers, scale behaviour, findings, session-end interview, AI boundary |
| 08 | [Materials, Stock and Waste](08_Materials_Stock_and_Waste.md) | Material-aware design, cut optimisation, "make it fit what I have", showing waste |
| 09 | [Scan to Space](09_Scan_to_Space.md) | Splats, point clouds, meshes, RoomPlan; cleaning; edge and plane detection; room shell; as-built compare |
| 10 | [Plan, Build and Publish](10_Plan_Build_and_Publish.md) | Build order, schedule, session log, export pack, projector templates, build log publishing |
| 11 | [Look and Feel](11_Look_and_Feel.md) | The dreamy, cartoony theme and pastel option, layout, workspace behaviour, accessibility |
| 12 | [Performance and Hardware](12_Performance_and_Hardware.md) | Capability probe, profiles, budgets, the computer as a tool in the inventory |
| 13 | [Scenario Walkthroughs](13_Scenario_Walkthroughs.md) | Dresser in an odd alcove, product package, activity backpack, fold-away bedroom, battery cover |
| 14 | [Roadmap](14_Roadmap.md) | Phased build from fork to the full vision, with exit criteria, plus what to salvage from v5 |
| 15 | [Decisions, Risks and Open Questions](15_Decisions_Risks_Open_Questions.md) | Decision table with reversal triggers, risk register, questions only the PM can answer |
| 16 | [Modules, Compatibility and Growth](16_Modules_Compatibility_and_Growth.md) | **PM answers of 2026-11-19 (evening)**: licence, modules, connectors, Craft bridges, the compatibility contract, phones, the Module Forge, Sancho v1. Supersedes 03/06 where they differ |
| 17 | [Delivery System](17_Delivery_System.md) | Who does what (Claude, the Crew, Kris, Mookie, Buddy), the keeper, atomic doc updates |
| 18 | [Design Prompt](18_Design_Prompt.md) | The preamble and task prompts every icon, panel and illustration is made from, and the gates that accept them |
| — | [ui-v1/](ui-v1/) | First UI pass (5 artboards), source of the shared design canvas |

## The six decisions everything else rests on

> **Update 2026-11-19 (evening):** decision 1 now reads "use SolidCraft's open crates at a
> pinned commit; don't rebuild them, and don't fork" (doc 16, D25). Decision 3 is confirmed and
> larger than estimated: SolidCraft's whole `ui-egui` shell depends on its brand crates.

1. **Fork SolidCraft; don't rebuild it.** Its layering, command registry, MCP server, timeline, sheet metal, assemblies and wasm build already match what v5 was trying to become. Its maturity: about 10k commits, and 75 of 76 reference parts match.
2. **Drop OpenCASCADE and adopt solidcore.** The B-rep kernel is pure, safe Rust. That removes the 415-DLL payload measured in v5, which is the biggest single cost on a low-end machine. A `Kernel` trait keeps an OCCT sidecar possible later if exchange fidelity requires it.
3. **Build our own UI kit, theme and icons.** SolidCraft's `ui-kit`, `look` and `theme` crates are under LICENSE-BRAND and cannot ship in a fork without written permission. This is also the chance to do the pastel, storybook look on purpose.
4. **The Project is the root object, not the Design.** A `.maker` file holds the project (intent, inventory snapshot, space, schedule, log, media), and one or more designs inside it.
5. **Environments are modules with a lifecycle** (dormant, then loaded, then active), declared by the project, probed against the hardware, and unloadable. Built-in environments are compiled in but cost nothing while dormant. Heavy or third-party environments are WASM components or sidecar processes. We do not use Rust `cdylib` plugins.
6. **Inventory is a constraint, not a catalogue.** Every body can be bound to a stock item: a board, a sheet, a fabric bolt or a salvaged part. Material use, waste and "you're 2 boards short" stay live while you model.

Plus one PM direction from 2026-11-19: **Sancho is a rigged 3D humanoid** with Mixamo- or Rigify-style bones in glTF. He uses GPU skinning with a CPU path, his root bone fixes his true height, he has light look-at and point-at IK and crossfaded clips, he is tiered by hardware profile, and users can bring their own model (doc 07 §7.3.1).

## About REAForge

The request asked for REAForge. This session found no REAForge skill, and the `rea` MCP server timed out on connect. This set was written with standard research, planning and design methods instead. If REAForge holds a specific template or scoring method, re-run with it connected, and the docs can be reshaped to that template. Section headings are stable on purpose to make that easy.

## Sources consulted

- https://github.com/tangency-labs/solidcraft (README, docs/architecture.md, docs/layers.md, LICENSE-BRAND, crates/)
- https://github.com/bherbruck/solidcore (kernel; read-only mirror of SolidCraft's `solidcore/`)
- https://github.com/jasonsutter87/sketchcraft
- https://github.com/storytold/cadcraft (README, architecture, licence)
- https://github.com/DraftCraft-DEV/DraftCraft
- https://github.com/rallegretti/PlanCraft (README, docs/ARCHITECTURE.md)
- https://github.com/d0pawlus/SurfaceCraft
- Local: `makercad_v5/docs/DESIGN.md`, `STATUS.md`, `DECISIONS.md`, `makercad is.txt`; `MakerCAD/Product_Walkthrough_Maker_Use_Case.md`; `MakerCAD_Human_Centered_Professional_CAD_Workspace_UX_Specification.md` (section list); `makercad_v4/MakerCAD_Universal_Geometry_Program_Proposal.md` (section list).

Facts about the repos (licences, crate names, stats) are as read on 2026-11-19. They change quickly. Re-check anything load-bearing before acting on it, especially licence files.
