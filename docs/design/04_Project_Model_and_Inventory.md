# 04 — Project Model and Inventory

## 4.1 The object model

```
Project
 ├─ Intent           goal text, success criteria, constraints, skill level, style words
 ├─ Deadline         due date, work windows (evenings/weekends), milestones
 ├─ Budget           money cap, currency, "spend only if needed" flag
 ├─ Spaces[]         where things must fit: measured boxes, scans, room shells, keep-outs
 ├─ Inspiration      mood board: images, notes, sketches, links, colour swatches, references
 ├─ Inventory (project snapshot)
 │    ├─ Tools[]          what you can do
 │    ├─ Materials[]      what you can make it from (consumable stock)
 │    ├─ Salvage[]        things you could take apart or reuse
 │    ├─ Hardware[]       screws, hinges, buckles, zippers, pulls, magnets
 │    └─ Computer         this machine, as a tool (doc 12)
 ├─ Designs[]        each a SolidCraft document plus MakerCAD extensions
 │    └─ Bodies → StockBinding → Material/Salvage item
 ├─ Environments[]   which modules this project uses
 ├─ Plan             tasks, resources, schedule (PlanCraft core)
 ├─ Journal          decisions, steps, sessions, outcomes, media
 └─ Exports          last generated export pack
```

A **project can hold several designs**: a pencil sharpener *and* its box, or a bed, shelf *and* desk that fold into each other. Designs can reference each other: the box's inner cavity can be driven by the sharpener's bounding box plus clearance, and that link survives edits.

## 4.2 Intent

```json
{
  "name": "Alcove dresser",
  "goal": "Build a dresser to fill the weird cutout in my bedroom wall.",
  "success": ["fits the alcove with < 5 mm gap", "4 drawers", "done before Dec 12"],
  "constraints": ["no power tools after 9pm (apartment)", "must come apart for moving"],
  "skill": "beginner",                 // beginner | comfortable | experienced | pro
  "style_words": ["warm", "simple", "mid-century"],
  "history": [ { "at": "...", "text": "...", "why": "..." } ]   // every edit kept
}
```

- The **goal text is the most important field in the program.** The project interpreter reads it (doc 05 §5.4), the toolbox is built from it, Sancho checks against it, and it heads the published log.
- **Success criteria** are checkable where possible. The interpreter turns "fits the alcove" into a fit check against a Space, and "before Dec 12" into a plan deadline. Uncheckable ones ("looks warm") remain notes, and the end-of-project interview asks about them.
- Intent edits are versioned with a "why". The gap between the first idea and the final build is part of the story.

## 4.3 Inventory: four kinds of thing

The inventory is **global per user** (`library.db`, SQLite) and **snapshotted per project**. Entry has to be quick, or nobody will do it. Doc 05 covers the entry UX: kits, photos, quick-add phrases.

Every tool and material is **global** or **local** (D30). *Global* items are the person's workshop: they have their own walkthrough, "My workshop" (tools, then materials, then what has changed), so keeping them up to date is easy, and every project can use them. *Local* items belong to one project (say a board bought for this task) and never enter the workshop on their own. A project's snapshot (`project/inventory.json`) marks each entry `"scope": "global"` or `"scope": "local"`. When the project is **finished**, its leftover local materials and tools are offered to the workshop: a material with how many are left, a tool with how the person has it now; the ones the person adds become global. A tool can also be there **only some days** (D31: "a multimeter on weekends"): each tool has a "when", every day unless said otherwise, or chosen days of the week with a free note ("when my brother visits"). The workshop's tools have theirs in My workshop; a project's own tools have theirs in its snapshot (`"when": {"days": ["sat","sun"], "note": "..."}`, written only when there is something to say). A plan can ask how many days until a tool can be used; the note is only shown to the person.

### 4.3.1 Tools

```json
{
  "id": "tool:jigsaw-1",
  "kind": "jigsaw",                       // from the tool taxonomy
  "label": "Ryobi jigsaw",
  "capabilities": ["cut.curve", "cut.straight:rough", "cut.sheet"],
  "limits": { "max_thickness_mm": 50, "kerf_mm": 1.5, "accuracy_mm": 1.0 },
  "access": "owned",                      // owned | shared | makerspace | rentable | wishlist
  "availability": "evenings",             // feeds PlanCraft resource calendar
  "noise": "loud",                        // apartment-friendly checks
  "photo": "media/…", "notes": ""
}
```

