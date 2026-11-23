# 05 — Opening Screen and the New Project Flow

## 5.1 The opening screen

Two big choices and nothing else competing for attention:

```
┌─────────────────────────────────────────────────────────────────────┐
│   ☁  MakerCAD                                         ⚙  ?  (Sancho) │
│                                                                     │
│     ┌───────────────────────┐     ┌───────────────────────┐         │
│     │   ✦  Start something   │     │   ↻  Keep going        │         │
│     │       new              │     │                       │         │
│     └───────────────────────┘     └───────────────────────┘         │
│                                                                     │
│   Recent ───────────────────────────────────────────────────────    │
│   [thumb] Alcove dresser      ●●●○○  due Dec 12   Incomplete log     │
│   [thumb] Sharpener + box     ●●●●○  no deadline                     │
│   [thumb] Hike pack v2        ●○○○○  due Mar 3                       │
│                                                                     │
│   My workshop:  41 tools · 23 materials · 6 salvage · this PC: Modest│
│   [Open workshop]   [Import project]   [Examples]                    │
└─────────────────────────────────────────────────────────────────────┘
```

- **Keep going** opens the last project exactly where it was left: the same environment and camera, plus a one-line "last time you…" from the journal.
- Recent projects show a **progress dot strip** (Define, Design, Plan, Build, Publish), the deadline, and log completeness.
- **My workshop** is the global inventory. It is editable here, outside any project.
- Startup budget: the opening screen appears in under 2 s on the potato. It needs no kernel, no GPU viewport and no environment. Thumbnails are cached PNGs.

## 5.2 The question flow (the "Quest")

A new project is a short, friendly conversation with Sancho. **Every step can be skipped**, and skipped steps can be revisited from the project's Intent card. A progress rail on the left shows the steps; the right side shows what is being built from the answers (toolbox, scale, environments).

