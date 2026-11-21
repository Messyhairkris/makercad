# 03 — Architecture

## 3.1 Shape of the system

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ FRONT ENDS (L5)   mk-desktop (egui shell)   mk-cli   mk-mcp   [mk-web later]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ ENVIRONMENT HOST (L4.5)  mk-env-host: registry, lifecycle, budgets, toolbox    │
│   ├─ built-in envs (compiled in, dormant until activated)                    │
│   │    solid · sketch · quickbuild · space · stock · fold · softgoods ·      │
│   │    motion · plan · drawings · output · journal                           │
│   └─ external envs:  WASM components (craft-plugin WIT)  |  sidecar processes │
├──────────────────────────────────────────────────────────────────────────────┤
│ ENGINE (L4)   SolidCraft engine + mk-project session: command registry, undo,  │
│               selection, ⚡ change-source marking, command guard              │
├──────────────────────────────────────────────────────────────────────────────┤
│ PROJECT (L3)  mk-project (.maker container) · mk-inventory · mk-log ·          │
│               mk-assist (Sancho rules/persona/mind) · io (+dxf/dwg/skp)        │
├──────────────────────────────────────────────────────────────────────────────┤
│ DOCUMENT (L2) SolidCraft doc: parameters, expressions, timeline, incremental   │
│               eval  + mk-stockbind (body↔stock), mk-plan (plancraft-core)     │
├──────────────────────────────────────────────────────────────────────────────┤
│ DOMAIN (L1)   sketch (+mk-sketch-tools) · kernel(solidcore) · dynamics ·       │
│               mk-nest · mk-fold · mk-pattern · mk-scan · render              │
├──────────────────────────────────────────────────────────────────────────────┤
│ BASE (L0)     geom · display-units · mk-units-maker (fractions, nominal lumber) │
├──────────────────────────────────────────────────────────────────────────────┤
│ SHARED        viewport (CPU raster) · viewport-gpu · viewport-egui · control · │
│               mcp-server · ui-registry · ui-text · mk-ui-kit · mk-look ·       │
│               mk-theme · mk-probe                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

The layer rules are SolidCraft's, enforced by its `cargo xtask layers`. We add our crates to its `TABLE`:

- Dependencies point down only. Shared crates depend only on shared crates, solidcore's public crates and external crates.
- Nothing below L5 reaches egui or wgpu.
- Every crate below the apps builds for wasm32. This keeps a browser MakerCAD possible, and it is also what lets environments run as WASM components.

## 3.2 Fork strategy: "thin fork, fat additions"

The SolidCraft repo moves fast. To keep pulling upstream fixes, especially kernel fixes:

1. **Fork at a tagged release.** Create the branch `makercad/main`. Track `upstream/main` and merge weekly at first, then per release.
2. **Never edit upstream crates unless there is no alternative.** All MakerCAD code lives in new `crates/mk-*` crates and `apps/mk-*` apps.
3. When an upstream crate must change, the change goes in a **patch series** (`patches/*.md` explains each one). Each patch is either upstreamed as a PR or tagged `carry`. The goal is fewer than 15 carried patches.
4. **Remove the brand crates** (`ui-kit`, `look`, `theme`) and the brand assets from our build. A CI check (`xtask brand-free`) fails the build if any `mk-*` crate or our app depends on them, or if a brand asset path appears in our bundle.
5. **Extension points we need upstream**, to be requested as PRs so we don't carry them:
   - command-registry *namespaces* that can be registered and unregistered at runtime (for environment unloading)
   - a document "extension store", so `mk-stockbind` and `mk-project` data can ride in the design file without patching `doc`
   - a viewport overlay hook (for Sancho, the space reference and the waste heat-map)
   - the toolbox/ribbon built from `ui-registry` data rather than a fixed layout

If upstream declines an extension point, carry it as a patch.

## 3.3 Repository layout

