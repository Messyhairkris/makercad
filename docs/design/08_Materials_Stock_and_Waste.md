# 08 — Materials, Stock and Waste

The brief: *"If they have 15 pine 2x4s in 3 foot lengths, the material budget is set. The user can sketch the cuts onto the boards to use the materials they have exactly, or they can model first and the program helps determine how the materials are used… If the dresser doesn't have enough wood for a specific design, how can it be modified to use what is available? … There is always some waste, and this will help visualise it."*

This document is how MakerCAD does that.

## 8.1 Stock binding

Every body in a design can be **bound** to a stock source (`mk-stockbind`):

```json
{
  "body": "body:left-post",
  "source": "mat:pine-2x4-36",           // or salv:… part, or "to_buy:…"
  "orientation": { "grain_axis": "body.Z", "face_up": "wide" },
  "form_fit": "exact",                    // exact | oversize_ok | needs_rip | needs_lamination
  "piece": "piece:7",                     // assigned physical piece (after layout), or null
  "locked": false
}
```

- **How bodies get bound:** automatically when made with stock-aware tools ("Board from stock", "Panel from stock", painting with a material in Quick Build); by drag-and-drop from the Inventory drawer onto a body; or by the **auto-binder**, which proposes bindings for unbound bodies by matching cross-section, thickness and material hints from the intent.
- **Form fit check:** a body is a candidate for a stock item if its minimal bounding box fits inside the stock's section and length, given orientation and grain. `needs_rip` means the section is smaller than the stock (rip it down), which needs `cut.rip` capability. `needs_lamination` means it is bigger than any single piece and has to be glued up.
- **Unbound bodies** show with a hatched "no material" look. The material meter counts them as "unknown".

## 8.2 The live material meter

Always on the status bar, and expandable into a panel:

```
Pine 2x4 @ 36″   ███████████░░░░  11 / 15 boards   waste 9% (+ 3 usable offcuts)
¾″ Plywood       ██████░░░░░░░░░  0.55 / 1 sheet   waste 14%
Drawer slides    ████████████████  4 / 4 pairs  ✓
Budget           ██████░░░░░░░░░  $38 / $120 (owned stock $0 marginal)
```

- It updates **incrementally** while modelling, on a debounce of about 300 ms after a commit, never during a drag. Layout (§8.3) is fast for typical maker counts (< 200 parts). Bigger jobs run as a background job, and the meter shows "≈" until it finishes.
- Colours: green within inventory, amber at > 85%, red over inventory (with "short by 2 boards"). A shortage is never an error; it's a fact with options (§8.5).
- **Why?** opens the layout that produced the numbers.

## 8.3 Layout engines (`mk-nest`)

| Form | Engine | Notes |
|---|---|---|
| Board, rod, tube (1D) | **Cutting-stock**: first-fit-decreasing seed, then improvement (best-fit swaps, exact DP or branch-and-bound for ≤ 25 parts per stock type) | Kerf per tool; trim allowance for rough ends (default 1/4″ off factory ends: on); defects as forbidden intervals |
| Sheet (2D, rectangular parts) | **Guillotine** packer (default) or maximal-rectangles | Guillotine = every cut goes edge to edge, which a circular saw, track saw or panel saw can actually make. Outputs a **cut sequence**. Grain locks rotation |
| Sheet (2D, shaped parts) | Rectangle-bounded first; **true-shape nesting** with the Laser/CNC environment | Packaging dielines; tab-and-slot parts |
| Fabric (roll) | 2D strip packing with grain line (rotation 0°/180° only, or ± a bias allowance) | Nap direction; pattern matching (stripes) as a later option; pairs (cut 2 mirrored) |
| Block (3D) | Bounding-box allocation only | Carving or turning blanks |
| Filament, liquid, granular | Volume or mass sum + process waste factor | Supports, purge, mixing loss |

