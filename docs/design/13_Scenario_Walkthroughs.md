# 13 — Scenario Walkthroughs

Five projects, end to end, showing which environments load, what Sancho does, and what comes out. These double as the **golden projects** for testing (docs 07, 10, 12).

---

## 13.1 The alcove dresser (room scale, wood, scan)

**Quest answers.** Name "Alcove dresser". Goal: *"Build a dresser to fill the weird cutout in my bedroom wall. It should have 4 drawers and come apart for moving."* Space: "I have a scan", a LiDAR-phone room export. Tools: "Starter woodshop" kit minus table saw (jigsaw, drill/driver, pocket-hole jig, clamps, sander, tape, square). Materials: "15 pine 2x4 3ft", "1 sheet 3/4 ply", "4 pairs drawer slides 14 inch". Deadline Dec 12, weeknights 7–9 and Saturday mornings. Budget $120. Inspiration: 3 mid-century photos.

**Interpreter:** words *dresser*, *drawers* and *bedroom* give scale `room` and envs Solid, Sketch, Stock & Cut, Plan, Journal. The scan attachment adds Space & Scan. *Come apart* adds a rule: prefer knock-down joinery (cam locks, bolts, cleats). No table saw adds a tip: favour jigsaw-friendly joinery and a circular-saw guide.

**Space.** The room export already has walls and openings. Detection adds skirting boards (12 mm proud, 90 mm high), an outlet in the alcove (Sancho: "there's an outlet at 300 mm on the back wall of the alcove — leave a cutout or keep the back open?") and a picture frame. The alcove measures 812 mm at the mouth, 790 mm at the back, 410 mm deep, and is 1.2° out of square.

**Design.** The user picks the starter "Dresser from space". The carcass is parametric: width = mouth − 2 × 3 mm, depth = 400 mm, back panel tapered to follow the 1.2° wall, or a plain back with a scribe strip. Sancho suggests the **scribe strip** (simpler, which matches skill = beginner). The frame comes from 2x4s, the panels and drawer boxes from plywood, the slides from inventory (the 14″ slides fix drawer depth ≥ 356 mm ✓).

**Stock.** The meter shows 2x4: 11/15 (waste 9%, plus three usable offcuts), plywood: 1.08 / 1 sheet ⚠. The fit solver offers: (1) re-layout with rotation where grain allows: 0.97 ✓ but the drawer bottoms run across grain; (2) drawer bottoms from ¼″ hardboard salvage (the user has an old bookshelf back panel); (3) buy a 2×4 ft ¼″ ply panel for $9. The user picks (2). The salvage is claimed, and a "Disassemble old bookshelf back" task appears.

**Motion.** A quick check: drawers fully open don't hit the bed (from the scan, 640 mm away ✓). The tip-over rule fires: tall and shallow with drawers out, so an anti-tip strap is added to the BOM and to the plan.

**Plan.** 19 steps, about 14 working hours at a beginner rate: 6 evenings and 1 Saturday, with the critical path through glue cure. It finishes Dec 9, so there are 3 days of slack.

**Output.** Cut diagrams per board, plywood sheet cut sequence (guillotine, circular-saw friendly), blueprints, build guide, shopping list (anti-tip strap, screws, finish: $31).

**After the build.** Photos per step from the phone. A scan of the room with the dresser in place, compared: built width 803 vs. designed 806 (−3 mm). The journal is published as HTML.

---

## 13.2 The product package (bench scale, paperboard, production)

**Quest.** Goal: *"A retail box for my handmade candle (jar 80 mm dia × 95 mm) — needs a window and a hang tab. I'll make 200 at first, then a print shop for 2000."* Tools: cutting mat, scalpel, bone folder, steel rule; a **cutting plotter** (Cricut-class) in inventory; a projector. Materials: 300 gsm SBS board (sheets 12 × 12 in), acetate film, double-sided tape.

**Interpreter:** *box*, *retail* and *window* give envs Fold & Package, Sketch, Drawings, Output. *200 / 2000* sets quantity, which brings the press-sheet nesting and print-shop export presets.

**Design (3D first).** Drop in the jar (a cylinder from typed dimensions, or a photo with reference). "Box around product" with 2 mm clearance, then template **ECMA reverse tuck end** sized to it. Add: window on the front panel (with 6 mm film overlap and acetate in the BOM), a euro-slot hang tab on the top tuck extended up, and a friction lock (slit lock) on the tuck flaps. The board's caliper (0.42 mm) gives fold allowances automatically, and the table is shown.

**Checks.** The fold animation at 0–100% shows the dust flap colliding with the window film area at 70%. Sancho suggests shortening the dust flap by 4 mm (one click). A crease is 1.5 mm from a cut on the hang tab, below the 3 mm minimum for this board, so the tab grows 2 mm. Artwork (SVG from the user) is mapped onto the panels in 3D, with the bleed check passed.

**Output.** (a) Plotter SVG with cut and score layers, nested 1 box per 12 × 12 sheet: board use 83% → 200 boxes = 200 sheets (inventory has 150, so the shopping list adds 50 + spares). (b) Print-shop PDF: dieline with spot-colour layers CUT/CREASE/BLEED/ART and dimensions, plus a press-sheet nesting of 6-up on 28 × 20 in for the 2000 run. (c) An assembly guide for hand-folding.