```
makercad/
  apps/
    mk-desktop/        egui app: shell, start screen, project wizard, environment switcher
    mk-cli/            headless: every command, plus `probe`, `export`, `bench`
    mk-mcp/            MCP server (SolidCraft mcp + mk namespaces + plan.*)
  crates/
    (upstream SolidCraft crates, untouched except for patches)
    mk-units-maker/    fractional inches, nominal lumber sizes (2x4 → 1.5x3.5), gauges, fabric weights
    mk-project/        .maker container, project session, intent, media store, migrations
    mk-inventory/      tools, materials, stock items, salvage, suppliers; local library DB
    mk-stockbind/      binds bodies to stock items; live consumption; grain; kerf
    mk-nest/           1D cutting-stock, guillotine 2D, rectangle pack; later true-shape
    mk-fold/           dieline model, crease/cut/perf, hinge graph, fold kinematics, unfold
    mk-pattern/        surface flattening (LSCM/ARAP), seam graph, allowances, notches
    mk-scan/           ingest (splat, points, mesh, RoomPlan), clean, detect, room semantics
                       (wraps vendored SurfaceCraft crates)
    mk-sketch-tools/   CADCraft/DraftCraft-derived modify tools, osnaps, grips, conic
    mk-plan/           wraps plancraft-core; design→task generator
    mk-log/            journal records, prompt schedule, session interviews, publisher
    mk-assist/         Sancho: rules, persona, mind adapter
    mk-env-host/       environment registry, lifecycle, budgets, toolbox builder
    mk-env-*/          one crate per built-in environment (UI + commands)
    mk-probe/          capability probe, hardware profile, "computer as a tool"
    mk-ui-kit/         our widgets (clean room)
    mk-look/           our icons and glyphs
    mk-theme/          tokens: Daydream, Pastel, Night Workshop, High Contrast
    mk-export/         export pack orchestration: blueprint, templates, projector, bundles
  vendor/
    cadcraft-geom/, cadcraft-dxf/, draftcraft-dwg/, draftcraft-constraints/  (pinned)
    surfacecraft-*/                                                          (pinned)
    sketchcraft-skp/, sketchcraft-cutlist/                                   (pinned)
  plugins/             first-party WASM environments (later)
  sidecars/            first-party sidecar environments (slicer, CFD)
  docs/  patches/  xtask/  testdata/maker-corpus/
```

## 3.4 Kernel decision: solidcore, with a door left open

