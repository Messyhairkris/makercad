# P0 — SolidCraft's `craft:plugin` world, read for Craft Modules

Pinned checkout `third_party/solidcraft`, rev `906b5e86`. Doc 14 P0 asks to read the WIT `craft:plugin`
world; doc 16 (C1, C2) and D19 build MakerCAD's code modules on it plus our own `maker:module`
package. This note records what the pinned revision actually has, and what that means for the plan.

## The contract (`crates/craft-plugin/wit/plugin.wit`)

`package craft:plugin@0.2.0`. App commands and their schemas are JSON data, so the WIT never names an
app's types and one contract can serve every "Craft" app.

| Interface | What it gives |
|---|---|
| `types` | `json` (a string), `plugin-error`: permission-denied, command-failed, limit-exceeded, invalid-manifest, invalid-input, trap, cancelled, unsupported |
| `commands` | `run(command, args) -> json`: the same command surface the app's MCP clients get |
| `log`, `control`, `clock` | logging; progress and cancellation; time |
| `files` | read/write a file through a named grant the user approved |
| `entry` (export) | `execute(entry, params)`: the plugin's own entries, called by name |
| `source`, `design`, `evaluate`, `importer` (export) | importers: read the source file in 1 MiB chunks, write a design file, check the result in a private session |

Two worlds: **`command-plugin`** (imports commands, log, control, files, clock; exports `entry`) and
**`importer-plugin`** (imports log, control, clock, source, design, evaluate; exports `importer`).
Capabilities named in a plugin's manifest: `commands@0.2`, `formats@0.2`, `evaluate@0.2`, `storage@0.1`.
Its own comment says UI and viewport capabilities "arrive as their own interfaces" later; a draft UI
world is in `docs/rfcs/0015-plugin-ui/craft-plugin.wit` (RFC stage, not in the host).

While the version is 0.x a minor version may break the one before it. The host still runs 0.1
plugins (`wit-0.1/plugin.wit`), reading the version from the component's imports and exports.

## The host (`crates/craft-plugin`, `crates/craft-plugin-wasm`)

- **App-independent and MIT/Apache.** `craft-plugin` depends only on serde, serde_json, schemars and
  thiserror; `craft-plugin-wasm` adds `craft-plugin`, wasmtime, cap-std and log. Neither depends on any
  SolidCraft crate (SolidCraft's `cargo xtask layers` forbids it), and neither is brand material.
- **An app implements `PluginHost`**: command schemas, each command's read or edit effect, running
  commands, transactions (one plugin call is one undo step; a failure rolls back), and the folders
  file grants may use. Granted file writes are staged and published only after the call succeeds.
- **Plugins ship no UI code.** Entries can ask for named menu or ribbon slots; the app decides which
  slots exist and draws forms from the entries' input schemas.
- **Sandbox:** wasmtime 48.0.8 with fuel, a memory ceiling (128 MiB by default, raised per package
  with the user's approval), a stack limit, and a deadline and cancellation check on every epoch
  tick; memory64 and multi-memory are off; **no network imports** in the linker.
- **Engine:** wasmtime's `cranelift` compiler, only on x86_64 and aarch64. Nothing else is compiled
  for other targets, so on the web the browser's own engine would run components.

## What this means for MakerCAD

1. **D19 holds.** Both crates can be used as they are. MakerCAD implements `PluginHost` over its own
   command registry; `craft:plugin` modules and SolidCraft plugins can run in MakerCAD.
2. **`maker:module@0.x` is a separate package**, never an edit of `craft:plugin` (doc 16 C1). Its
   first interfaces are the ones doc 16 needs that `craft:plugin` lacks: environments, panels as data,
   overlays as draw lists, and inventory/stock access for Method and manufacturer modules.
3. **iOS needs an interpreter.** The pinned host enables only `cranelift`, which generates code at run
   time, and iOS forbids that. Doc 16 already says "interpreter only" for iOS. The mobile build
   needs wasmtime's `pulley` interpreter feature or `wasmi` behind the same `PluginHost`. Measure
   that on a phone before promising mobile Craft Modules.
4. **The network stays out.** The host refuses network access, so doc 16's `maker:net@0.1` (for
   manufacturer catalogues) is ours to design, opt-in and labelled (principle P10).
5. **Watch 0.x churn.** `craft:plugin` may break between minor versions. Doc 16's `xtask wit-frozen`
   and a corpus of modules built against each released version guard against that.
