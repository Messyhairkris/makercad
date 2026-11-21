# What the Fusion 360 pages say, and where we stand

The PM captured the Fusion help at `C:\Users\<you>\Documents\ShareX\Screenshots\2026-11\`.
Those PNGs are the specification. The text is now extracted locally, no cloud, by
`C:\Users\<you>\MakerCAD\tools\ocr_screenshots.py` (Windows' own OCR engine through
`winsdk`):

```
python tools/ocr_screenshots.py "C:\Users\<you>\Documents\ShareX\Screenshots\2026-11"
```

It writes `ocr\<page>.txt` and a combined `ocr\INDEX.md` beside the images. Re-run it when
new screenshots arrive; already-read pages are skipped.

Page numbers below are the screenshot numbers (`Fusion -  (N).png`). Quotes are OCR of the
page, tidied only where OCR clearly misread a character.

## 1. The window (pages 1, 13-17)

Page 1 numbers the whole interface: 1 Data Panel, 2 Application Bar (Home, Data Panel, File,
Save, Undo/Redo, design tabs, New Design, Assistant, Extensions, Job Status, Notification,
Help, Profile), 3 Toolbar (workspace switcher, tabs, contextual tabs and environments),
4 Browser, 5 the assembly in the canvas, 6 ViewCube, 7 Marking Menu, 8 Navigation Bar,
9 Timeline, 10 Assistant access, 11 Assistant panel.

| Part | Ours today |
|---|---|
| Application Bar | ribbon with the same verbs; no design tabs, no profile |
| Toolbar with tabs and contextual tabs | ribbon rows; no tabs, no contextual switching |
| Browser | Browser panel with bodies and sketches |
| ViewCube | four buttons (Home, Top, Front, Right) |
| Marking Menu (right-click) | context menu, now with **Select Other** |
| Navigation Bar | not present; navigation is on the mouse |
| Timeline | timeline panel with a rollback marker |

## 2. Fully defining a sketch (pages 26-45)

The tutorial's own summary, page 26:

> "For sketches to maintain their shape when making any edits ... you must dimension and
> constrain the sketch until you can no longer freely move its entities. When you can no
> longer click and drag any entities in the canvas, the sketch is fully defined. All the
> individual sketch entities change from **blue to black** in colour."

**Correction, 2026-11-14:** this section first said we did not have the sketch state. That was
wrong - Gordon read the palette after writing it and found it already built (`sketcher.rs`
`sketch_panel`: "fully constrained" or "N degrees of freedom"; `viewport.rs`: blue until
`dof == 0`, then the locked colour). The three rules below are the ones to *keep* true as the
sketch tools grow, not to add from scratch.

1. **Every entity has a state**: under-constrained (blue), fully constrained (black),
   over-constrained (refused, with a reason).
2. **Dragging is the definition**: if the solver can move a point, the sketch is not
   defined. So the DOF count is not a decoration, it is the answer.
3. **Finish Sketch is a step**, not a mode you leave by accident.

### The workflow the tutorial teaches (the order matters)

| Step | Page | Ours today |
|---|---|---|
| Workspace switcher, File > New Design | 29 | New project flow exists |
| ViewCube > Top, then click the plane to sketch on | 29 | Look-from buttons + plane pick |
| **S key searches for a tool by name** | 29 | not built (backlog 14) |
| Rectangle > **Center Rectangle**, origin at its centre | 29 | centre rectangle exists |
| Line, snapping to the **midpoints** of the rectangle's sides | 30 | midpoint snap exists |
| Select the line, **Sketch Palette > Options > Construction** (solid becomes dashed) | 30 | X key toggles construction |
| Circle > Center Diameter Circle on the origin | 30 | circle exists |
| Rectangle + two arcs for the cutout, then **Mirror** | 33 | mirror exists (line must be drawn) |
| Mirror dialog: **Objects field with a count (12 objects)**, Mirror Line field, X to cancel, preview, OK | 34 | no dialog; mirror takes the selection |
| Constraints: make the two arcs **equal** so they always stay the same size | 36 | Equal constraint exists |
| Mirror creates **associativity** between the halves: edit one, the other follows | 36 | mirror copies, no association |
| Dimensions: outline height and width, the circle diameter, three on the cutout | 40 | dimension tool exists |
| **After all dimensions, the sketch turns from blue to black** | 40 | **built** - `p.sketch` (blue) until the solver reports 0 DOF, then `p.sketch_locked`; the palette says "fully constrained" or "N degrees of freedom" (`sketcher.rs`, `sketch_panel`) |
| **Show Constraints** checkbox on the Sketch Palette | 44 | constraints always drawn |
| **Finish Sketch** on the palette | 44 | Finish Sketch exists |

## 3. What this changes about our priorities

The PM's complaint was "the model always shows all lines through the object", which the
display modes answer. The tutorial adds one thing of the same weight: **a sketch must say
whether it is finished**. Both are "the program must tell the truth about its own state"
problems, and both are cheap once the state exists.

Order I would build, with a check for each:

1. **Sketch state colours** (blue / black / red) and a palette line that says
   `4 degrees of freedom left` or `fully defined`. Check: the tutorial's outlet sketch,
   built by the same clicks, is black when the last dimension lands and blue one edit
   before.
2. **Drag-to-prove**: any point can be dragged while under-constrained, and drags solve.
   Check: drag a corner of a dimensioned rectangle and it stays a rectangle.
3. **Studio/workspace switcher** and the application bar verbs, so the window reads like
   Fusion's (page 1) without changing behaviour.
4. **S to search tools**, since it is the answer to "where is that tool".
5. **Mirror dialog with a count and a line field** (page 34) and association between the
   halves.

## 4. Honest limits of this note

- OCR drops small labels and misreads symbols (`O15` for `ø15`, `R17m` for `R17`). Where a
  number matters, look at the page.
- Pages 1 and 13-25 are the window; the numbering in the OCR is not contiguous with the
  headings, so search `ocr\INDEX.md` for a phrase rather than trusting the number alone.
- Nothing here has been taken from Fusion itself beyond the vocabulary: the program is not
  copying an implementation, only the workflow a person already knows.