**Manual mode, "draw cuts on the boards"** (the brief's first workflow): Stock & Cut shows the **rack**, every owned board and sheet drawn to scale. The user drags parts onto pieces, or **sketches parts directly on a piece** ("sketch on stock", doc 06 §6.1); the sketched outline becomes a body bound to that piece. Manual placements are `locked`, and the optimiser places the rest around them.

**Design-from-stock:** select boards on the rack, then "Use these" makes bodies from them at full length. The user then cuts them to size in the model. This is how many reclaimed-wood makers actually think.

## 8.4 Worked example: the 15 boards

Inventory: 15 × pine 2x4 (actual 1½″ × 3½″) at 36″. Jigsaw kerf ⅛″ (from the tool record). Rough-end trim off.

A simple dresser carcass frame has:

| Part | Qty | Length |
|---|---|---|
| Post | 4 | 33″ |
| Front/back rail | 8 | 15″ |
| Side rail | 6 | 12″ |

Total part length: 4×33 + 8×15 + 6×12 = **324″**. Stock on hand: 15 × 36″ = **540″**.

The cutting-stock layout:

| Boards | Pattern | Offcut each |
|---|---|---|
| 4 | 33 \| kerf | 36 − 33 − ⅛ = **2⅞″** |
| 4 | 15 \| 15 \| (2 kerfs) | 36 − 30 − ¼ = **5¾″** |
| 3 | 12 \| 12 \| (2 kerfs) | 36 − 24 − ¼ = **11¾″** |

**11 of 15 boards used** (396″). Parts 324″ + kerf 2¼″ + offcuts 69¾″ = 396″ ✓.

The meter reports two numbers, because both are honest:
- **Consumed waste:** (396 − 324) / 396 = **18.2%**
- **True waste:** offcuts below the "usable" threshold (6″ for 2x4, user-settable) plus kerf: 4 × 2⅞ + 4 × 5¾ + 2¼ = 36¾″, which is **9.3%**. The three 11¾″ offcuts go back to inventory as **usable offcuts** (condition `offcut`) and are offered for drawer runners or cleats. The remaining 4 full boards stay available.

**Visualisation:** in Stock & Cut, each board is drawn with parts in pastel colours, kerfs as thin dark lines, usable offcuts striped and hatched-green, and true waste in a soft red. The same colours show in 3D as a waste overlay toggle, which colours each body by how efficiently its board was used.

## 8.5 "Make it fit what I have"

When the design exceeds inventory, Sancho (rules first) runs the **fit solver**: a search over design parameters and techniques that reduces material, ranked by how much it changes the design. Each option shows the before/after meter and a 3D preview. Applying one is a normal parametric edit, with a "why" written to the journal.

Strategies, in this order (least invasive first):

1. **Re-layout**: a better packing or rotation (if grain allows). Changes no geometry.
2. **Use offcuts and salvage**: route small parts to usable offcuts or claimed salvage.
3. **Tolerance nudges**: reduce lengths by ≤ 2% where a parameter isn't constrained by a Space fit ("posts 33″ → 32⅝″ lets 2 rails come from each post board").
4. **Change a part's source**: rip a wide board into two narrow parts; use plywood strips instead of solid boards for hidden rails.
5. **Joinery swap**: a half-lap instead of a separate cleat; a frame-and-panel instead of solid.
6. **Structural thinning**: fewer rails, wider spacing, with a check against the structure rules (sag and racking), and the margin shown.
7. **Scale the design**: reduce a free dimension (height, depth) within the intent's constraints.
8. **Change technique**: lamination (glue up short boards to make long ones, needing `clamp` and `join.glue`), finger-jointed splices, or a lamina-style technique (stacked or interlocked from sheet).
9. **Buy**: the shopping list with the minimum purchase ("one 8 ft 2x4 covers it: $4.10").

**Example.** The design needs 38″ posts and the boards are 36″. Rule `stock.part_longer_than_stock` fires. Options:
- Lower the dresser 2⅛″ (posts 35⅞″). The alcove fit is unaffected; the top drawer gets 2⅛″ shallower.
- Laminate posts from two board pieces with a staggered joint (needs clamps, which you have).
- Use a plinth: 33″ posts on a 5″ base frame from offcuts. **Uses 0 new boards**, and fits the "boat-like" style words.
- Buy 2 × 8 ft 2x4s ($8.20).

The point is **context that sparks a creative solution**, not a refusal.

## 8.6 Kerf, tolerance and tool reality

- Kerf comes from the **tool used for that cut**, not a global setting: table saw ⅛″, jigsaw ~1.5 mm, laser ~0.1–0.2 mm, knife 0.
- Cut accuracy (from the tool) propagates into **fit checks**: a joint designed at 0 clearance with a ±1 mm tool raises a tip, "add 0.5 mm clearance or plan to pare it to fit."
- **Measured overrides**: the "measure one" prompt ("Your ¾″ plywood: please caliper it") updates thickness project-wide, and dados resize automatically. This is the classic maker failure, and it costs one prompt to prevent.

## 8.7 Shopping list and supplier links

- Generated from shortfalls, hardware not on hand and consumables (glue, screws, finish), using the **minimum purchase** that covers the need given standard sizes (8′, 10′, 12′ lengths; 4×8 sheets; fabric by the ¼ yd).
- Optional local supplier price lists (CSV import). No scraping, no accounts.
- Items can be marked "bought" and move into inventory with a cost.

## 8.8 Grain, defects and reclaimed material

- **Grain arrows** on stock and on bound bodies. A rule flags grain against load on shelves and short-grain weakness on curved parts.
- **Defect mapping:** photograph a reclaimed board beside a tape. Mark nail holes, splits and knots as zones (by tapping on the photo), and the nester avoids them. Optional automatic suggestion from the photo, user-confirmed.
- **Character placement:** mark a defect as "feature" instead of "avoid" (a nice knot) and pin it to a visible face of a chosen part.

## 8.9 Material use beyond wood

| Material | What "budget" means | Example rule |
|---|---|---|
| Paperboard / corrugated | Sheets of a size; caliper; flute direction | Flute must run perpendicular to the main fold for strength |
| Fabric | Yards/metres of a width; nap; stretch | Pattern piece wider than the fabric's usable width |
| Paper-mâché (recycled paper) | Mass of paper + paste for a surface area × layers | "6 layers over 0.8 m² ≈ 1.1 kg newspaper — you listed about 2 kg ✓" |
| Filament | Grams on spools | Print needs 340 g; spool has ~210 g left |
| Rubber / tubing | Length/area; joining method | Welded joints need a solvent you don't have listed |
| Salvage objects | Specific recovered parts | Kallax panels are 15 mm particleboard; edges won't hold screws |
