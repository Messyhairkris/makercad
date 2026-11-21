# Rules for every change (people, Claude and the Crew)

These override anything else in the repo. They come from what went wrong in MakerCAD v1–v5 and
what worked in Idea Animator.

## 1. No claim without a command and a number

A feature is done when STATUS.md has a row: the claim, the exact command, the result, the
machine. "It should work" is not a row. Measure on the reference machine (i5-6600 / HD 530 /
8 GB) when the claim is about speed or memory.

## 2. Never crash

Non-test code has no `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`,
`unimplemented!` and no `unsafe`. Return `Result`; use `?`, `ok_or`, `let … else`. Indices and
arithmetic on anything that came from a file, a person or a plug-in use `get()` and checked or
saturating math. A crash loses someone's work; every crash fix gets a regression test.

## 3. Brand-free, clean room

- SolidCraft's `ui-kit`, `look` and `theme` crates, its `ui-egui` shell and its name, logo and
  icons are never a dependency of anything here (`pwsh tools/brand_free.ps1` checks
  `cargo tree`; it becomes `xtask brand-free` in P1).
- No code is copied from GPL, LGPL or AGPL projects (lamina, most slicers). Their ideas may be
  described and re-implemented; their code may not be read while writing ours.
- Reference screenshots of other programs live in `reference/` (gitignored). Never commit them,
  never trace them, never ship them.

## 4. Every asset is ours or licensed, and listed

Icons, fonts, images and models carry a sidecar `<file>.attribution` (author, source, licence,
how it was made) and appear in `ATTRIBUTION.md`. Icons follow doc 18 exactly.

## 5. Everything is a command

A feature exists as a command with an id, a JSON schema and a doc line before it has a button.
The UI, the CLI, Sancho, the journal and MCP all call the same command.

## 6. Modules ship no UI code

A module declares commands, schemas and slots; the shell draws them (doc 16 §16.3).

## 7. Compatibility rules C1–C8 (doc 16 §16.4)

Released WIT files are frozen; old modules keep running; commands are never silently renamed;
files never lose sections they don't understand; migrations are one tested step at a time;
ids are never reused.

## 8. Crew output never lands directly

Crew modules arrive in `ATLAS-api-harness/runs/crew/outbox/workspace/`. Claude vendors them on a
branch, runs the gates, and Kris approves the commit on the Review Desk.

## 9. Gates before every commit

`cargo xtask ci` (from P1): fmt, clippy `-D warnings`, tests, layers, brand-free, wasm.
Until P1 lands, the crate's own `cargo test` and `cargo clippy -- -D warnings`. The cloud build
(`.github/workflows/build.yml`, decision D28) runs fmt, brand-free, clippy and the tests on every
push and pull request; a red cloud build blocks the next commit to `main`.

## 11. The repository is public

Nothing personal goes in: no user names in paths (write `C:\Users\<you>` or `$env:USERPROFILE`),
no machine or share names, no email addresses, no keys or tokens. Other programs' screenshots stay
in `reference/` (rule 3).

## 10. Look at the UI you change

Take a screenshot of the real app (`cargo run -p mk-desktop --bin makercad -- --demo
--size 1440x900 --screenshot home.png [--look night-workshop]`: it draws the window, writes the
PNG and closes) and put the picture in the PR. A UI change nobody has looked at is not finished.

## Tools

| Tool | What it does |
|---|---|
| `tools/fetch_solidcraft.ps1` | fetches the pinned SolidCraft rev into `third_party/solidcraft` |
| `tools/p0_build.ps1` | builds stock SolidCraft, logging to `target/p0/`; `-LowMemory` for 8 GB machines (one crate at a time, solidcore at opt-level 1) |
| `tools/p0_watch.ps1` | logs a running build's progress (crate count, rustc memory and CPU) every few minutes to `target/p0/watch.log` |
| `tools/p0_measure.py` | starts SolidCraft with its sample part and measures start time, adapter, UI time per frame and idle memory over its control channel; `--spin N` captures SolidCraft's own frame-rate readout during N seconds of spin (the real FPS) |
| `tools/p0_cli.ps1` | builds SolidCraft's headless CLI together with the app, so no shared crate is rebuilt |
| `tools/p0_oracle.py` | replays SolidCraft's reference parts (`tests/oracle`) one by one with the CLI and times each |
| `tools/lowmem_build.ps1` | builds a package one crate at a time on an 8 GB machine, logging to `target/lowmem/<package>/` (watch with `p0_watch.ps1 -Dir`) |
| `tools/reuse_check.py` | starts a build and kills it before it recompiles solidcore or the engine, printing the `rustc` line so the cause can be found |
| `tools/blender/inspect_blend.py`, `inspect_rig.py` | list a `.blend`'s objects, armatures, deform bones, meshes, pose library and controls (run Blender with `--disable-autoexec`) |
| `tools/blender/export_bodykun.py` | rebuilds BodyKun (CC0) as Sancho's placeholder glTF: 71 deform bones, one wood body, 1.68 m, Wave and Grip |
| `tools/blender/sancho_page.template.html` | the phone page for Sancho (`@@GLB@@` base64, `@@SIZE@@`); `#check=CLIP@SECONDS[,hand]` pins a pose for screenshots |
| `tools/brand_free.ps1` | fails if a SolidCraft brand crate is in our dependency graph |
| `tools/package_windows.ps1` | zips the release `makercad.exe` with READ-ME-FIRST.txt, the licences and ATTRIBUTION.md into `dist/` (the cloud build's test build) |
| `tools/icon_lint.py` | doc 18's SVG rules for an icon-set module (also the Crew's hidden tests) |
| `tools/icon_sheet.py` | a contact sheet of icon sets at 16, 32 and 64 px for a person to judge |
| `tools/keep_awake.ps1` | keeps Windows from sleeping until given files exist (long builds, campaigns) |
| `tools/commons_cad.py` | polite downloader for Wikimedia Commons CAD screenshots into `reference/` |
