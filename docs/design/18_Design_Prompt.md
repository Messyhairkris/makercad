# 18 — The Design Prompt: one voice for every icon, panel and feature

**Added 2026-11-19.** The PM: *"Create a prompt to get consistent design results when creating the
icons or other features."* Crew members (and any image model) get **the same preamble every time**
(§18.1), then one of the task prompts (§18.3–18.6). The preamble is the contract; the checks in
§18.7 are how a result is accepted. Change the preamble only by a decision in doc 15, because every
asset made before the change was made against the old one.

---

## 18.1 The preamble (paste verbatim at the top of every design task)

```text
You are drawing for MakerCAD, a CAD program for makers: people who design AND build the thing
themselves, with the tools and materials they actually have. Its look is "Storybook Workshop":
a sunny workshop drawn for a picture book. Warm, soft and friendly on the surface; crisp and
exact wherever there is a number, an edge or a dimension. It must never look like Autodesk
Fusion, SolidWorks, SolidCraft, DraftCraft or any other program: borrow no icon, glyph, layout
signature or colour scheme from them.

THE SIX RULES
1. Soft surfaces, crisp data. Rounded shapes and pastel fills; dimensions, edges and text stay
   sharp, high-contrast and readable.
2. One idea per icon. A single object or action a maker would recognise from a real workshop
   (a saw, a board, a pencil, a clamp, a folded box), not an abstract diagram.
3. Two tones plus one accent. Ink outlines over a pale fill; at most ONE accent-coloured
   element, and only when it marks the action (the part that moves, the cut, the new thing).
4. Thick, round, friendly strokes. Rounded caps and joins everywhere. No hairlines, no sharp
   spikes, no 3D bevels, no gradients, no shadows, no glossy highlights.
5. Readable at 16 px. If it is not recognisable at 16 px, remove detail until it is.
6. Colour is never the only signal. A meaning carried by colour is also carried by shape or
   pattern (a dashed line, an arrow, a symbol).

THE PALETTE (role colours; the app recolours them per theme by exact match)
  ink     #2B2A4C   outlines, text, the "drawn" lines
  paper   #FFFDF7   the main fill of objects
  tint    #CFE3FA   a second, cooler fill for a back face, a shadow side or a background shape
  wood    #E8C9A8   wood, boards, sheet goods
  accent  #FF8A7A   the ONE element that is the action (use sparingly; often none)
  cut     #E5484D   ONLY for a cut line, and always drawn solid
  fold    #3E7BFA   ONLY for a fold or crease, and always dashed
  ok      #2BA471   ONLY for "fits / done / safe"
  warn    #F08C00   ONLY for a warning
No other colours. No transparency except whole-shape opacity 0.5 for a "ghost" (something
shown for reference, like a scan or the before-state).
```

## 18.2 The look in words (for people; the preamble is for models)

- **Mood:** the inside cover of a well-loved how-to book. Kind, capable, a little playful. Never
  childish: a professional uses this to cut real wood.
- **Shapes:** start from rounded rectangles, circles and capsules; corners rounded at 2–4 units
  on the 64 grid. Objects sit slightly toward the viewer (a 3/4 view is allowed for objects like
  boxes and boards, never true perspective).
- **Metaphors:** real workshop objects first (doc 11 §11.5). An action is the object plus a small
  ink motion mark (an arc, a short arrow, a spark), not a separate arrow icon bolted on.
- **Family resemblance:** every icon in an environment shares one recurring element (Stock & Cut:
  the board with a grain line; Fold: the dashed fold line; Sketch: the pencil tip).

## 18.3 Task prompt: one icon (SVG code)

```text
TASK: draw the icon `{id}` for the command "{label}".
What it does: {one sentence in the user's words}
Environment: {environment} (family element: {family element})
Metaphor to use: {metaphor}

OUTPUT: exactly one SVG document and nothing else, following these rules:
- <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"> with no width or height.
- Everything inside the safe area x 4..60, y 4..60 (the outer 4 units stay empty).
- Elements allowed: path, circle, ellipse, rect, line, polyline, polygon, g. Nothing else:
  no text, no image, no use, no defs, no gradient, no filter, no mask, no style, no script,
  no transform, no class, no id.
- Path data in absolute commands only: M L H V C Q A Z.
- Every outline: stroke="#2B2A4C" stroke-width="4" stroke-linecap="round"
  stroke-linejoin="round". Detail lines inside a shape may use stroke-width="3".
- Fills only from the palette roles. fill="none" for lines.
- At most 12 elements. At most one accent (#FF8A7A) element.
- Coordinates on whole or half units (e.g. 12 or 12.5), never more decimals.
```

## 18.4 Task prompt: an icon set (for a crew code task)

The crew's gates judge code, so an icon set is delivered as a Python module whose only content is
data. That lets the existing hidden-test harness (§18.7) check every icon automatically.

```text
TASK: write `icons_{set}.py`, a Python module with exactly one public name:

    ICONS: dict[str, str]   # icon id -> one SVG document as a string

It contains these {n} icons (id, label, what it does, metaphor):
{table}

Each SVG follows the icon rules below exactly. The module imports nothing and defines nothing
else. Use plain string literals (no f-strings, no concatenation, no functions).
{the rules from 18.3}
```

## 18.5 Task prompt: a panel, dialog or feature (layout as data)

