# 15 — Decisions, Risks and Open Questions

## 15.1 Decisions

Same format as v5 DECISIONS: each decision has its reason and the thing that would reverse it. Status is **Proposed** until the PM accepts.

| # | Decision | Why | Reversed if | Status |
|---|---|---|---|---|
| D1 | MakerCAD is a **fork of SolidCraft** | Its architecture is what v5 was building, ~100× further along; Fusion-like workflow already present | P0 shows it can't run acceptably on the potato, or upstream changes licence | Proposed |
| D2 | **solidcore** replaces OCCT | Pure Rust, no DLL payload, wasm, zero integration cost in the fork | > 10% of the maker corpus or of STEP imports fail after 2 upstream cycles → add the OCCT exchange sidecar (not a full switch) | Proposed |
| D3 | **Our own UI kit, theme and icons** | SolidCraft LICENSE-BRAND forbids redistributing `ui-kit`/`look`/`theme` in a fork | Tangency Labs grants written permission (we'd still want our own look) | Proposed (forced by licence) |
| D4 | **Project** (`.maker`) is the root file; designs are SolidCraft `.sc` + extension sidecar | The program is project management around CAD; keeps designs openable in SolidCraft | Upstream adds a document extension store, so the sidecar folds in | Proposed |
| D5 | Environments = **compiled-in, initialise on demand**; external = **WASM components or sidecars**; **no cdylib** | RAM cost is data, not code; Rust has no stable ABI | Measurements show dormant built-ins cost > 50 MB total | Proposed |
| D6 | **Inventory is a constraint, not a gate** (P1) | Context sparks creativity; locking frustrates | Never | Proposed |
| D7 | Sketch enrichment by **porting** CADCraft/DraftCraft tools onto SolidCraft sketch entities; **one** sketch model and **one** solver | Two sketch models would double bugs | — | Proposed |
| D8 | **PlanCraft core** as the scheduling engine; tasks derived from design | CC0, pure logic crate, egui-compatible, rich model | It can't express elapsed-time lags or equipment calendars as needed (check in P5) | Proposed |
| D9 | SurfaceCraft pipeline vendored for scans (same kernel) | Native primitives without translation | — | Proposed |
| D10 | SketchCraft: **interaction model ported**, not its UI (Tauri/React mismatch) | Can't embed a web UI in egui without a webview, which costs RAM on the potato | — | Proposed |
| D11 | Sancho is a **rigged glTF humanoid** (Mixamo/Rigify naming), GPU skinning with a CPU path, root-height true scale, light IK, crossfades only, tiered by profile; BYO Sancho | PM direction 2026-11-19 | Potato tiers can't hit < 2 ms per frame → portrait default on potato | **Accepted (PM)** |
| D12 | Default Sancho asset is **authored in-house** (Blender + Rigify), not raw Mixamo | Clean redistribution rights | Legal confirms Mixamo assets may ship embedded | Proposed |
| D13 | The Quest **never locks** the toolbox (reverses v5's "toolbox stays locked") | Contradicted P1 | — | Proposed |
| D14 | Skipped log prompts are **open pages**, still flagged | Keeps honesty, removes punishment (v5 risk F4) | Testers still read it as punitive | Proposed |
| D15 | AI (the Mind) is optional, an MCP client with no privilege, ⚡-marked, zero network when off | Trust, cost, offline use | — | Proposed (carried from v5) |
| D16 | Status discipline: no claim without a command and a number on the potato | The hardest-won v4 lesson | Never | Carried |

## 15.2 Risks

| # | Risk | Likelihood | Impact | Early signal | Mitigation |
|---|---|---|---|---|---|
| R1 | **solidcore immaturity**: fillets and booleans fail on maker parts | Medium | High | Maker corpus failures in P0/P3 | Corpus in CI; upstream reports; joinery designed with simple geometry; OCCT exchange sidecar as the escape hatch |
| R2 | **Upstream churn**: SolidCraft moves fast; merges get painful | High | Medium | Merge time > 1 day per cycle | Thin fork; extension points upstreamed; patch count < 15 |
| R3 | **Brand-licence entanglement**: `ui-egui` depends deeply on `ui-kit` | Medium | Medium-high | P0 dependency map | Budget P1 for a full shell rewrite if needed; never copy brand-crate code (clean room) |
| R4 | **The potato GPU path** (Vulkan segfault, DX12 driver quirks on the HD 530) | High (known) | High | P0 | Child-process probe; DX12, then GL, then CPU; CPU raster floor tested in CI |
| R5 | **Breadth before depth** (v5 F3, again): 20 environments tempt | High | High | Commits in P6+ areas before P3 exits | Phase gate rule (doc 14); one environment per phase except marked ‖ tracks |
| R6 | **Inventory entry friction**: users won't catalogue | High | High | Quest step 4/5 skip rate > 50% | Kits, workshop reuse, photo intake, lazy precision; measure the skip rate in tests |
| R7 | **Sancho annoys** | Medium | Medium | Testers mute or switch off | Speaking rules, override-demotion, Silence ≠ Off, false-flag gate |
| R8 | **egui version skew** across vendored Craft crates | Medium | Low-medium | Build breaks on vendor bump | Vendor only non-UI crates from CADCraft/DraftCraft/PlanCraft/SurfaceCraft; our UI on SolidCraft's egui version |
| R9 | **Licence drift**: a vendored repo changes licence or adds GPL deps | Low | High | `cargo deny` licence check | Pin commits; `cargo deny` in CI; ATTRIBUTION table updated per vendor bump |
| R10 | **Scan accuracy over-promised** (splat scale, cheap LiDAR) | Medium | Medium | As-built deltas > 1 cm | Second-reference check; confidence display; "measure to confirm" prompts on critical dims |
| R11 | **Production outputs not production grade** (dielines, patterns) | Medium | High (for business users) | Print shop or plotter rejects | Physical QA per release; prepress checks; golden files |
| R12 | **Scope of the soft-goods flattening** (doubly curved, stretch) | Medium | Medium | Physical test misses ± 1 cm | Start with developable panels + darts; LSCM/ARAP with a distortion map; physical prototype loop |

## 15.3 Open questions for the PM

1. **REAForge:** what template or method should these docs follow? (The `rea` server didn't connect, and no skill by that name exists in this session.) Doc headings are stable, so a reshuffle is cheap.
2. **Licence of MakerCAD itself.** Code from the sources is MIT/Apache/CC0. Do we ship MakerCAD as MIT/Apache (most compatible), or something else? This affects the slicer sidecar choice and whether lamina-style AGPL concepts ever become code (they shouldn't).
3. **Upstream relationship.** Do we want to contribute to SolidCraft publicly (extension points, kernel bugs) under our names? It speeds everything up, and it makes MakerCAD visible to their community.
4. **Ask Tangency Labs about `ui-kit`?** Worth one polite issue, or go straight to our own kit?
5. **Product name and the "Craft" family.** MakerCAD keeps its name, so there's no confusion with ArtCraft trademarks. Confirm.
6. **The Mind provider.** Is there a default (cloud API or a local model)? Is there a monthly cost ceiling for development testing?
7. **The reference potato.** Is the i5-6600 / HD 530 still the floor, or should an even lower floor be considered (4 GB RAM, Celeron)? That would change the budgets in doc 12.
8. **v5 backup folder.** Is it OK to tag v5 as `v5-final` and delete `makercad_v5_backup_2026-11-13_pre-pass2` after?
9. **First audience.** Which golden project should be the "MVP story" for a first public preview: the dresser (wood, room, scan) or the battery cover (fast, small)? The roadmap assumes the dresser path (P3 before P6), but the battery cover could ship earlier as a demo after P5.
10. **Sancho's look.** Is the workshop-companion direction (flat cap, apron, tape measure) OK, or do you want a more literal Sancho Panza (squire, donkey nearby)? Concept sketches are needed either way before modelling.