| Step | Question | Input methods | What it produces |
|---|---|---|---|
| 1 | **What shall we call it?** | Text | Project name, file name |
| 2 | **What would you like to make?** | Free text (multi-line), voice dictation where the OS supports it, optional sketch or photo | Intent goal; feeds the interpreter (§5.4) |
| 3 | **Where does it need to fit? How big is it?** | First, **which system the project measures in** (metric or imperial; starts from the app's setting). Then pick: "I'll type sizes" · "I have a scan" · "I'll photograph it with a reference" · "It's small, no fixed space" · "I'll decide later". A typed size can be written any way ("1200 mm", "4 ft", "3 1/2 in", "5 ft 6 in"); the box says what it comes to in the project's system as it is typed, and Enter (or moving on) turns the text into that system (D34) | Spaces; scale class (§5.5); the project's measuring system |
| 4 | **What tools do you have?** | Kits + untick; checklist filtered by what the interpreter thinks is relevant; "same as my workshop"; photos of the pegboard | Tools snapshot |
| 5 | **What materials do you have?** | Quick-add phrases ("15 pine 2x4 3ft"), workshop picks, photos, salvage items; examples shown | Materials, salvage, hardware snapshot |
| 6 | **When do you need it?** | Date picker + "no deadline"; work windows ("when can you work on it?") | Deadline, calendar |
| 7 | **What's your budget?** | Number or "use only what I have" or "not sure" | Budget |
| 8 | **What inspires you?** | Drop images, paste links, write notes, sketch | Mood board |
| 9 | **Here's your workspace** | Review card: environments to load, toolbox preview, scale, Sancho's first observations | Confirm → project opens |

Design rules:
- **Questions are data** (`mk-log/quest.toml`), not code. Project templates (packaging, soft goods, room) can add or reorder steps. For example, packaging asks "What goes in the box?" and "How many will you make?" (quantity changes the economics: a die versus a cutting plotter).
- **Examples under every field**, chosen by the interpreter: if the goal mentions "box", the material examples show cereal boxes, chipboard and corrugated E-flute.
- **Quick-add parser** for materials. It handles `qty × nominal × length [material] [condition]` in many forms: "15 pine 2x4s, 3 foot", "half sheet 3/4 ply", "2 yards 500D Cordura", "old tent". Ambiguity is confirmed inline ("3 foot = 36″ each?").
- **Nothing blocks the CAD.** "Skip to designing" is always there. A skipped question shows as a gentle outstanding item on the Intent card, and the card says so again where it becomes relevant, in its own words (Sancho does not carry the message: D32; "I can't check material use yet — tell me what you have?").

> Note on v5's "toolbox stays locked if creation is skipped": dropped. Locking contradicted P1. Instead, the toolbox falls back to the full catalogue in default order, and the project is marked "Intent missing".

## 5.3 Fast entry for real inventories

Inventory entry is the biggest friction in the whole concept. Mitigations:

1. **Workshop once, projects forever.** Global inventory is reused. Each new project only confirms what is relevant ("You have 41 tools; these 9 matter for a dresser"). What a person types into a project's Quest is **local** to that project (D30); the workshop itself is kept up to date in its own walkthrough, and leftover local materials and tools are offered to it when the project is finished. A tool can be there only some days (D31): in the tools step, clicking a project's own tool opens a small form for its days (with a way to take the tool out), and a workshop tool shows its days on its chip.
2. **Kits** (doc 04 §4.3.1).
3. **Photo intake.** Photograph the pegboard, shelf or stack. On-device recognition is optional and offline-capable, and it *proposes* items, which the user confirms. With the Mind enabled, a vision model can do better, clearly marked as AI.
4. **Barcode/receipt intake** (later): scan a receipt to add materials with costs.
5. **Lazy precision.** "Some plywood" is a valid entry; Sancho asks for dimensions only when a design actually uses it.

## 5.4 The project interpreter (the linter, grown up)

From the 2026-08 walkthrough: "A linter looks for words, phrases and other text indicators that begin to pull potential tools for the user's toolbox." The interpreter is a **deterministic rules engine first**, with an optional AI pass second.

**Inputs:** goal text, style words, mood-board notes, materials and tools, space type, and attachments (file extensions: `.ply`/`.splat` mean scan, `.svg`/`.dxf` mean 2D).

**Rule examples** (`mk-assist/interpreter.toml`):

```toml
[[rule]]
when.words = ["dresser","cabinet","shelf","desk","bookcase","table","bed"]
then.scale = "room"
then.envs = ["solid","sketch","stock"]
then.tools_ui = ["box_joint","drawer_wizard","shelf_pins","panel_from_stock"]
then.questions = ["space"]

[[rule]]
when.words = ["box","package","packaging","carton","sleeve","mailer","dieline"]
then.envs = ["fold","sketch","drawings"]
then.ask = "What goes in the box?"
then.templates = ["FEFCO 0201","ECMA A20.20 reverse tuck end","tuck-top mailer","sleeve"]

[[rule]]
when.words = ["backpack","bag","pouch","sew","fabric","pattern","quilt","cover"]
then.envs = ["softgoods","solid"]
then.scale = "body"

[[rule]]
when.words = ["fold away","murphy","foldable","collapsible","boat","tiny house","convertible"]
then.envs = ["motion","solid","stock","plan"]
then.tools_ui = ["fold_states","hinge","gas_strut","clearance_sweep"]

[[rule]]
when.words = ["replacement","broken","battery cover","clip","knob","bracket"]
then.scale = "bench"
then.envs = ["solid","sketch"]
then.suggest_tools = ["measure_from_photo","calipers_prompt","print3d_check"]

[[rule]]
when.attach_ext = ["ply","splat","spz","e57","las","usdz"]
then.envs = ["space"]
```

**Outputs** (shown on the review card, each with *why*):
- **Environments to load**, e.g. "Fold & Package, because you said *box*."
- **Toolbox contents**: the subset of commands surfaced in the side toolbox. The rest stays reachable by search. Experts can pin tools to "always show" (walkthrough requirement).
- **Scale class** (§5.5).
- **Templates and starters** to offer.
- **Early observations** from Sancho: "You listed a jigsaw but no table saw; I'll favour joinery a jigsaw can do."

**Confidence:** each conclusion carries a score. Under 0.6, it is shown as a question, not a decision ("Is this something you'll sew?").

**AI pass (optional):** if the Mind is on, the goal text also goes to the model, which can propose extra environments or questions. These appear as ⚡ suggestions next to the rule-based ones, never mixed in silently.

## 5.5 Scale classes and the workspace

The scale class sets grid, units display, camera defaults, snapping increments, Sancho's size and the default "stage".

| Class | Typical size | Stage | Sancho | Grid / snap |
|---|---|---|---|---|
| `room` | 0.5–10 m | The Space (room shell or a default room box 3.6 × 3 × 2.5 m) | True scale, 1.68 m (5′6″), standing in the room | 100 mm / 1″ snaps, 1 m / 1 ft major |
| `furniture` | 0.3–2.5 m | A floor plane with a rug-sized patch | True scale beside the piece | 10 mm / ¼″ |
| `body` | 0.1–1 m, worn or carried | A mannequin or an "accessory layout" mat | True scale; can "wear" the design (backpack on Sancho's back) | 5 mm / ⅛″ |
| `bench` | 10–500 mm | A **workbench at standing height** (900 mm): the design's floor plane is the bench top, a cutting mat with a 10 mm grid lies under the part, at least 600 × 450 mm | **True scale, standing on the room floor beside the bench**, his feet a bench height below the part (Kris, 2026-11-21; replaces the earlier "giant" Sancho) | 1 mm / 1/32″ |
| `macro` | < 10 mm | A cutting mat with mm grid | A giant eye with a magnifying glass | 0.1 mm |

- The class can be switched any time (View ▸ Stage). Multi-design projects can have different classes per design: the sharpener is `bench`, its box is `bench`, and a display stand might be `furniture`.
- Sancho's height is a real dimension line. Hovering him shows "Sancho is 168 cm (5′6″)". Users can set him to their own height, so it is *their* body in the room.

## 5.6 The first five minutes after "Open"

1. The workspace opens on the stage at the right scale, with the Space visible if one exists.
2. Sancho gives **one** greeting line tied to the intent: "A dresser for the alcove — it's 812 mm wide at the mouth and narrows to 790 at the back. Want to start from the space?"
3. A **starter strip** (dismissible) offers 2–4 ways in, specific to the project:
   - "Draw the footprint on the floor" (Quick Build)
   - "Start from a dresser template" (parametric: width, height, drawers, from the Space)
   - "Sketch your own" (Sketch)
   - "Lay out your boards first" (Stock & Cut, designing from the material)
4. The **Intent card** (top-left chip) and **Inventory drawer** (left) and **Material meter** (status bar) are present from the first second.
5. If a scan was attached but not processed, a **non-blocking job** starts cleaning it, with a progress chip. The user can design against the raw scan, and the room shell appears when ready.

## 5.7 Continuing an existing project

- The project opens to its last state. The journal's last session summary appears for 5 seconds as a toast ("Last time: cut list done, two drawers glued. Next: drawer runners").
- **Inventory drift check**: if the global inventory changed, offer to update the project snapshot (doc 03 §3.6).
- **Deadline check**: if the plan is behind, the plan says so once with options (cut scope, extend, add a work window); Sancho never carries the message (D32). Doc 10 §10.3.
