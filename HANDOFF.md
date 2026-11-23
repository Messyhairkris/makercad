# MakerCAD: handoff at the pause (2026-11-23)

Written for Kris and for whoever (a person, or a new Claude session) picks this up next. It is
short on purpose: the long record is `STATUS.md` (one row per claim) and `docs/design/` (what we
decided and why). Read this first, then those.

## In one paragraph

MakerCAD is a CAD program for makers: you say what you want to make and what you have, and it works
out how to build it with that. A de-branded desktop window now exists, with an opening screen, a
nine-question "Start something new" that makes a project file, a workshop of the tools and materials
you own, tool days, a measuring system per project, command search, three looks, and a Design
workspace that shows SolidCraft's examples with Sancho beside them at true size. Under the window sit
eight tested library crates written mostly by the Crew (units, nesting, inventory, project file,
journal, rules, Sancho, hardware probe). **You cannot yet model anything in it**: the toolbox has only
view buttons. That is the biggest gap between what exists and what the program is for.

## What to do first

1. **Run it.** Download the newest `makercad-testNN-windows-x64.zip` from
   <https://github.com/Messyhairkris/makercad/releases> (an optimised build, about 88 MB), or run the
   debug build at `third_party\solidcraft\target\debug\makercad.exe` in the repo.
2. **Open the pause-point test pass**: `docs/test-passes/Test pass - MakerCAD pause point - 2026-11-23.html`
   (about 80 minutes, no rush, saves in the browser, Export gives text to paste back). It explains the
   process, shows every screen, walks you through the program, says what is real and what is a
   placeholder, and lists the decisions waiting for you. The older pass for the Quest, workshop and
   Findings (`...Quest workshop Findings...html`, 50 points) is the detailed one for those areas.
3. **FigCraft.** `docs/design/ui-v3/` has the fifteen real screens as PNGs with numbered questions;
   `docs/design/ui-v2/` has six "in use" screens as editable SVGs with 21 questions. Save your
   `.figcraft` file in `docs/design/ui-v3/notes/` or on the share. `python tools/figcraft_read.py FILE`
   reads it (tested only on a file built from FigCraft's own schema, never a real one: tell us if it
   fails).

## What exists, and where

| Area | Where | State |
|---|---|---|
| The window (shell, looks, opening screen, command search, status bar) | `apps/mk-desktop` | Works. 102 unit tests |
| The Design workspace | `apps/mk-desktop/src/design.rs` + pinned SolidCraft (`third_party/solidcraft`) | A **viewer** of SolidCraft's examples with Sancho; no modelling tools in the toolbox |
| Projects (`.maker` files, crash-safe saving, recents) | `crates/mk-project`, `apps/mk-desktop/src/project_file.rs` | Works |
| The Quest (nine questions) | `crates/mk-log` (the questions), `crates/mk-assist` (the interpreter), `quest_screen.rs` | Works; makes a project |
| My workshop, global/local items, tool days, the finish offer (D30, D31) | `crates/mk-inventory` (SQLite `library.db`), `workshop_screen.rs` | Works |
| A project's measuring system, typed lengths (D34) | `crates/mk-units/src/system.rs`, `quantity.rs` | Works in the Quest's size box; no variables panel yet |
| Sancho (scale figure) | `crates/mk-sancho` | A still figure beside the design (BodyKun placeholder, D26). Clips, light rig and persona exist as tested modules, **not in the window** |
| Rules and Findings | `crates/mk-assist` | The panel shows a **demo** dresser; the real design is not connected (`design_facts` exists, unwired) |
| Intent card, toolbox from answers, observations, voice, journal prompts, session interview | `mk-project`, `mk-assist`, `mk-log` | Tested modules, no screen |
| Stock & Cut, Plan, Build, Space, Share | doc 14 roadmap P3 to P8 | Not built; the tabs show a placeholder page |
| FigCraft files as UI (D29) | `tools/figcraft_read.py` | A reader only, never tried on a real file; decision still "Proposed" |

## Decisions in force (the last week)

D26 BodyKun placeholder, D27 Sancho is a scale figure first, D28 public repo and cloud build, D29
FigCraft files as UI (proposed), **D30** tools and materials are global or local, **D31** tools can be
there only some days, **D32** Sancho is never the channel for alerts (he reacts, like a game
character), **D33** tools are anything that modifies or refines a material, and the list stays small
and common, **D34** a project measures in one system and every typed length is turned into it. All in
`docs/design/16_Modules_Compatibility_and_Growth.md`.

## Waiting on Kris

The test pass's last section has the full list with numbers. In short: **what to build next**
(modelling tools, then the real Findings, then Stock & Cut, or a polish round from your FigCraft notes);
whether the next build waits for your FigCraft work; whether to keep every test build (31 so far, about
2.7 GB); whether a project knows it is finished; sourced material numbers or marked handbook values;
narrowing "your 41 tools" to the 9 a project needs; the measuring precision (1/64 inch); whether the
Findings panel keeps Sancho's name; whether measuring instruments count as tools and whether to add
pliers, a wrench and a pencil. Also:

- **GitHub still serves the old commits by their hash.** The Windows user name is gone from every
  branch and test tag (history rewritten on 2026-11-23 with Kris's yes; a fresh clone of what GitHub
  serves has none in 875 objects, nobody has forked or starred the repository). GitHub keeps commits
  nothing points at reachable by hash until it cleans up. To remove them for good, Kris asks GitHub
  Support to remove cached views and run garbage collection on `Messyhairkris/makercad`. Every test
  build's release notes still name the old short hashes (cosmetic). A backup of the old history sits
  in the session's scratch folder; it holds the name, so delete it once Kris is content.
