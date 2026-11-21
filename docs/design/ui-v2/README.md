# UI pass 2: the Design workspace in use (for FigCraft)

Six screens of MakerCAD's Design workspace in use, after Kris asked on 2026-11-21 for more UI
to make notes and decisions on in FigCraft. Pass 1 (`../ui-v1`) set the look and the opening
screen; this pass is about **using the tool**: sketching, pulling up, searching, changing a step,
and Sancho with the new workbench stage.

| File | Screen |
|---|---|
| `01-design-bench.svg` | The workspace with a small part on the bench and Sancho beside it (MakerCAD's own render) |
| `02-sketch-on-a-face.svg` | Drawing a slot on the top face, sizes typed at the cursor |
| `03-pull-up.svg` | Pull up (extrude) with a live preview, a drag arrow and a stock check |
| `04-command-search.svg` | Ctrl K: plain-language command search |
| `05-change-a-step.svg` | Changing "Round over" from the timeline, with a tip from your toolbox |
| `06-sancho-stage.svg` | Sancho's settings: who stands there, height, stage, measurements, pose |

Each screen is 1440 × 900 and comes as an **SVG** (import into FigCraft: every text, shape and
icon stays editable) and a **PNG** (to look at anywhere). The coral numbers are the questions
below, in their own group named `callouts`: hide it to see the bare screen.

The layout follows doc 11 §11.4: project bar, environment tabs instead of a ribbon, a short toolbox
for this project, the context panel, the timeline in maker words, and the status bar with the
material meter. The icons are the app's own (`apps/mk-desktop/assets/icons`), except ten still in
rework with the Crew (`mk-icons-4`), shown here as their first drafts. Screens 2–5 draw the part
as simple vectors; screens 1 and 6 show what MakerCAD itself draws.

Regenerate: `python docs/design/ui-v2/make_screens.py --render-dir <folder with view-bench.png> --png`.

## Questions, by screen and number

### 1 · The part on the bench
1. **Project bar**: name, what you said you want (the intent), the deadline, search, undo/redo,
   Sancho, settings. Is the intent worth a permanent line?
2. **Intent pill**: always shown, or folded away after the first day?
3. **Toolbox**: short and per project (view, sketch, make for this part), icons with words under
   them. Words, or icons only once you know them?
4. **The stage**: a small part sits on a workbench at 900 mm with Sancho on the floor beside it
   (your direction of 2026-11-21). Is the bench the right size and colour? Should the cutting mat's
   10 mm grid be the snap grid?
5. **Context panel**: the selection's size (mm and inches), material from your rack, "size against
   Sancho", findings. Which of these earn their place?
6. **Timeline** in maker words ("Pull up", "Round over"), with CAD words one click away. A strip
   along the bottom, or a list down the left like most CAD programs?
7. **Status bar**: material meter, fit, budget, units, Sancho's height. Keep all of it?

### 2 · Sketching on a face
1. While sketching, the toolbox **swaps to sketch tools**. Swap, or keep the toolbox and add a
   sketch bar?
2. Sizes are **typed at the cursor** while drawing (the 40 mm box). Enough, or also in the panel?
3. The view **stays 3D** and you draw on the face. Or turn to look straight down when a sketch starts?
4. **"How fixed is it?"** says "2 sizes to go" instead of "2 degrees of freedom". Right words?
5. **Finish / Cancel** at the bottom of the panel (and Enter / Esc). Right place?

### 3 · Pull up
1. The **drag arrow** with the distance at its tip.
2. **Which way**: Up / Down / Both. And "What it does": Add / Cut / New part. Clear?
3. A **finding checks your stock while you drag** (the 19 mm board). Too early, or just right?
4. The main button **names the action** ("Pull up"), not "OK". Keep that everywhere?
5. **Sancho compares sizes** ("a little less than my thumb is wide"). Helpful, or too chatty? How often?

### 4 · Command search
1. **Plain language** ("put a hole for a screw").
2. Commands from **environments that aren't loaded** show up and offer to load them.
3. Each result says **where it lives** (Make, Stock & Cut).
4. **"Ask Sancho"** as the last row: explains using your part.

### 5 · Changing a step
1. **Rolling back**: later steps fade, with a coral marker after the step being changed.
2. The **live preview** while you type the radius.
3. A **tip from your toolbox**: your router's ⅛ in round-over bit is 3.2 mm. Offer the nearest real tool?
4. **Done rebuilds** the later steps and says whether anything broke.

### 6 · Sancho and the stage
1. **Who stands there**: the placeholder, your own figure, or a scan of you (D27).
2. **Height**, metric and imperial.
3. **Stage**: Auto (bench up to 500 mm, floor above), Bench, Floor, Room.
4. **Measure with him**: height, hand length, finger width, reach and eye height. Which by default?
5. **Pose**: Stand / Lean in / Hold it. Which poses matter first?

## Sending notes back

Make your notes and decisions in FigCraft and save the `.figcraft` file (put it in
`docs/design/ui-v2/notes/`, or on the share). Claude reads `.figcraft` files directly: they are a
zip of `document.json` and images, so pages, frames, text, comments, annotations and prototype
links come through as they are (decision D29, proposed).