Modules ship no UI code (doc 16 §16.3), and neither does a design task. A panel is described as
data that the shell draws.

```text
TASK: design the panel "{name}" for the environment {environment}.
Who uses it and when: {situation}
What it must let them do: {numbered list of actions}
Commands it calls: {command ids with their JSON schemas}

OUTPUT: one JSON object:
{ "id", "title", "width": "narrow|normal|wide",
  "sections": [ { "title", "rows": [ {"kind": "field|choice|toggle|meter|list|button|note",
                                      "label", "bind": "<command arg or state path>",
                                      "hint": "<one short line in the user's words>"} ] } ],
  "primary": "<the one button that matters>", "secondary": ["..."] }

RULES: the words are plain and kind ("Board length", not "L_stock"); every number shows its
unit; the panel works at 1366x768 (narrow = 280 px); no more than 7 rows before a section break;
one primary button; destructive actions are never primary; every field has a hint.
```

## 18.6 Prompt for image models (concept art, Sancho, illustrations)

For pictures that are not icons: the opening-screen illustration, empty-state drawings, Sancho
concept sheets. Paste the preamble, then:

```text
A {subject}, drawn as a flat storybook illustration: thick rounded ink outlines (#2B2A4C),
soft pastel fills from the palette, a warm cream background (#FFF8EC), no gradients, no
photorealism, no 3D rendering, no text, no logos, no trademarks. Friendly and capable, like the
illustrations in a good woodworking book for beginners.
```

**Sancho v1** (doc 16 §16.11): "a simple wooden artist's mannequin with round joints and
expressive jointed fingers, standing like a helpful workshop assistant; a soft cream body with
ink outlines, a tape measure clipped at the waist, no face except two dot eyes". He is generated
as a rigged model by code; the illustration is only a guide for proportions and mood.

## 18.7 How a design is accepted (gates a member cannot argue with)

| Check | Tool | Fails when |
|---|---|---|
| SVG subset | `tools/icon_lint.py` | an element, attribute, command or colour outside the rules; anything outside the safe area; more than 12 elements; more than one accent |
| Renders at every size | `tools/icon_render.py` (resvg) | it fails to render at 16, 24, 32 or 64 px |
| Reads at 16 px | Gordon (vision member) on the 16 px sheet, then a person | it isn't recognisable as its label |
| Family resemblance | a person, on the environment's contact sheet | it looks like it belongs to another set or another program |
| No borrowing | a person, against `reference/` | it resembles another program's icon |

The first two are automatic and are the crew task's hidden tests. The last three happen on the
Review Desk, with the contact sheet attached.

## 18.8 The first icon set (P1 shell, 60 icons)

Grouped as the toolbox will show them. Metaphors are starting points; the member may improve one
if the label still reads.

| Group | Icons (id: metaphor) |
|---|---|
| **Shell (10)** | `app.new`: a sparkle over a blank card · `app.open`: an open folder with a box in it · `app.save`: a notebook with a ribbon bookmark · `app.undo` / `app.redo`: a curved arrow on a pencil line · `app.search`: a magnifying glass over a small saw · `app.settings`: a gear made of rounded teeth · `app.help`: a speech bubble with a question mark · `app.sancho`: a mannequin head with two dot eyes · `app.workshop`: a pegboard with three tools |
| **Project (8)** | `project.intent`: a heart on an index card · `project.inventory`: a crate of boards · `project.space`: a room corner with a tape measure · `project.deadline`: a calendar page with a pin · `project.budget`: a jar with a coin · `project.inspiration`: a corkboard with a pinned photo · `project.journal`: an open notebook with a pencil · `project.export`: a box with an outward arrow |
| **Sketch (12)** | `sketch.line`, `sketch.rectangle`, `sketch.circle`, `sketch.arc`, `sketch.polygon`, `sketch.slot`, `sketch.trim` (scissors cutting a line), `sketch.offset` (two parallel lines with a gap arrow), `sketch.fillet` (a rounded corner), `sketch.mirror` (a dashed axis with a shape on each side), `sketch.dimension` (a dimension line with arrowheads and a pill), `sketch.finish` (a pencil with a check) |
| **Solid (10)** | `solid.extrude` (a profile rising into a block), `solid.revolve`, `solid.hole` (a drill bit into a block), `solid.fillet`, `solid.chamfer`, `solid.shell`, `solid.combine`, `solid.pattern`, `solid.measure` (calipers), `solid.section` (a block with a cut face) |
| **Stock & Cut (8)** | `stock.board` (a board with grain), `stock.sheet` (a sheet with a corner lifted), `stock.assign` (a part dropping onto a board), `stock.layout` (three parts on a board), `stock.meter` (a board with a fill bar), `stock.offcut` (a short board with a green check), `stock.shopping` (a tote bag), `stock.kerf` (a saw tooth line) |
| **Fold & Package (6)** | `fold.crease` (a dashed fold line on a card), `fold.cut` (a solid red line on a card), `fold.unfold` (a box opening flat), `fold.box` (a closed carton), `fold.flap`, `fold.animate` (a box half-folded with a motion arc) |
| **Plan & Build (6)** | `plan.steps` (a numbered checklist), `plan.timeline` (bars on a calendar), `plan.board` (three columns of cards), `build.projector` (a projector beam on a table), `build.photo` (a camera), `build.compare` (two overlapping outlines) |
