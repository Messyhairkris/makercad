# MakerCAD v5 — decisions

One row per decision, with the reason and the thing that would reverse it.

| Date | Decision | Why | Reversed if |
|---|---|---|---|
| 2026-11-12 | Rust, not Godot / C++ GDExtension | v4's five rendering attempts all failed; the engine was the risk, not the UX | never (PM direction) |
| 2026-11-12 | OpenCASCADE 7.9.3 stays the B-rep kernel | PM direction; accuracy and the STEP path are already paid for | a kernel that matches its oracle and is smaller |
| 2026-11-12 | Down-only layers, enforced by `mc layers` | one gate replaces a hundred conventions; it caught a real omission on its first run | never |
| 2026-11-12 | Two modelling truths: sheet + B-rep | maker work is sheet goods; 2D booleans are cheap, B-rep is not | if sheet modelling loses to B-rep on the bracket benchmark |
| 2026-11-12 | Renderer is swappable; CPU raster is the floor | "works on a potato" must be literal, not hopeful | never; only the default backend changes |
| 2026-11-12 | Capability probe at install sets the profile | the program must adapt to the machine, not the reverse | if the probe mis-classifies the reference machine |
| 2026-11-12 | Every action is a command with a schema | UI, CLI, MCP and the log call one path; three bug classes die | never |
| 2026-11-12 | The log is a view of the document, not an exporter | reconstructing what you did is the hard part of publishing | never |
| 2026-11-12 | A skipped log prompt leaves the project **Incomplete** | the PM: completing the design is what the program is for | if users report it as punitive rather than honest |
| 2026-11-12 | Helper = rules (free) + voice (free) + mind (optional) | help must survive no network and no budget | if rules cannot reach 60% of findings |
| 2026-11-12 | Sancho's name and persona live in a data file | a user must be able to rename, re-skin or remove him | never |
| 2026-11-12 | Lamina: concepts only, no code | it is AGPL-3.0; linking it would force our whole program to AGPL | a PM decision to licence the whole product AGPL |
| 2026-11-12 | Unfold via spanning tree, overlap tested on union area | documented doctrine; an intersection test misses fans past 360° at saddle vertices | if a case is found that the union test also misses |
| 2026-11-12 | Fable approval expires 2026-11-16 | cost we cannot afford after the window closes | PM extends it in writing |