| | OCCT 7.9.3 (v5) | solidcore (SolidCraft) |
|---|---|---|
| Language | C++ behind a C shim | Safe Rust, no `unsafe` |
| Payload | 83 OCCT + 332 third-party DLLs (measured in v5) | Compiled into the binary |
| Threading | Global mutex required (v5 bug #2) | Rust ownership; parallelisable |
| wasm | No | Yes (CI-checked) |
| Maturity | Decades | 0.0.x; 75/76 SolidCraft reference parts pass |
| Integration cost for us | We'd rewrite every SolidCraft feature against it | Zero: it is what the fork already uses |
| STEP fidelity | Excellent | SolidCraft's `io` does STEP; quality to be measured on our corpus |

**Decision: solidcore.** Choosing OCCT would mean rewriting SolidCraft's feature layer, which defeats the point of forking.

**The door left open:** `kernel` is already SolidCraft's boundary crate. If STEP exchange or a class of fillets fails our corpus badly, we add an **out-of-process OCCT sidecar** ("Precision Exchange" environment) for that one job: import a STEP, heal it, write it back as solidcore-readable B-rep or mesh. It is loaded only when needed, it is a separate process, and its LGPL obligations stay contained.

**Trigger for building it:** more than 10% of the maker corpus or of user-reported STEP imports fail after two upstream release cycles.

## 3.5 The command registry: one path for everything

SolidCraft's engine already makes every action a command, and MCP, the CLI and the UI share that registry. MakerCAD extends each command with metadata:

```rust
struct CommandMeta {
    id: &'static str,              // "stock.assign", "fold.add_crease", "sketch.trim"
    env: EnvId,                    // which environment owns it (for unload)
    schema: JsonSchema,            // args, for MCP + CLI + Sancho
    log_template: Option<&'static str>,   // "Cut {part} from {stock} ({length})"
    build_step: Option<StepKind>,  // does this become a physical build step? (doc 10)
    cost_hint: CostHint,           // Cheap | Kernel | Heavy  (probe budgets, doc 12)
    safety: Option<SafetyClass>,   // triggers Sancho's confirm rule
    change_source: ChangeSource,   // User | Sancho | Agent(⚡) | Import | Plugin
}
```

Engine-wide rules:

- **Command guard** (from DraftCraft): every command runs in a transaction. On error, the document is restored and the error becomes an honest message.
- **⚡ marking** (from PlanCraft): changes made by Sancho's Mind or by any MCP agent are marked in history and the log, and can be undone as one group.
- **Log templates** feed the journal automatically (doc 10).
- **Namespaces register and unregister with their environment**, so a dormant environment's commands are absent from search, the toolbox and MCP. They are not broken, just not there, and Sancho can say "that's in the Fold environment — load it?"

## 3.6 The `.maker` project file

A MakerCAD file is a **project**, not a part. It is a single zip-like container (store-only for media, deflate for JSON), written atomically: a temp file, then fsync, then rename, keeping one recovery copy (PlanCraft's pattern).

```
my-dresser.maker
  manifest.json              format version, app version, environments used, hash index
  project/intent.json        name, goal text, success criteria, deadline, budget, skill
  project/inventory.json     snapshot of tools/materials/salvage *as used by this project*
  project/space/             space definitions, scan refs, calibration, room semantics
  project/inspiration/       mood board items (images, notes, sketches, links)
  designs/<id>.sc            SolidCraft design documents (one per design in the project)
  designs/<id>.ext.json      MakerCAD extension store: stock bindings, fold/pattern data
  plan/plan.plancraft        PlanCraft-format plan (open in PlanCraft as well)
  log/journal.jsonl          append-only journal records
  log/sessions/*.json        end-of-session interviews
  media/                     photos, video refs (large video stays external by default)
  scans/                     raw scans (optional embed; default = external link + hash)
  exports/                   last export pack manifest (exports are regenerated, not trusted)
```

Rules:
- **Large media is linked by default**, not embedded: scans and video are often gigabytes. The project stores a path plus a content hash and warns if the file goes missing. "Pack for sharing" embeds everything.
- **Inventory is snapshotted into the project.** The user's global inventory lives in `%APPDATA%\MakerCAD\library.db`. The project keeps the copy it used, so a shared project still makes sense on someone else's machine. On open, differences from the local inventory are offered as a diff ("you now have 3 more boards").
- **A design is plain SolidCraft format plus a sidecar extension file.** A design can be opened in stock SolidCraft (losing stock bindings). That's a good property, and a free compatibility test.
- **Migrations** are versioned. Every format change ships with a migration test.

## 3.7 Environment host (summary; full catalogue in doc 06)

An environment is a bundle of commands, tools, panels, viewport overlays, document extensions and Sancho rules, with a manifest:

```toml
[env]
id = "fold"
name = "Fold & Package"
kind = "builtin"            # builtin | wasm | sidecar
requires = ["sketch"]       # other envs
provides_ns = ["fold"]
cost = { ram_mb = 40, gpu = "low", cpu = "low", disk_mb = 0 }
suggest_when = ["box", "package", "carton", "dieline", "fold", "origami", "papercraft", "sleeve"]
min_profile = "potato"
```

Lifecycle: **Available** (known, nothing loaded), then **Dormant** (compiled in, zero init), then **Loaded** (commands registered, data structures built), then **Active** (UI visible, overlays drawn), and back again.

- The **project declares** which environments it uses. They are chosen in the wizard and stored in `manifest.json`.
- **Switching** environments hides one and shows another; both stay Loaded. **Unloading** frees memory. The host unloads automatically, least recently used first, when the profile's memory budget is exceeded, but never while an environment's data is unsaved or mid-operation.
- **"Keep all loaded"** is a toggle for capable machines. Its only advantage is instant switching and cross-environment live previews (for example, the fold animation updating while you edit in Sketch).

Why compiled-in "dormant" is the right default for built-ins: in Rust, the code of an unused module costs disk and a little address space, not RAM or GPU. What costs RAM is **initialised data**: meshes, solver state, GPU buffers, caches. So "load on demand" means **initialise on demand**. That gets about 95% of the benefit with no plugin ABI problems.

**External environments:**
- **WASM components** through SolidCraft's `craft:plugin` WIT, extended with an `environment` world (commands, panels described as data, overlays as draw lists). They are sandboxed with permission manifests, and are good for third-party environments (circuits, generative tools).
- **Sidecar processes** for heavy native engines (slicer, CFD, OCCT exchange). They talk the same JSON command protocol as the CLI/MCP (stdio JSON lines). A sidecar crash cannot take down the app. On a potato, the sidecar can even run on another machine on the LAN.

## 3.8 Renderer

SolidCraft's split is used as is: `viewport` (camera, pictures, **CPU rasteriser**, shading), `viewport-gpu` (wgpu), `viewport-egui`. MakerCAD adds:

1. **Backend selection from the probe** (doc 12), never left to wgpu's default, because of the Vulkan segfault on the HD 530. Order on Windows: DX12, then GL, then CPU.
2. **CPU path as guaranteed floor**, tested in CI by rendering golden images.
3. **Overlay layers**: Sancho, space reference (scan/splat/room shell), waste heat-map, projector calibration.
4. **Style pass** for the storybook look (doc 11): soft outlines, flat-plus-rim shading, no post-processing on potato.

## 3.9 AI and MCP boundary

- `mk-mcp` exposes the whole registry: SolidCraft's tools, plus `project.*`, `inventory.*`, `stock.*`, `fold.*`, `pattern.*`, `scan.*`, `plan.*` (from PlanCraft), `log.*` and `assist.*`.
- Outside agents such as Claude Code connect with a per-launch secret key (PlanCraft's pattern).
- Sancho's Mind is just another MCP client running in-process. It has **no privileged access**, and its changes are ⚡-marked (doc 07).
- The model provider is pluggable: any cloud API the user configures, or a local model server. With the Mind off there is **zero network**, and that is tested.

## 3.10 Quality gates

| Gate | Command | Fails when |
|---|---|---|
| Layers | `cargo xtask layers` | upward dependency |
| Brand-free | `cargo xtask brand-free` | any dependency on `ui-kit`/`look`/`theme` or a brand asset |
| wasm | `cargo xtask wasm` | a non-app crate stops building for wasm32 |
| Maker corpus | `cargo xtask corpus` | a corpus part's volume/area/bbox, stock usage or cut list changes unexpectedly |
| Golden exports | `cargo xtask golden` | DXF/SVG/PDF/log HTML differ from golden files |
| Potato run | `mk-cli bench --profile potato` on the reference machine | a budget in doc 12 is exceeded |
| No-network | `mk-cli selftest --offline` under a network sandbox | any socket opened with the Mind off |