The **tool taxonomy** maps tools to **capabilities**. Design steps need capabilities, not brands. A straight cut can be made with a table saw, a circular saw and guide, a jigsaw (rough), a hand saw, or a laser for thin stock. Each tool supplies a capability at some accuracy, speed and kerf. This is how MakerCAD can say "your jigsaw can do this cut but expect ±1 mm; clamp a straightedge" instead of "you need a table saw".

Capability families (v1): `cut.straight`, `cut.curve`, `cut.sheet`, `cut.crosscut`, `cut.rip`, `cut.angle`, `cut.laser`, `cut.knife`, `cut.die`, `cut.fabric`, `drill.hole`, `drill.pocket`, `join.screw`, `join.glue`, `join.dowel`, `join.weld`, `join.solder`, `join.sew.straight`, `join.sew.zigzag`, `join.sew.heavy` (webbing), `join.rivet`, `shape.plane`, `shape.sand`, `shape.route`, `shape.lathe`, `shape.mill`, `shape.print3d`, `shape.mold`, `fold.score`, `fold.crease`, `finish.paint`, `finish.oil`, `measure.length`, `measure.angle`, `measure.scan`, `project.light` (projector), `print.paper` (with max sheet size), `clamp`, `heat.press`, `heat.gun`.

**Tool kits** for fast entry: "Basic household" (tape, utility knife, screwdriver set, hammer, scissors), "Starter woodshop", "Full woodshop", "Sewing corner", "Papercraft/packaging", "3D printing", "Laser", "Makerspace member". Pick a kit, then untick what you don't have.

### 4.3.2 Materials (consumable stock)

```json
{
  "id": "mat:pine-2x4-36",
  "material": "wood.softwood.pine",       // material taxonomy → properties
  "form": "board",                         // board | sheet | rod | tube | fabric | roll | block | sheet_paper | filament | liquid | granular
  "nominal": "2x4", "actual_mm": [38.1, 88.9],
  "length_mm": 914.4,
  "qty": 15,
  "condition": "new",                      // new | used | reclaimed | offcut
  "defects": [],                           // per-piece: knots, splits, nail holes (reclaimed)
  "grain": "along_length",
  "cost_each": 3.48, "source": "Home Depot", "on_hand": true
}
```

**Forms** decide how the stock is nested and cut (doc 08): boards and rods use 1D cutting; sheets and fabric use 2D nesting; rolls are 2D with one dimension unbounded; blocks are 3D subtractive; filament, liquids and granular materials are consumed by volume or mass.

**The material taxonomy** carries properties used by the checks: density, stiffness class, minimum bend radius, foldability (paperboard caliper and flute), stretch (fabric warp/weft/bias), laser-safe (**PVC → never**), glue compatibility, outdoor rating, food-safe. Values come with ranges and sources, and they are **editable**: "my plywood is actually 17.6 mm" is normal, and per-item measured overrides are encouraged ("measure one with calipers" prompt).

**"Anything can be a material."** Examples the quick-add should understand: "a stack of newspapers" (paper-mâché, papercraft); "old cereal boxes" (folding cartonboard); "an old tent" (ripstop nylon, poles as rods); "bike inner tubes" (rubber sheet, tubing); "two IKEA Kallax units" (salvage, see below); "half a sheet of 3/4 ply in the garage"; "5 kg of PLA". Unknown materials are allowed with a free-text kind and user-entered properties. Sancho will just be less sure.

### 4.3.3 Salvage (upcycling)

Salvage is **an object you could take apart or reuse**, modelled as a parent with recoverable parts:

```json
{
  "id": "salv:old-bookshelf",
  "label": "Old particleboard bookshelf",
  "photo": "media/…",
  "parts": [
    { "as": "mat", "form": "sheet", "material": "wood.particleboard.melamine", "actual_mm": [800, 300, 16], "qty": 4 },
    { "as": "hardware", "kind": "shelf_pin", "qty": 16 },
    { "as": "hardware", "kind": "cam_lock", "qty": 8 }
  ],
  "disassembly": { "effort": "low", "tools": ["screwdriver"], "notes": "back panel is 3mm hardboard" },
  "status": "intact"                      // intact | partially_used | used
}
```

