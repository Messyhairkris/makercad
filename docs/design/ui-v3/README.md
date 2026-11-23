# UI pass 3: the screens as built (for FigCraft)

Fifteen **real screenshots** of the running program, taken on 2026-11-23 at the pause after the
first project-layer work. Pass 1 (`../ui-v1`) set the look and the opening screen; pass 2
(`../ui-v2`) drew six screens of the Design workspace *in use* as editable SVG; this pass is the
other half: **what actually exists today**, so notes and redesigns in FigCraft start from the real
thing and not from a picture of what we hoped for.

They are PNGs, not SVGs: the program draws them, and an image is exactly what it drew. Import them
into FigCraft as images and annotate, or redraw them as frames (pass 2's SVGs are the editable
starting point for the Design workspace in use).

| File | Screen | What is real |
|---|---|---|
| `01-home.png` | Opening screen, Daydream look | Real. The workshop line counts a sample library |
| `02-home-night.png` | Opening screen, Night Workshop | Real |
| `03-home-pastel.png` | Opening screen, Pastel | Real |
| `04-design.png` | Design workspace: SolidCraft's sample plate on a bench, Sancho 1.68 m beside it | A viewer: the toolbox has only view buttons, no modelling tools yet |
| `05-command-search.png` | Ctrl+K with "round" typed | Real search; most commands say which phase brings them |
| `06-stock-tab.png` | An environment that is not built (Stock & Cut) | A placeholder page |
| `07-quest-name.png` | "Start something new", step 1 | Real. The rail, the question, and "What your answers set up" |
| `08-quest-size.png` | Quest step 3: a size typed in feet and inches, shown in millimetres | Real (D34) |
| `09-quest-tools.png` | Quest step 4: kits, your workshop's tools (filled) and this project's own (outlined) | Real (D30) |
| `10-quest-materials.png` | Quest step 5 | Real |
| `11-quest-review.png` | Quest step 9, the review | Real |
| `12-workshop-tools.png` | My workshop, "Your tools" | Real (a sample library) |
| `13-workshop-days.png` | My workshop with the "when can you use it?" form open | Real (D31) |
| `14-finish-offer.png` | The end-of-project offer over the Design workspace | Real; reached by Ctrl+K, "Finish the project" |
| `15-findings.png` | The Findings panel with a "Why?" open | A **demonstration** on a built-in dresser, not your design |

Every picture is 1440 px wide (taken in a 1920 × 1017 window), 256 colours, so the folder stays
small. Regenerate them with `pwsh tools/ui3_screens.ps1` then `python tools/ui3_prepare.py`.

## Questions, by screen

The pause-point test pass (`docs/test-passes/Test pass - MakerCAD pause point - 2026-11-23.html`,
sections 02 and 05) asks the same things with room to answer. In short:

### 1 to 3 · Opening screen and the three looks
1. First word for the feeling? Warm and soft, a workshop, a game, something else?
2. **Start something new** and **Keep going**: right order once a project exists?
3. Sancho: size, place, and what he says. Should it change each visit?
4. Which look would you use all day? Is Pastel different enough from Daydream? A high-contrast look is
   planned: where should it sit?

### 4 · Design workspace
1. Does an empty toolbox read as "not built yet" or as "broken"?
2. The bench and the floor patch with Sancho beside the part: right idea?
3. The first modelling tool you would want in that toolbox.

### 5 · Command search
1. Commands that do not work yet are listed with the phase that brings them: useful, or noise?
2. "Ask Sancho" as the last row.

### 6 · An unbuilt environment
1. Pages that say "comes in P3": keep them (the shape of the program is visible), or hide the tabs
   until they work?

### 7 to 11 · The Quest
1. One question per page, or the whole form at once?
2. The right-hand "What your answers set up" panel: useful, or too much? Shown all the way through, or
   only at the review?
3. **Skip to designing** and **Next** are both coral: which should be loud?
4. Two kinds of chip (workshop: filled, project only: outlined): clear at a glance?
5. Kits first, or a short checklist of everyday tools?

### 12 and 13 · My workshop
1. Would you keep this up to date? What would make you open it?
2. Days and notes on a tool ("weekends", "at Sam's"): the right words and shape?

### 14 · Finishing a project
1. A button on the project instead of a command? Where would you look for it?

### 15 · Findings
1. Is this how you want a list of problems to look (worst first, a plain fix, a "Why?")?
2. Sancho is not the channel for alerts (D32): rename it plainly "Findings", or keep his name?

## Beyond these screens

Screens that do not exist yet are the cheapest place to decide the feel: **Stock & Cut, Plan, Build,
Space, Share**, a part's material picker, the toolbox when a tool is chosen, and the Intent card.
Draw those in FigCraft whenever you like; nothing has to be undone.

## Sending notes back

Save the `.figcraft` file in `docs/design/ui-v3/notes/` (or on the share). Claude reads it directly
with `python tools/figcraft_read.py FILE.figcraft [--json]`: pages, frames, text, comments,
annotations and prototype links (decision D29, still only proposed). That reader has been tested on a
file built from FigCraft's own schema, **never on a real FigCraft file**: if yours does not read, say
so and send FigCraft's REST export instead.