- Deleting the old Crew build caches (`pa-work\cargo-target` and `pa-work\cargo-target-q`, in the home
  folder beside the repos) needs Kris's yes (disk is tight).
- The ATLAS harness's `crew/agent.py` has a parser bug (it ends a submitted file at the first inner
  code fence); the fix waits because Kris's checkout has uncommitted changes there.

## Known gaps and risks (honest list)

- No modelling tools; the Findings are a demo; a project does not remember that it is finished; a
  plan does nothing with a tool's days.
- Material numbers in the rules (pine 10,000 MPa, plywood 6,000 MPa, 2 mm sag per metre) are handbook
  values marked "not yet sourced".
- Performance: 51 to 57 FPS orbiting the sample plate and 1.1 to 1.7 s to the first frame, both in a
  **debug** build on the reference PC. The optimised build (from the cloud) is not timed. Doc 12's
  stricter dresser-and-room target is not tried. The GL backend crashed once in three starts during P0;
  MakerCAD's order on Windows is DX12 then GL, because SolidCraft's Vulkan-first order crashed here.
- `add_leftovers` (the finish offer) adds materials and then tools in separate steps: a failure between
  them could add the materials twice on a retry. Rare; not fixed.
- A metric length typed into an imperial project is nearly always rounded to 1/64 inch and says so.

## How to work on it

- **Layout.** `apps/mk-desktop` (the window), `crates/mk-*` (libraries), `third_party/solidcraft`
  (SolidCraft pinned at a commit, D25; no contact with its makers, D23), `tools/` (scripts),
  `docs/design/` (the design, numbered 00 to 18), `docs/test-passes/`, `docs/screens/`.
- **The rules** are in `AGENTS.md`: no claim without a command and a number; never crash; brand-free;
  every asset licensed and listed; everything is a command; Crew output never lands directly; gates
  before every commit; look at the UI you change; **the repository is public** (no user names in
  paths, no emails, no keys: write `C:\Users\<you>`). **In every new clone run
  `python tools/personal_scan.py --install`**: git then refuses a commit or push that carries personal
  data (`tools/personal_scan.py`, D35; the cloud build and Claude Code check too).
- **Build and test** (this PC has 8 GB and a hard disk, so one heavy job at a time):
  - the app: `cargo test -j 1 -p mk-desktop --lib` (about 10 to 15 minutes cold; the shared target is
    `third_party/solidcraft/target`);
  - a small crate: set `CARGO_TARGET_DIR` to a **short** path such as `C:\Users\<you>\mk-st`
    (rusqlite fails to build under a long path) and run `cargo test -p mk-units`;
  - **never** delete files from `target/debug/deps` by age: it caused an hours-long rebuild once;
  - after `cargo clippy --fix`, run `cargo fmt`, never before.
- **The gates** (the cloud runs all of them): `cargo fmt --check` (our crates), `tools/brand_free.ps1`,
  `python tools/layers.py`, the wasm check, `cargo clippy --workspace --all-targets --locked -- -D
  warnings`, and the tests. Cloud status needs no sign-in:
  `https://api.github.com/repos/Messyhairkris/makercad/actions/runs`.
- **Screenshots and tests passes.** `makercad --screenshot PATH` draws the window to a PNG (with
  `--quest N`, `--workshop N`, `--finish`, `--findings dresser`, `--sample`, `--typed`, `--when`).
  `pwsh tools/ui3_screens.ps1` takes all fifteen; `python tools/ui3_prepare.py` makes the repo and
  embedded copies; `python tools/test_pass/make_pause_pass.py` rebuilds the pause-point pass (it uses
  the `test-pass` skill's template).
- **Commits.** The repo's identity is the GitHub noreply address, never the personal one. Commits are
  approved while Kris is away; push after the gates pass locally.
- **The Crew** lives in the ATLAS harness repo (`ATLAS-api-harness`, next to this one). Default to
  delegating bounded, checkable work: Claude writes the contract, reference and hidden tests, the
  Crew writes the module, then mutation testing and the gates decide. `crew/runners/mk_pipeline/` has
  the pipeline and a README of what bit us (a brief must stay under about 29 KB; Windows' 32 KB
  command-line limit). Never trust crew output that a test it cannot read has not passed.
- **Windows and PowerShell.** `$args` is an automatic variable (do not use it as a parameter); a
  PowerShell here-string cannot feed `git commit -F -`, use `-m @'...'@`; `Wait-Process -Timeout` is
  how to wait on cargo (long foreground sleeps are blocked); egui text inputs must keep the raw typed
  string (collapsing spaces each frame eats them).

## Next-build options (from the test pass; the order is Kris's)

(A) **Modelling tools** in the toolbox (a button runs one of SolidCraft's engine commands: sketch,
pull up, round over). Medium-large. Needed before anyone can build anything here.
(B) **Findings on the real design**: parts carry a material from the project's list. Needs (A). Medium.
(C) **Stock & Cut (P3)**: the nesting crate (`mk-nest`) in the window, cut lists, the waste meter.
Needs (A) and (B). Large.
(D) **A plan from steps** using tool days (P5), shown on the plan, never by Sancho. Needs (A) and (C).
(E) **A polish round from Kris's FigCraft notes** on the screens that exist. Small to medium; needs the
notes first.