- Salvage parts become stock **only when claimed by a design**. The disassembly then becomes a plan task: "Take apart the old bookshelf (20 min, screwdriver)".
- **Reclaimed defects matter.** A reclaimed board can have marked nail holes and splits (photo-traced in Stock & Cut), and the nester routes parts around them.
- **Salvage scanning:** photograph an item beside a scale reference (doc 09 §9.7) for rough dimensions, or scan it for the exact shape.

### 4.3.4 Hardware and the parts library

Fasteners, hinges, slides, pulls, buckles, zippers, magnets, bearings, electronics. Each item is a **parametric part** where possible (screw: gauge, length, head; drawer slide: length, extension, clearance), or a **modelled part** (the vintage pull, modelled from photos per the 2026-08 walkthrough).

- A local **parts library** is shared across projects ("Use Part" on right-click, per the walkthrough).
- Parts can be exported and imported as `.makerpart` (geometry, parameters, metadata, photo, licence) and shared with other people.
- Experimental: **AI-assisted model from photos** for a part with no model online, flagged as experimental and checked against measured dimensions before use.

### 4.3.5 The computer

The current machine is an inventory item of kind `computer`, filled in by the probe (doc 12). It has capabilities (`render.splat`, `run.physics.realtime`, `slice.local`, `scan.process.large`) and limits. Treating it as a tool means the same machinery that says "you need a router for this" also says "this scan is too big for this laptop to clean in one go; I'll process it in chunks overnight, or you can do it on the desktop."

Phones and tablets join as tools too: a **phone with LiDAR** is `measure.scan` (room); a phone camera is `measure.photo`; a projector is `project.light` with resolution and throw ratio.

## 4.4 Spaces

A Space is **where things must fit, or must not go**. Kinds:

| Kind | Made from | Example |
|---|---|---|
| `box` | Typed dimensions | "Under the window: 1200 × 450 × 760" |
| `outline` | 2D polyline plus height | Alcove footprint traced from a tape-measured sketch |
| `room_shell` | Scan to semantic room (doc 09) | Bedroom: walls, floor, ceiling, door swing, window, outlets |
| `envelope` | Body or accessory set | "Everything that goes in the backpack" (doc 13 §13.3) |
| `keep_out` | Any of the above | Door swing arc, radiator clearance, walkway 600 mm |
| `bench` | Implicit for small projects | The virtual workbench top |

Spaces are **reference geometry** in every environment: they can be snapped to, measured and used as constraints ("dresser width = alcove width − 2 × 3 mm"). Fit checks run continuously and report in the status bar: **Fits ✓ 3 mm each side**.

## 4.5 Deadline, budget and time

- **Deadline** is a date plus **work windows** ("weeknights 7–9, Saturday mornings"). These become the maker's calendar in PlanCraft, and the schedule respects them.
- **Budget** is the money cap. The budget meter counts **owned stock at zero marginal cost**, with "replacement value" shown as a note, plus purchases and rentals. Sancho warns at 80% and 100%.
- **Time estimate** per step comes from technique and tool rates times quantities. It starts generic and calibrates as the user logs actual times ("your cuts take 1.4× the estimate; I've adjusted the plan").

## 4.6 Inspiration board

- A free canvas of images, notes, colour swatches, link cards, hand sketches (pen input on tablets) and quick 3D snapshots.
- Any image can be **placed into the design as a reference**: as a calibrated canvas on a plane (scale from two clicked points and a typed distance), or projected onto a surface.
- **Style words** extracted from notes ("boat-like", "fold away", "warm wood") feed Sancho's suggestions and the toolbox.
- The board belongs to the project, stays visible as a side drawer in every environment, and is the first page of the published log.

## 4.7 Units

- Stored internally as millimetres (SolidCraft's `display-units`).
- Display preference per user: metric, or **imperial with fractions** (`mk-units-maker`: 1/16″, 1/32″ display rounding, "35 7/16″").
- **Nominal versus actual** is a first-class concept: "2x4" means 1.5″ × 3.5″ actual; "3/4 ply" means about 18 mm (measured override recommended). Shown as `2x4 (1½″ × 3½″)`.
- Fabric in gsm or oz/yd²; paperboard in pt/caliper and ECT; wire in AWG.
