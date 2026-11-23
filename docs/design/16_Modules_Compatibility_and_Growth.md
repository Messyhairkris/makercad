# 16 — Modules, Compatibility and Growth

**Added 2026-11-19 (evening), after the PM's answers.** This document supersedes the parts of docs
03 and 06 that treat environments as the only unit of extension. Everything else in 01–15 stands.

## 16.1 What the PM decided

| # | Direction | Where it lands |
|---|---|---|
| 1 | Free to use, open to **commercial and open-source** use | Licence **MIT OR Apache-2.0** (the same as every source project). D17 |
| 2 | MakerCAD **connects to manufacturers** (Shapeways and others) through **Modules** | Connector modules, §16.7 |
| 3 | A **Module** is a plug-in for a method or a system | Module kinds, §16.3 |
| 4 | **Do not contact** the SolidCraft maintainers. Learn from Idea Animator | Own shell, clean room; Idea Animator patterns, §16.10 |
| 5 | ArtCraft tools (FigCraft and others) should be **plug-ins, not rebuilds** | Craft bridge, §16.8 |
| 6 | Sancho starts as a **simple rigged model with fingers**, not a person | Doc 07 amendment, §16.11 |
| 7 | Features are built **when a project needs them**; AI helps build a missing module | Module Forge, §16.6 |
| 8 | **Small, agile, portable**, aiming at iOS and Android eventually | Portability, §16.5 |
| 9 | **Grow without losing compatibility** with future versions | The compatibility contract, §16.4 |
| 10 | Many hands: Claude, the Crew, and the humans Kris, Mookie and Buddy | Delivery system, doc 17 |

## 16.2 The shape: a small core, and everything else a module

```
┌──────────────────────────────────────────────────────────────────────────┐
│ CORE (always present, small)                                             │
│  project model (.maker) · command registry · undo · journal · inventory  │
│  SolidCraft document + kernel (solidcore) · module host · shell · Sancho │
├──────────────────────────────────────────────────────────────────────────┤
│ MODULES (each one optional, versioned, shareable)                        │
│  environments · methods · formats · connectors · Craft bridges ·         │
│  content packs (themes, icons, kits, materials, rules, questions)        │
└──────────────────────────────────────────────────────────────────────────┘
```

The core is what every project needs and nothing more. A module adds commands, document data,
Sancho rules, toolbox entries, questions or file formats, and it can be added and removed without
changing the core.

## 16.3 Module kinds

| Kind | What it adds | How it runs | Example |
|---|---|---|---|
| **Environment** | A way of working: tools, panels, overlays, data | Compiled in, dormant until a project loads it (doc 06) | Sketch, Solid, Stock & Cut, Fold |
| **Method** | One building technique and its tools | WASM component (`craft:plugin` + `maker:module`) | Pipe Frame (PVC and wood loom), box joints, papier-mâché shell |
| **Format** | Opens or writes a file type | WASM component, `formats@` capability | FigCraft layout import, `.skp` import, HPGL export |
| **Connector** | Sends work to a manufacturer or a machine | Sidecar process, or WASM with `maker:net` | Shapeways quote and order, a local laser shop, OctoPrint |
| **Craft bridge** | Uses another Craft app as a tool | Sidecar: that app's control channel or MCP | FigCraft, VectorCraft, MaterialCraft, GridCraft |
| **Content pack** | Data only, no code | Files in a package | Themes, icon packs, tool kits, material libraries, rule packs, question packs, Sancho personas |

One package format serves every kind (§16.9). A module never ships UI code. It declares commands
with JSON schemas and named slots, and MakerCAD draws the dialog, exactly as SolidCraft's plug-in
host does (`docs/plugins.md` upstream: "Plugins ship no UI code"). That rule keeps modules small,
safe, portable to phones, and consistent with the storybook look.

## 16.4 The compatibility contract

These rules let MakerCAD grow for years without breaking old projects or old modules. Each rule
has a test in CI. A rule without a test is a wish.