---

## 13.3 The activity backpack (body scale, textiles, flattening)

**Quest.** Goal: *"A climbing approach pack that carries exactly my rope, harness, shoes, chalk bag and a 1.5 L bladder; roll-top; 50 cm torso."* Tools: domestic sewing machine plus a borrowed walking-foot machine ("Saturdays"), rotary cutter, mat. Materials: 1.5 m 500D Cordura (150 cm wide), 1 m X-Pac, 3 m 25 mm webbing, buckles, foam sheet, a 1 m piece of an old bike inner tube (for a hydration port grommet).

**Envelope.** The user photographs each item on the floor beside an A4 sheet. Each photo is rectified and traced, and an extrude height is typed (rope coil 120 mm). The items are arranged in 3D as the **accessory layout** to make the **envelope** Space, and Sancho reports: "Contents ~19 L."

**Design.** Soft Goods + Solid. The pack body is modelled as a lofted shape around the envelope + 10 mm clearance. Seam lines are drawn on the surface: front, back, sides, base, roll-top extension. Flattening: the side panels show 6% distortion where the base curves, and woven Cordura doesn't stretch, so Sancho suggests a dart or a separate base panel. The user adds a base panel, and distortion drops below 1%. Seams: bound inside seams on the body; webbing bar-tacked at the shoulder points (the walking-foot machine is required, which feeds the schedule: Saturdays). Hardware: buckles from the library fitted to the 25 mm webbing ✓. The hydration port is made from inner tube (salvage, rubber; flag: glue, not sew).

**Sancho on Sancho:** set to 50 cm torso length, he wears the pack and the straps are checked. The shoulder strap anchor is 4 cm too low for this torso, so it is moved.

**Output.** Pattern pieces nested on the 150 cm fabric width with the grain line respected: 1.12 m used of 1.5 m ✓. Tiled PDF with test squares, plus **projector mode** for cutting directly on the table. A sewing order of 23 steps, with machine changes batched (all walking-foot steps on Saturday).

---

## 13.4 The fold-away bedroom (room scale, multi-state, budget)

**Quest.** Goal: *"Redesign my 2.6 × 3.1 m bedroom like a boat cabin. A Murphy bed that becomes a bookshelf face, a fold-down craft table, and a living mode with a fold-out bench."* Tools: a mid workshop, open to buying. Budget $1,400. Deadline 3 months. Inspiration: sailboat interiors, Japanese tansu.

**Interpreter:** *fold-down*, *Murphy* and *boat* give envs Motion, Solid, Space, Stock, Plan. Multi-design project: bed unit, table, bench, shelves.

**Space.** A room scan gives the shell, window, door (with swing keep-out), radiator (keep-out, 150 mm clearance) and outlets.

**Design.** Three **named states**: Sleep, Craft, Living. The bed unit is a pivoting frame. The counterbalance helper sizes two gas struts (e.g. 2 × 650 N) from the mattress mass entered in inventory, showing a lift-force curve that "feels like ~4 kg at the handle across the swing". Clearance is checked along every transition: the craft table can't fold down while the bed is down (an interlock note is added); the bench can't open in Sleep (fine, it's documented). The bookshelf face on the bed's underside is limited to items ≤ 120 mm deep with a lip, and Sancho adds a "books fall out at 70°" warning, with a retaining-bar fix.

**Budget.** The meter shows $1,520 / $1,400 ⚠. The fit solver: plywood grade B/BB → BC for hidden parts (−$140) ✓. The struts stay.

**Plan.** 3 phases over 11 weekends, with the critical path through the bed unit and an external dependency on hardware delivery (lead time). Staged build so the room stays usable (the bed is built last, and the old bed stays until then).

**Output.** Blueprints per unit, CNC-shop DXF for the plywood parts (the user doesn't own a track saw; it's in the "Send to CNC shop" preset with a dogbone option), hardware BOM, a safety page (wall anchoring into studs found in the scan, or a "use a stud finder" step).

---

## 13.5 The battery cover (bench scale, fix-it, 3D print)

**Quest.** Goal: *"Replacement battery cover for my cordless drill's remote, the clip broke off."* Tools: 3D printer (bed 220 × 220, 0.4 nozzle), calipers. Materials: PLA 210 g, PETG 600 g.

**Interpreter:** *replacement* and *battery cover* give scale `bench` and envs Solid, Sketch, 3D Print Prep. Giant Sancho leans over the bench.

**Measure.** Photograph the opening on the calibration mat, which gives a rectified image. Trace the opening in Sketch. The calipers prompt asks for opening depth, clip slot width and the latch ledge height.

**Design.** Model the opening first, then the cover = opening − 0.2 mm clearance per side. A snap clip is sized with the "cantilever snap" helper (PETG suggested over PLA for fatigue, because the clip flexes every battery change).

**Print.** Orient with the clip printing along layers (strength, rule: layer lines across a cantilever break). The overhang check passes. 6 g of PETG ✓, 35 min. A sidecar slices, or 3MF is exported for the user's slicer.

**Journal.** Three entries and two photos. It publishes in one click as a "fix it" card.

Total time: ~20 minutes. This is the "small project in a big program" test. The program must not feel heavy here: only 3 environments load, and the Quest is short because the interpreter skipped the space and deadline questions.
