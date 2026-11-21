# MakerCAD

**Design with what you have.** A project tool with a CAD core, for people who design *and*
build the thing themselves: with their own tools, their own pile of materials, their own room,
their own computer and a deadline.

Status: **P1, the MakerCAD shell around SolidCraft's kernel**, started 2026-11-19. Nothing here
is a product yet. Every claim about what works lives in [STATUS.md](STATUS.md) with the command
that proves it.

## Test builds

Every push to `main` is built in the cloud ([.github/workflows/build.yml](.github/workflows/build.yml))
and published as a **test build** for Windows x64 on the
[Releases page](https://github.com/Messyhairkris/makercad/releases): download the zip, unzip,
run `makercad.exe`. Its `READ-ME-FIRST.txt` says what to try. The program is not signed yet, so
Windows asks first ("More info", then "Run anyway").

## What it is built from

| Piece | Where it comes from | How |
|---|---|---|
| Exact B-rep kernel (solidcore), document, timeline, sketch solver, commands, viewport, MCP | [SolidCraft](https://github.com/tangency-labs/solidcraft), MIT OR Apache-2.0 | Path dependencies into a pinned checkout in `third_party/solidcraft` (rev in `third_party/SOLIDCRAFT_REV`). Its brand-licensed crates (`ui-kit`, `look`, `theme`) and its UI shell (`ui-egui`) are **never** in our dependency graph |
| Plug-in contract (`craft:plugin` WIT) | SolidCraft's app-agnostic `craft-plugin` crates | Shared, so modules work in both programs |
| Drafting tools, DXF/DWG | CADCraft / DraftCraft | Ported onto SolidCraft's sketch |
| Scheduling | PlanCraft core (CC0) | Dependency |
| Scan to geometry | SurfaceCraft | Vendored pipeline |
| Push/pull "Quick Build" | SketchCraft's interaction model | Re-implemented |
| Everything that makes it MakerCAD | this repo: `crates/mk-*`, `apps/mk-*` | Our own code, MIT OR Apache-2.0 |

## Read first

- [docs/design/00_README.md](docs/design/00_README.md): the design set (vision, architecture,
  environments, Sancho, materials, roadmap).
- [docs/design/16_Modules_Compatibility_and_Growth.md](docs/design/16_Modules_Compatibility_and_Growth.md):
  the PM's decisions of 2026-11-19, the module system and the compatibility contract.
- [docs/design/17_Delivery_System.md](docs/design/17_Delivery_System.md): how work flows
  between Claude, the Crew and the people.
- [docs/design/18_Design_Prompt.md](docs/design/18_Design_Prompt.md): the one prompt every icon
  and panel is drawn from.
- [AGENTS.md](AGENTS.md): rules for every change, by anyone.

## Layout

```
apps/            mk-desktop (the `makercad` program: opening screen, Design view); mk-cli, mk-mcp in P1
crates/          mk-units (lengths as makers type them), mk-nest (boards → cut list),
                 mk-inventory (quick-add: "15 pine 2x4s, 3 foot"), mk-sancho (Sancho v1 as glTF)
docs/crew/       reviews of Crew campaigns before their code lands here
docs/design/     the design set, UI pass 1 in ui-v1/
docs/history/    what earlier versions (v1-v5) wrote down, kept for their lessons
tools/           scripts: the P0 build, reference gathering
third_party/     pinned SolidCraft checkout          (gitignored, fetched)
reference/       other programs' screenshots and docs (gitignored, never committed)
```

## Licence

MIT OR Apache-2.0, at your option. Third-party notices in [ATTRIBUTION.md](ATTRIBUTION.md).

## Build and run it

```
pwsh tools/fetch_solidcraft.ps1                          # SolidCraft at the pinned rev, once
cargo run -p mk-desktop --bin makercad -- --demo         # the opening screen with sample projects
cargo run -p mk-desktop --bin makercad -- --sample       # the Design view: SolidCraft's sample plate, Sancho beside it
cargo run -p mk-desktop --bin makercad -- --demo --screenshot home.png   # draw, save a PNG, close
```

`--backend dx12|vulkan|gl` forces a graphics backend; `--retest` forgets the one that worked;
`--look night-workshop` picks a look; `--fit` frames the sample plate without Sancho.
The first build compiles all of SolidCraft's crates: about an hour on a 16 GB machine, several
on an 8 GB one with a hard disk (use `pwsh tools/lowmem_build.ps1 -Package mk-desktop` there).
Later builds compile only MakerCAD's own crates.