| # | Rule | Test |
|---|---|---|
| C1 | **Interfaces are versioned WIT packages.** MakerCAD never edits `craft:plugin`; it adds its own package `maker:module@0.x`. A new capability is a new interface, never a change to an old one | `xtask wit-frozen`: every released WIT file is byte-identical to its tag |
| C2 | **Old modules keep running.** The host serves every frozen WIT version it has ever released (SolidCraft already does this for `craft:plugin` 0.1 and 0.2) | A corpus of modules built against each released version runs in CI |
| C3 | **Commands are data.** Every command has an id, a JSON schema and a version. A renamed command keeps its old id as an alias for two major versions, and `engine.describe` says so | `xtask commands-diff` against the last release: removed ids and narrowed schemas fail |
| C4 | **Files never lose data.** A `.maker` file keeps sections it does not understand byte for byte, including module data, when the module is missing (SolidCraft's plug-in storage rule) | Round-trip a file with a fake future section; bytes are identical |
| C5 | **Migrations go forward, one step at a time.** Each format version has one tested migration to the next. A file from a newer version opens read-only with a plain message | `testdata/formats/v<N>/` holds samples from every released version; all of them open |
| C6 | **Module data carries its own format number** (`data_format`). An older module reads newer data read-only; a newer module migrates older data in one undo step | Corpus test per first-party module |
| C7 | **IDs are never reused**: command ids, material taxonomy ids, tool capability ids, rule ids, environment ids | `xtask ids` keeps a registry of retired ids |
| C8 | **The core stays below its budget.** Startup, idle memory and binary size are measured on the potato each release (doc 12) | `mk-cli bench --profile potato` |

**Version numbers.** The app, the `.maker` format, `maker:module` and each module have separate
version numbers. While an interface is `0.x`, a minor version may break the one before it, *but the
host still runs the old one* (C2). `1.0` is declared only after a real module has lived through two
releases.

## 16.5 Portability: desktop now, phones later

| Layer | Desktop (Windows, macOS, Linux) | Browser | Android | iOS |
|---|---|---|---|---|
| Core crates (no UI) | yes | yes (wasm32, CI-checked) | yes | yes |
| Kernel (solidcore) | yes | yes | yes | yes |
| Module runtime | wasmtime (fast compiler) | browser's own WASM engine | wasmtime | **interpreter only**: iOS forbids generating code at run time, so the host needs wasmtime's interpreter or `wasmi` |
| Shell (egui) | yes | yes | yes (winit) | partial; needs work |

**Plan for phones:** the first phone app is a **companion**, not the full CAD program. It handles
scan capture, photos with a scale reference, inventory intake (photograph the pegboard), the
build-mode step viewer and journal entries. It shares the core crates and the `.maker` format.
Full design on a tablet follows when the shell runs well there. Every rule above (no UI code in
modules, wasm32 core, data-driven commands) exists so this path stays open.

**Small and agile, in numbers** (to be measured, doc 12): installer under 60 MB with no bundled
scan stack; cold start under 2 s on the potato; idle memory under 150 MB.

## 16.6 The Module Forge: building a missing module when a project needs it

The PM's example: *a user wants to build a loom and has only some wood and PVC pipe.* No built-in
environment does pipe frames. The Forge builds one, to the user's need, in the same form as every
other module, so the next person with PVC pipe can use it.

**The flow**

1. **Notice the gap.** The interpreter or Sancho sees a method with no module ("PVC pipe frame").
   It says so once: "There's no pipe-frame tool yet. Want me to make one for this project?"
2. **Write the spec, with the user.** A short form: what the parts are (pipes: diameter, length;
   fittings: tee, elbow, cross; wood braces), what a user does (place a pipe, join with a fitting,
   drill a hole, add a brace), what must be checked (pipe length within stock, fitting exists for
   the angle, hole edge distance), and what comes out (cut list, fittings list, drilling template).
   The spec is data: commands with JSON schemas, rules with test cases, a material list.
3. **Build it.** The Mind (or, during development, the Crew) writes the module in Rust against
   `craft:plugin` + `maker:module`, compiled to a WASM component.
4. **Gate it.** The same gates as everything else: `plugin check` (manifest, permissions, WIT),
   `plugin test` with the test cases from the spec, plus hidden cases the author never saw. A
   module that fails is not installed.
5. **Preview, then install.** The user sees what the module does to *their* design in a preview
   (SolidCraft's host already previews plug-in commands on a scratch copy), then installs it for
   this project or for their workshop.
6. **Share it** (optional). The package goes to the shared store (§16.9) with its spec, tests and
   source, so anyone can read what it does before installing it.

**What the loom needs from the Pipe Frame module:** commands `pipe.place`, `pipe.join`
(fitting chosen from inventory), `pipe.drill`, `brace.add` (a wood member bound to stock),
`frame.check` (rules), and an export of the cut list and fittings list. All of it is ordinary
geometry in the document, so Solid, Stock & Cut and Plan work on it with no special cases.

**Guard rails:** a forged module gets no files, no network and no clock unless the user grants
them; it is labelled "made for this project" until it passes the shared-store review; and its
commands are ⚡-marked in history like every other AI-originated change (doc 07 §7.8).

## 16.7 Connector modules: "make it for me"

MakerCAD stays local-first (P10): **nothing leaves the machine unless the user sends it.** A
connector is the only kind of module that may use the network, and only like this:

- It declares the hosts it talks to (`api.shapeways.com`, for example) in its manifest. Nothing
  else is reachable.
- Every send shows the user what will be sent (files, dimensions, material, quantity, address
  fields) and waits for a click. No silent uploads, no background sync.
- Accounts and keys live in the operating system's credential store, never in the `.maker` file.
- Quotes come back as journal records (price, lead time, material, the service's own terms link).
- An order is never placed by MakerCAD without the user confirming the exact total on screen.

First connectors, in order of maker value: a **generic "send to a shop" pack** (a zip of the
export pack plus a printable order sheet, no network at all), **OctoPrint/Moonraker** (a printer
on the local network), then **Shapeways-type 3D printing services**, **laser and CNC shops**
(SendCutSend-type sheet services), and **print shops** for dielines. Each service's API and terms
are checked when its module is built. Nothing about any service is assumed here.

The `craft:plugin` host refuses the `net` permission today. MakerCAD adds `maker:net@0.1` as its
own interface, with the rules above enforced by the host, not by the module.

## 16.8 Craft bridges: ArtCraft tools as plug-ins, not rebuilds

The ArtCraft organisation publishes about twenty clean-room Craft apps under Apache-2.0
(checked 2026-11-19): FigCraft (interface design), VectorCraft (vector art), CADCraft (drafting),
MaterialCraft (node materials), TextureCraft (texture painting), GridCraft (spreadsheets),
DeckCraft (slides), PdfCraft, PhotoCraft and others. They share three things MakerCAD can use
without rebuilding any of them:

1. **Everything is a command**, reachable from a control channel and an MCP server.
2. **Common file formats**: SVG, DXF, PDF, PNG, CSV.
3. **A shared extension contract in progress**, `craft-extensions` (§16.9).

A **Craft bridge** is a sidecar module that starts (or finds) the other app, drives it through its
command protocol, and exchanges files. MakerCAD never links the other app's code, so there are no
brand-licence questions and no egui version clashes.

| Bridge | What MakerCAD uses it for |
|---|---|
| FigCraft | Read the UI layouts the PM designs (the first use, §16.12); mood boards |
| VectorCraft | Packaging artwork mapped onto dielines; laser engraving art |
| CADCraft | Heavy 2D drafting and DWG work beyond MakerCAD's Drawings environment |
| GridCraft | Cut lists, BOMs and budgets as live spreadsheets |
| MaterialCraft / TextureCraft | Materials and textures for the storybook render |
| DeckCraft | Presenting a published build log |

**File first, protocol second.** The first version of every bridge is file exchange, which works
even when the other app is not installed on this machine. Live control comes next.

## 16.9 One package format, shared with the Craft suite

- **Code modules** use SolidCraft's app-agnostic plug-in crates, `craft-plugin` and
  `craft-plugin-wasm` (MIT OR Apache-2.0, deliberately free of any SolidCraft dependency). A
  MakerCAD module that uses only `commands@0.2` also runs in SolidCraft, and every SolidCraft
  plug-in runs in MakerCAD. That two-way compatibility is free, so we keep it.
- **Packaging and installing** follows `craft-extensions` (the suite's shared store contract:
  "install once, enable per app"; themes, plug-ins and resources). It is provisional upstream, so
  MakerCAD reads its manifest format now and adopts its store once the contract settles. Until then
  a MakerCAD package is a `craft:plugin` zip with an extra `maker` section in `plugin.json`, which a
  host that doesn't know MakerCAD simply ignores.
- **Themes** use the `craft-extensions` theme palette and contrast check, so a MakerCAD theme is a
  Craft theme.

## 16.10 What we take from Idea Animator

Idea Animator (an in-house fork of EffectCraft) already solved problems MakerCAD v4 and v5 kept
hitting:

| Problem | Idea Animator's answer | MakerCAD rule |
|---|---|---|
| The default graphics path crashes on the HD 530 | `setup/launch.ps1` starts the app, waits for a responding window, falls back to OpenGL and **remembers** it in `%LOCALAPPDATA%` (`-Retest` to forget) | Same launcher pattern in front of the probe (doc 12) |
| On OpenGL, resizing at start-up misbehaves | Maximize a second after the window is up | Same |
| Agents can't see the UI they change | A **headless snapshot** example renders the UI to a PNG, driven by control-channel steps | `mk-desktop --snapshot` from the first shell commit |
| Brand marks in a fork | Removed, with a NOTICE that says so; own logo under the project licence | Same; plus SolidCraft's brand crates are never in our dependency graph |
| Proprietary reference screenshots | Kept local and gitignored, never committed | `reference/` is gitignored |
| Crashes lose work | "Never crash" rule: no `unwrap`/`expect`/`panic!` outside tests | Same, in `AGENTS.md` |

## 16.11 Sancho, first version (amends doc 07 §7.3.1)

Sancho v1 is **a simple rigged figure with fingers**: a mannequin-like body (capsule limbs, a
round head with two eyes for direction) with a full hand rig (three joints per finger, a thumb)
so he can point, count, hold a board and give a thumbs-up. Mixamo-compatible bone names from day
one, so a later, more characterful model drops in through the persona file. The figure is
**generated by code** (a crate that writes the glTF), so it has no licence questions and every
proportion is a parameter (height is the root bone, doc 05 §5.5).

### 16.11.1 Sancho's order of work (PM, 2026-11-20; D27)

Kris, on seeing the placeholder move: Sancho is **a feature, not the program's intent**. His first
and lasting job is **scale**: a measuring stick the user reads at a glance. Height for a dresser,
hand and finger lengths for a bracket or a knob.

- **Now (P1):** a **still, posable figure stands beside every design**, as SketchUp puts a person
  in each new model. He sits outside the design (not in the parts list, never picked, never
  saved as geometry). He is at true size, can be hidden, and is posed by code rather than animated.
- **The figure is the user's choice**: the placeholder, Kris's own design when it is ready, or any
  humanoid model at any height. That includes a **3D scan of the user, rigged**, so the
  measuring stick is their own body. Doc 07's "bring your own Sancho" covers the file side.
- **Later, one role at a time, as projects need them:** reach envelopes, eye height, wearing or
  carrying the design, pointing at findings (doc 07 §7.3).
- **Movement comes last**, as the polish working with the planning tools: clips from **motion
  capture** rather than hand-keyed, and reactions to the project the way a game character reacts
  to its world. Until then the placeholder's hand-keyed Wave stays a rig test only.

This reorders doc 14: P2's "Sancho rigged model v1" becomes a still, posable figure plus height
and hand measurements. Skinned animation, IK and clips move to the end of the plan.

## 16.12 The next steps this enables

1. **First UI design** (a page Kris, Mookie and Buddy can look at), then the PM's FigCraft layout
   file. The FigCraft bridge's first job is to read that file into our own layout description.
2. **Crew-built icons and features**, all from one design prompt (`docs/design/18_Design_Prompt.md`)
   so they stay consistent.
3. **The Pipe Frame method module** as the first Forge test, because it is small, real and not
   covered by any environment.

## 16.13 Decisions added

| # | Decision | Why | Reversed if | Status |
|---|---|---|---|---|
| D17 | Licence **MIT OR Apache-2.0** | Commercial and open use; matches every source project | — | **Accepted (PM)** |
| D18 | **Small core, everything else a module**; modules ship no UI code | Small, portable, consistent look | A module kind proves impossible without its own UI → add a `ui` capability as data (forms, draw lists) | Proposed |
| D19 | Code modules use **`craft:plugin`** (SolidCraft's app-agnostic crates) plus our own **`maker:module`** package | Two-way plug-in compatibility with SolidCraft for free | Upstream licence change on those crates | Proposed |
| D20 | Packaging follows **`craft-extensions`** once it settles | Shared store with the whole Craft suite | It stalls for 6 months → ship our own store format, keep a converter | Proposed |
| D21 | Network only in **connector modules**, through `maker:net` with per-send consent | P10, trust | — | Proposed |
| D22 | ArtCraft tools join as **Craft bridges** (sidecar, files first) | Use, don't rebuild; no egui or brand clashes | — | **Accepted (PM)** |
| D23 | **No contact** with SolidCraft maintainers; consume their open crates at a pinned commit; own shell | PM direction; the brand licence covers their UI code | PM changes direction | **Accepted (PM)** |
| D24 | Sancho v1 is a **code-generated rigged mannequin with finger bones** | PM direction; no asset licence questions | — | Superseded by D26 (the code-built figure stays as the fallback) |
| D25 | SolidCraft is consumed as **path dependencies into a pinned shallow checkout** (`third_party/solidcraft`, rev in `third_party/SOLIDCRAFT_REV`), not a full fork | Brand crates never enter our graph; an upstream update is one rev change; no merge work. Cost found in P1: Cargo hashes path crates by their path from the workspace root, so SolidCraft's own build is never reused; our first build compiles its crates once (about 5 h on the potato) | We need to carry more than 15 patches → become a fork | **Accepted (PM, 2026-11-20: "go on with the fork")** |
| D26 | Until Kris designs Sancho, the **placeholder is BodyKun** from "BodyChan – BodyKun, Posable Drawing Figures" by vinchau (BlendSwap 23521, **CC0**), converted to glTF with Blender; the code-built mannequin (`mk-sancho`) is the fallback | PM direction 2026-11-20; a nicer figure than the code-built one, licence-free | Kris's own model arrives → it replaces the placeholder (one asset file) | **Accepted (PM)** |
| D29 | **A `.figcraft` file can be an app's UI.** MakerCAD reads FigCraft's native files directly (a zip of `document.json` and images: pages, frames, auto layout, components, text, comments, prototype links). First as a reader so Claude can read the PM's notes and decisions; then as a Craft bridge module that turns frames and components into MakerCAD's layout description (layout as data, D18) and prototype connections into navigation routes, shared with other ArtCraft apps through `craft-extensions` | Kris, 2026-11-21: "a possible plugin for future ArtCraft programs that allow UI to be created as a native figcraft file for direct use as UI and routing information" | FigCraft's format proves unstable between versions → read its REST-API JSON export instead | Proposed |
| D28 | MakerCAD lives in a **public GitHub repository** (`Messyhairkris/makercad`). A **cloud build** (GitHub Actions, `.github/workflows/build.yml`) runs the gates and builds an optimised Windows `makercad.exe` on every push to `main`, published as a test build on the Releases page | PM direction 2026-11-20 ("set it up", public). The reference PC needs hours for SolidCraft's crates and cannot make optimised builds; testers need a download without a GitHub account; public repos build for free | Costs or a licence question → private repo with invited testers | **Accepted (PM)** |
| D31 | **A tool can be there only some days, and a finished project offers its own tools too.** Each tool (the workshop's, and each one a project lists) carries **when it can be used**: every day unless said otherwise, or chosen days of the week, with a free note for what days cannot say ("when my brother visits"). A multimeter that is only there on weekends is still a tool the person has; what changes is when a step that needs it can happen. The days are kept as a set so a plan can ask "how long until I can use it?" (`Availability::wait_days`); the note is only ever shown to the person. A workshop tool's days are changed in My workshop; a project's own tool has its own days, set in the project's Quest. `project/inventory.json` tool entries carry `"when": {"days": ["sat","sun"], "note": "..."}` only when there is something to say, so every older file still reads as every day. When a project is finished, its leftover **local tools** are offered with its leftover materials, each with how the person has it now (default "I own it") and the days it came with; a kind the workshop already has is not added twice. The workshop file `library.db` moves to schema version 2 (two columns on tools); a version 1 file is brought up when opened and loses nothing | PM direction 2026-11-23: "tools might have conditions attached to them, i.e. the user might have access to a multimeter on weekends" | A condition a set of weekdays cannot carry (a date range, "after the 15th", another person's schedule) → a richer condition. Not built: a plan that places a step needing a limited tool onto its days | **Accepted (PM)** |
| D30 | **Tools and materials are global or local.** *Global* ones are the person's workshop (`library.db`): kept up to date in their own walkthrough ("My workshop"), reused by every project. *Local* ones belong to one project only (a material bought for the task) and stay out of the workshop. Anything typed into a new project's Quest is local. When a project is finished, its **leftover local materials are offered** to the workshop, with how many are left, and each one added becomes global. A project's `inventory.json` is a snapshot in which every entry says which it is | PM direction 2026-11-23: "materials are applicable to either global (all use) and local (only the current project)... remaining materials from a local project have the opportunity at the end of the project to add to the global options" | A person wants something else from a finished project to join the workshop (local tools: done, D31) | **Accepted (PM)** |
| D27 | Sancho is a **scale figure first**: a still, posable, user-chosen figure (any height; a rigged scan of the user welcome) **stands beside every design** from P1. Animation, motion-capture clips and reactions come **last**, as polish with the planning tools (§16.11.1) | PM direction 2026-11-20: "Sancho is a feature, not the intent of the program"; the hand-keyed wave read as rough | — | **Accepted (PM)** |
