# MakerCAD --- Human-Centered Professional CAD Workspace & UX Specification

**Document status:** Accepted Product Direction / Design Baseline\
**Date:** 2026-08-12\
**Product:** MakerCAD\
**Primary goal:** Make professional-grade 3D modeling accessible and
anxiety-free for non-CAD designers while preserving the operational
grammar, shortcuts, navigation, precision, history, and predictability
expected by professional CAD users.

------------------------------------------------------------------------

## 1. Executive Product Decision

MakerCAD is **not** a traditional CAD application with a simplified
"Beginner Mode."

It is a **goal-oriented spatial design environment built on a
professional parametric CAD foundation**.

The product must support two simultaneous truths:

1.  A non-CAD user should be able to begin with an idea such as:
    -   "I want to build a desk."
    -   "I need a cabinet to fit this wall."
    -   "I need this object to fit in this scanned room."
    -   "I have these materials and these tools."
2.  A professional user must be able to operate the same system using
    familiar CAD conventions:
    -   selection
    -   sketches
    -   constraints
    -   features
    -   bodies/components
    -   dimensions
    -   parametric history
    -   timeline
    -   keyboard shortcuts
    -   right-click contextual menus
    -   marking menus
    -   precise numeric input
    -   standard viewport navigation
    -   Enter to commit
    -   Escape to cancel
    -   Ctrl+Z to recover

The central design principle is:

> **Traditional CAD is the engine language. MakerCAD's project workflow
> is the human language.**

The user should be able to approach the same underlying operation
through:

``` text
Professional:
E → 25 → Enter

Beginner:
"Make this board 25 inches thick."

Direct manipulation:
Drag the face → type 25
```

All three should produce the same deterministic parametric operation.

------------------------------------------------------------------------

# 2. Core Product Architecture

MakerCAD should be understood as six conceptual layers:

``` text
USER
  ↓
INTENT
  ↓
PROJECT CONTEXT
  ↓
WORKFLOW
  ↓
CAD COMMANDS
  ↓
PARAMETRIC MODEL / GEOMETRY
```

### Layer 1 --- User Intent

What is the person trying to accomplish?

Examples:

-   Build a desk.
-   Design a replacement bracket.
-   Animate a character.
-   Create a game asset.
-   Fit an object into a scanned environment.

### Layer 2 --- Project Context

What constraints surround the task?

Examples:

-   environment
-   dimensions
-   materials
-   inventory
-   budget
-   available tools
-   workspace limitations
-   desired output
-   target manufacturing process
-   imported scans/splats/models
-   preferred units

### Layer 3 --- Workflow

What sequence of activities makes sense for this project?

Example:

``` text
Understand space
→ inventory materials
→ inventory tools
→ rough design
→ validate fit
→ refine dimensions
→ optimize material usage
→ create build plan
→ produce instructions
```

### Layer 4 --- CAD Commands

The workflow resolves to ordinary professional CAD operations:

-   Sketch
-   Line
-   Rectangle
-   Dimension
-   Constraint
-   Extrude
-   Revolve
-   Fillet
-   Chamfer
-   Hole
-   Pattern
-   Mirror
-   Move
-   Combine
-   Split
-   Offset
-   Project
-   Measure
-   Create Component
-   Assemble
-   etc.

### Layer 5 --- Parametric Model

The design remains:

-   editable
-   deterministic
-   history-based
-   constraint-aware
-   dimensionally precise
-   recoverable

### Layer 6 --- Geometry / Kernel

The final geometric representation is owned by the CAD/modeling system,
not by the AI.

**AI must never become the authoritative owner of geometry.**

------------------------------------------------------------------------

# 3. Product UX Principle: Forgiving Surface, Rigorous Core

MakerCAD should have:

> **A forgiving surface + a rigorous professional core.**

The surface may:

-   simplify language
-   infer intent
-   offer suggestions
-   automatically apply safe constraints
-   explain terminology
-   provide previews
-   identify conflicts
-   suggest repairs
-   guide users through workflows

The core must remain:

-   deterministic
-   explicit
-   undoable
-   inspectable
-   parametric
-   scriptable
-   compatible with expert workflows

AI may:

-   interpret
-   explain
-   suggest
-   preview
-   calculate
-   identify
-   guide

AI must not silently:

-   change consequential geometry
-   delete history
-   alter constraints
-   rewrite the user's design
-   change units
-   change references
-   apply an optimization without clear user intent

------------------------------------------------------------------------

# 4. Traditional CAD Interaction Baseline

Fusion is the primary professional UX reference for MakerCAD's
mechanical interaction grammar.

Autodesk's current Fusion interface documentation identifies the
following major workspace regions:

-   workspace selector
-   tabs
-   contextual tabs/environments
-   Browser
-   Canvas
-   ViewCube
-   Marking Menu
-   Navigation Bar
-   Timeline
-   Autodesk Assistant access/panel

This is the baseline MakerCAD should preserve conceptually while
improving its organization and contextual behavior.

Reference: Autodesk Fusion interface documentation:
https://help.autodesk.com/view/fusion360/ENU/?contextId=LP-STEPS-P13N-SNP-GS-OTH-CRD-1

------------------------------------------------------------------------

# 5. Traditional Selection Behavior

Selection is a foundational professional CAD contract.

Fusion currently supports:

-   canvas selection
-   Browser selection
-   Timeline selection
-   Ctrl additive/removal selection
-   Shift additive selection
-   Window selection
-   Crossing selection
-   Freeform selection
-   Paint selection
-   selection filters
-   selection priorities
-   Select Through
-   Select by Name
-   Select by Boundary
-   Select by Size
-   Invert Selection
-   Selection Sets

Reference:
https://help.autodesk.com/cloudhelp/ENU/Fusion-Model/files/SLD-SELECTION.htm

## Required MakerCAD behavior

### Individual selection

Left-click selects the object under the pointer.

### Add/remove selection

`Ctrl + click`

-   add object
-   remove object

### Add-only selection

`Shift + click`

Should remain available for users accustomed to Fusion's behavior.

### Window selection

Drag **left → right**.

Select objects completely enclosed by the boundary.

### Crossing selection

Drag **right → left**.

Select objects enclosed by or crossing the boundary.

### Visual feedback

Selection boundaries must communicate the difference visually.

``` text
Left → Right

┌──────────────┐
│   OBJECT     │  ← fully enclosed
└──────────────┘
```

``` text
Right → Left

┌──────────────┐
│────OBJECT────│  ← crossing boundary
└──────────────┘
```

### Beginner assistance

Briefly display:

> **Window selection:** selects objects completely inside.

> **Crossing selection:** selects objects touched by the boundary.

This guidance should disappear as the user demonstrates competence.

------------------------------------------------------------------------

# 6. Right-Click and Marking Menu

Fusion uses a right-click Marking Menu on the canvas. Frequently used
commands appear in the radial wheel, with additional commands in an
overflow menu.

MakerCAD should retain this pattern.

## Required behavior

Right-click should expose context-sensitive commands based on:

-   current selection
-   current modeling environment
-   active tool
-   object type
-   workflow context

Example for a selected part:

``` text
              Edit
                ↑
        Hide  ○  Move
                ↓
             Delete
```

Example for an active sketch:

``` text
              Finish
                ↑
        Trim   ○  Offset
                ↓
              Cancel
```

The Marking Menu should never attempt to expose every available command.

It should expose the **most likely next actions**.

------------------------------------------------------------------------

# 7. Keyboard Shortcut Compatibility

MakerCAD must preserve professional single-key modeling shortcuts as a
hidden expert layer.

Fusion's documented shortcuts include, among others:

  Shortcut   Function
  ---------- ------------------------
  `E`        Extrude
  `H`        Hole
  `F`        Fillet
  `M`        Move
  `V`        Toggle visibility
  `A`        Appearance
  `J`        Joint
  `L`        Line
  `R`        Two-point rectangle
  `C`        Center-diameter circle
  `T`        Trim
  `O`        Offset
  `I`        Measure
  `P`        Project
  `X`        Normal/Construction
  `D`        Sketch Dimension
  `1`        Window selection
  `2`        Freeform selection
  `3`        Paint selection
  `Del`      Delete

Reference:
https://help.autodesk.com/view/fusion360/ENU/?guid=GUID-F0491540-0324-470A-B651-2238D0EFAC30

## MakerCAD rule

Do not make beginners learn these.

Make them discoverable.

Example:

``` text
Extrude

Turn this flat shape into a 3D solid.

Shortcut: E
```

Repeated use should gradually reduce the amount of instructional UI.

Expert:

``` text
E
```

Beginner:

``` text
[Extrude]
Turn this shape into a solid.
Shortcut: E
```

Same command system.

------------------------------------------------------------------------

# 8. Navigation Compatibility

Fusion's documented viewport conventions include:

-   MMB = pan
-   mouse wheel = zoom
-   Shift + MMB = orbit
-   Shift + click + MMB = orbit around a point
-   Ctrl+Z = undo
-   Ctrl+Y = redo

MakerCAD should preserve these conventions wherever possible.

These are not merely UI details. They are muscle memory.

------------------------------------------------------------------------

# 9. Enter / Escape / Undo Contract

These are sacred interaction contracts.

## Enter

> Accept, commit, or advance the current input.

## Escape

> Safely cancel/exit the current command or active state.

## Ctrl+Z

> Undo the last meaningful operation.

## Ctrl+Y

> Redo.

The beginner should eventually internalize:

> **Escape is the panic button.**

The system should make it safe to experiment.

------------------------------------------------------------------------

# 10. MakerCAD Workspace Window --- Primary Layout

The default MakerCAD workspace should be based on the familiar structure
of Fusion while adding a project-context layer.

Recommended layout:

``` text
┌───────────────────────────────────────────────────────────────────────────┐
│ MAKERCAD │ File │ Edit │ View │ Project │ Design │ Inspect │ Help         │
├───────────────────────────────────────────────────────────────────────────┤
│ Project / Workflow Selector │ Workspace │ Search │ Undo │ Redo │ Save    │
├───────────────────────────────────────────────────────────────────────────┤
│ Select │ Sketch │ Create │ Modify │ Assemble │ Inspect │ Project Tools   │
├──────────────┬─────────────────────────────────────────────┬──────────────┤
│              │                                             │              │
│ PROJECT      │                                             │ CONTEXT      │
│ BROWSER      │                                             │ / TASK       │
│              │                                             │ PANEL        │
│ Environment  │                                             │              │
│ Components   │                                             │ Tool         │
│ Bodies       │                 3D CANVAS                   │ properties   │
│ Sketches     │                                             │              │
│ Features     │                                             │ Guidance     │
│ References   │                                             │              │
│              │                                             │              │
├──────────────┴─────────────────────────────────────────────┴──────────────┤
│ Timeline / History / Build History                                         │
├───────────────────────────────────────────────────────────────────────────┤
│ Navigation │ Selection │ Units │ Model Health │ Command State │ Assistant │
└───────────────────────────────────────────────────────────────────────────┘
```

This is the **default professional-capable workspace**.

------------------------------------------------------------------------

# 11. Workspace Region Specifications

## 11.1 Application / Project Header

Topmost region.

Must contain:

-   MakerCAD identity
-   project name
-   project status
-   save state
-   file/version state
-   undo/redo
-   global search
-   project/workflow context
-   application-level menus

Example:

``` text
MakerCAD
Custom Desk Project
● Saved

[Undo] [Redo] [Search]
```

The user must always know which project is open.

------------------------------------------------------------------------

# 12. Workspace Selector

The workspace selector should remain familiar to CAD users.

Example:

``` text
WORKSPACE

Design
Sketch
Surface
Assembly
Manufacture
Animation
Render
Drawing
```

However, MakerCAD should also allow workflow-driven organization.

For the maker:

``` text
PROJECT WORKFLOW

Understand Space
Design
Materials
Build
Document
Share
```

These are **workflow layers**, not replacements for the underlying
technical workspaces.

------------------------------------------------------------------------

# 13. Tabs / Toolbar

The toolbar is the main professional command surface.

Fusion organizes workspace commands into logical tabs and supports
contextual tabs/environments.

MakerCAD should use the same conceptual structure.

Recommended base tabs:

``` text
Select
Sketch
Create
Modify
Assemble
Inspect
Construct
Manage
```

Contextual tabs can appear for:

-   Sketch
-   Surface
-   Mesh
-   Assembly
-   Animation
-   Drawing
-   Manufacturing

## Rule

Never remove professional commands from the underlying command system
merely because they are absent from the current contextual toolbar.

------------------------------------------------------------------------

# 14. Dynamic Project Toolbox

This is one of MakerCAD's major innovations.

The project interpreter should identify likely tools from:

-   project description
-   uploaded files
-   materials
-   objects
-   desired output
-   user vocabulary
-   workflow history

Example:

``` text
YOUR PROJECT TOOLS

⭐ Always Available
  Select
  Move
  Measure
  Undo

📌 Recommended
  Environment Scan
  Parametric Boards
  Materials
  Hardware Library
  Fit / Clearance
  Cut List
  Build Planner

＋ All Tools
```

## Tool selection confidence

The system should internally track confidence.

Example:

``` text
Environment Scan       98%
Material Inventory     97%
Parametric Boards      94%
Hardware Library       91%
Photogrammetry         61%
Advanced Surface       34%
```

The user should not be forced to accept the tool recommendations.

------------------------------------------------------------------------

# 15. Browser / Project Tree

The Browser is a professional requirement.

Fusion uses the Browser for objects such as:

-   components
-   bodies
-   sketches
-   origins
-   joints
-   construction geometry
-   visibility

MakerCAD must retain this functionality.

However, MakerCAD should add a **semantic layer**.

Traditional:

``` text
Body1
 ├─ Sketch1
 ├─ Extrude1
 ├─ Fillet1
 └─ Hole1
```

Maker-friendly:

``` text
Desk Top
 ├─ Desktop Sketch
 ├─ Desktop Thickness
 ├─ Rounded Edges
 └─ Drawer Holes
```

Both names should remain accessible.

Recommended object metadata:

``` text
Friendly Name: Desktop Thickness
Technical Name: Extrude001
Type: Extrude
Parent: Desk Top
```

------------------------------------------------------------------------

# 16. Project Graph

The Browser should optionally expose a higher-level project graph.

Example:

``` text
CUSTOM DESK
│
├── Environment
│   ├── Room Scan
│   ├── Wall A
│   └── Floor
│
├── Materials
│   ├── 3/4" Plywood
│   └── Reclaimed Board
│
├── Hardware
│   └── Vintage Drawer Pull
│
├── Design
│   ├── Desktop
│   ├── Frame
│   └── Drawers
│
└── Build
    ├── Cut List
    ├── BOM
    └── Instructions
```

This is not a replacement for the professional Browser.

It is a second interpretation layer.

------------------------------------------------------------------------

# 17. Canvas

The Canvas is the primary interaction area.

The user must be able to:

-   select
-   draw
-   drag
-   rotate
-   orbit
-   pan
-   zoom
-   place geometry
-   edit dimensions
-   inspect geometry
-   manipulate references
-   see previews
-   interact with scans/splats
-   interact with contextual assistant elements

The Canvas should always remain the dominant visual region.

------------------------------------------------------------------------

# 18. Human Scale / Assistant Character

The MakerCAD assistant character is a contextual guide, not a
replacement UI.

Default character:

-   approximately 5'6"
-   always rendered at correct scale
-   spatially positioned relative to the project
-   capable of walking/positioning itself around the environment
-   capable of demonstrating physical scale

The character may explain:

-   what a tool does
-   what a measurement means
-   why a design may be unsuitable
-   how to use a project tool
-   what the next workflow step is

The character must never constantly interrupt professional users.

Modes:

``` text
Assistant

● Active
○ Helpful only
○ Silent
○ Hidden
```

The assistant should learn when guidance is no longer needed.

------------------------------------------------------------------------

# 19. ViewCube

MakerCAD should retain a ViewCube-style navigation control.

Primary functions:

-   Top
-   Bottom
-   Front
-   Back
-   Left
-   Right
-   corner/isometric views
-   click-to-orient
-   orbit by dragging

The ViewCube should remain in a predictable upper-right position.

It should not be replaced by a novel navigation system.

Enhancements:

-   show current orientation clearly
-   support hover previews
-   allow "look at selected"
-   expose standard views through keyboard shortcuts
-   optionally display human/environment orientation when working in
    scans

The ViewCube is a professional familiarity anchor.

------------------------------------------------------------------------

# 20. Navigation Bar

The navigation bar should occupy a predictable region adjacent to the
lower/right side of the Canvas.

It should contain:

-   orbit
-   pan
-   zoom
-   fit
-   look at
-   display settings
-   grid
-   snapping
-   camera/projection controls
-   viewport display modes

Fusion documentation identifies the Navigation Bar as the location for
zoom/pan/orbit and display settings.

MakerCAD should preserve the concept while making controls discoverable.

------------------------------------------------------------------------

# 21. Display / Viewport Controls

MakerCAD should provide familiar display modes such as:

-   shaded
-   shaded with edges
-   wireframe
-   hidden edges
-   x-ray/transparency
-   realistic/material view
-   technical/inspection view

For beginners, labels should be human-readable.

Example:

``` text
View

Realistic
Solid
Solid + Edges
Wireframe
Transparent
Technical
```

Professional terminology may be shown as secondary labels.

------------------------------------------------------------------------

# 22. Selection Pre-Highlight

Before clicking an object, hovering over it should provide subtle
preselection feedback.

The user should understand:

> "This is what I am about to select."

This is particularly important for:

-   small faces
-   edges
-   sketch entities
-   splat references
-   crowded assemblies

------------------------------------------------------------------------

# 23. Command Halo / Context HUD

When a tool is active, the Canvas should communicate the current state.

Example:

``` text
                    LINE

                 42.5 mm
                    │
                    ●───────────────→
                       0°

        Click: place point
        Enter: finish
        Esc: cancel
```

This is the bridge between beginner guidance and professional dynamic
input.

------------------------------------------------------------------------

# 24. Dynamic Input

MakerCAD should prioritize on-canvas numeric input.

The user should not need to leave the model to type values.

Example:

``` text
Length      42.5 mm
Angle        90°
```

Typing a bare number should be interpreted according to current context.

The user should also be able to type expressions where appropriate:

``` text
3/4
2 * 12
30 - 1/8
```

Unit parsing should be explicit and safe.

------------------------------------------------------------------------

# 25. Units

Project units should be established near the beginning of a project.

MakerCAD should support:

-   inches
-   feet
-   millimeters
-   centimeters
-   meters
-   mixed input
-   unit expressions

A user can set:

``` text
Project Units: Inches
```

and still enter:

``` text
19 mm
```

The application converts it safely.

------------------------------------------------------------------------

# 26. Environment Calibration

Imported environments must not be blindly trusted.

MakerCAD should support:

1.  choose known dimension
2.  select two endpoints
3.  enter actual measurement
4.  calibrate environment

Example:

``` text
CALIBRATE ENVIRONMENT

Select two points with a known real-world distance.

Measured:
43.2"

Actual:
[42.0"]

[Calibrate]
```

Once calibrated:

``` text
✓ Environment calibrated
```

All dependent spatial measurements inherit the calibration.

------------------------------------------------------------------------

# 27. Splat / Scan Reference World

Splat data should not be treated as failed CAD geometry.

It is **spatial evidence**.

MakerCAD should distinguish:

``` text
REFERENCE WORLD
----------------
Splat
Scan
Photos
Measurements
Existing mesh

DESIGN WORLD
----------------
Sketches
Bodies
Components
Features
Parametric geometry
```

Both worlds exist simultaneously.

------------------------------------------------------------------------

# 28. Splat Measurement Probe

The user should be able to select two visible points within a splat/scan
and obtain a measurement.

Example:

``` text
●────────────────●
        43.2"
```

Then:

``` text
Is this a known measurement?

[Use 43.2"]
[Correct to 42"]
```

This creates a bridge between visual reference and parametric design.

------------------------------------------------------------------------

# 29. Primitive-Based Rough Design

Non-CAD makers should be allowed to begin with primitives.

Example:

``` text
DESK STARTER

Width:  60"
Depth:  24"
Height: 30"

[Create Starter]
```

The user can manipulate the primitive immediately.

They should not be required to understand sketches first.

Later, MakerCAD can expose:

> This shape is controlled by a parametric feature.

------------------------------------------------------------------------

# 30. Parametric Controls

Parametric editing should be understandable in both beginner and expert
forms.

Beginner:

``` text
Desktop thickness
[ 3/4" ]

Why this matters:
Matches available plywood.
```

Professional:

``` text
Extrude001
Distance = 0.750 in
```

Both control the same parameter.

------------------------------------------------------------------------

# 31. Dimension UX

Double-clicking a dimension should immediately expose editable numeric
input.

Example:

``` text
50 mm

→

┌──────────────────────┐
│ Dimension             │
│                       │
│ [ 60 mm ]             │
│                       │
│ ✓ Geometry updates    │
│                       │
│ [Apply]               │
└──────────────────────┘
```

If the dimension is driven rather than driving:

``` text
This measurement is calculated from other dimensions.

Why:
• Overall width = 100 mm
• Centered constraint
• Symmetry constraint

[Change controlling dimension]
[Remove constraint]
[Leave unchanged]
```

------------------------------------------------------------------------

# 32. Constraint System UX

Constraints remain professional CAD constraints internally.

MakerCAD should provide human explanations.

Instead of:

> OVER-CONSTRAINED SKETCH

Prefer:

> **These two rules disagree.**

Then:

``` text
Conflicting rules:

1. Horizontal
2. 45° angle

These cannot both be true.

[Show conflict]
[Fix automatically]
[Let me fix it]
```

The system should visually highlight the conflicting geometry.

------------------------------------------------------------------------

# 33. Universal "Why?" Function

Every important object, property, constraint, feature, warning, and
relationship should support:

> **Explain this**

Examples:

> Why can't I edit this dimension?

> Why is this part locked?

> Why did this feature fail?

> Why is this edge highlighted?

> Why is this component moving?

Answers should explain causality, not merely terminology.

------------------------------------------------------------------------

# 34. Error and Warning System

MakerCAD must never treat an error as a dead end.

Instead:

``` text
WHAT HAPPENED
WHY IT HAPPENED
WHAT IS AFFECTED
WHAT CAN FIX IT
```

Example:

``` text
THIS FEATURE NEEDS ATTENTION

Extrude "Base Thickness"

The sketch it uses changed.

Missing reference:
Sketch edge → Edge of Body001

[Show me]
[Choose replacement]
[Remove relationship]

Your previous version is safe.
```

------------------------------------------------------------------------

# 35. Model Health

A persistent but unobtrusive model-health indicator should exist.

``` text
MODEL HEALTH ✓ Healthy
```

or:

``` text
MODEL HEALTH ⚠ 2 items need attention
```

Selecting it opens:

``` text
2 warnings
0 errors

Sketch003
  Lost reference

Fillet002
  Radius cannot be applied

[Review]
```

This allows beginners to understand model status without reading a
complex feature tree.

------------------------------------------------------------------------

# 36. Undo as a Safety System

Undo is not merely a technical command.

It is a confidence mechanism.

Every major operation should be:

-   previewable
-   undoable
-   inspectable
-   recoverable

For potentially destructive operations:

``` text
Delete 3 features?

Used by:
• 2 sketches
• 1 hole
• 3 downstream features

[Cancel] [Delete]
```

Do not confirm every action.

**Confirm consequences, not clicks.**

------------------------------------------------------------------------

# 37. Preview Before Reality

Every consequential operation should preview the result directly in the
Canvas.

Example:

``` text
EXTRUDE

Distance: [25 mm]
Direction: [Symmetric ▼]

✓ Preview updates as you type

[Cancel] [Create]
```

The preview should exist in the model, not merely in a dialog.

------------------------------------------------------------------------

# 38. Soft Commit

MakerCAD may implement a soft-commit interaction layer.

Sequence:

``` text
Edit
 ↓
Live Preview
 ↓
Commit
 ↓
Brief recovery affordance
```

Example:

``` text
✓ Fillet created

[Undo] [Edit]
```

This is not a replacement for Undo.

It is a confidence aid.

------------------------------------------------------------------------

# 39. Timeline / History

The Timeline is a core professional feature and must remain visible in
the standard Design workspace.

Fusion documents the Timeline as a list of operations performed in the
design; operations can be right-clicked for changes, and in parametric
mode can be reordered by dragging.

MakerCAD should preserve:

-   chronological feature history
-   feature dependencies
-   edit-in-place
-   right-click context menu
-   suppression
-   reorder where valid
-   rollback
-   feature inspection
-   failure indicators
-   dependency visualization

------------------------------------------------------------------------

# 40. MakerCAD History Enhancement

The Timeline should support two simultaneous histories.

## Technical history

``` text
Sketch001
Extrude001
Fillet001
Hole001
```

## Human decision history

``` text
Changed desktop depth
↓
Selected reclaimed wood
↓
Reduced thickness to match inventory
↓
Moved drawer pull
↓
Adjusted cut plan
```

These histories must remain linked.

------------------------------------------------------------------------

# 41. Timeline Visual Semantics

Timeline icons/status should distinguish:

-   healthy
-   selected
-   suppressed
-   warning
-   error
-   externally referenced
-   dependent
-   user-modified
-   AI-suggested but not applied

Example:

``` text
[Sketch] → [Extrude] → [Fillet ⚠] → [Hole]
```

Clicking the warning should immediately explain the problem.

------------------------------------------------------------------------

# 42. Project History vs CAD History

MakerCAD should not confuse:

**Model history**

with:

**Project history**

Project history may contain:

-   design decisions
-   material changes
-   workshop limitations
-   rejected concepts
-   user notes
-   build corrections
-   photos
-   final build changes

This allows the final project to document not just *what* was modeled,
but *why*.

------------------------------------------------------------------------

# 43. Context / Task Panel

The right-side Context Panel should replace the need for permanent
property clutter.

It should dynamically display:

-   active command
-   selected object
-   parameters
-   project guidance
-   assistant explanation
-   warnings
-   suggested next actions

Example:

``` text
EXTRUDE

Selected:
Desktop Profile

Distance
[ 0.75" ]

Direction
[ One Side ▼ ]

Operation
[ New Body ▼ ]

Preview ✓

[Cancel] [Create]
```

------------------------------------------------------------------------

# 44. Assistant Integration

The Assistant should be accessible but not dominate the CAD workspace.

Fusion's current interface includes Autodesk Assistant access and an
Assistant panel; Autodesk's 2026 release notes also describe
natural-language capabilities for modeling, manufacturing, and
collaboration.

MakerCAD should treat this as a contextual assistance layer.

The assistant should answer:

-   What does this tool do?
-   Why did this fail?
-   What should I do next?
-   What are my options?
-   Will this fit?
-   Do I have enough material?
-   Can my workshop build this?
-   What changed?

It should not replace the Canvas, Browser, Timeline, or command system.

------------------------------------------------------------------------

# 45. Natural Language Commands

Natural language should map into existing CAD commands.

Example:

User:

> "Round these edges."

MakerCAD:

``` text
FILLET

Selected: 4 edges

Suggested radius:
2 mm

[Preview]
[Choose radius]
[Apply]
```

The underlying feature remains an ordinary Fillet.

------------------------------------------------------------------------

# 46. AI Geometry Generation

AI-generated geometry is acceptable as an experimental workflow,
particularly for difficult-to-model physical objects such as vintage
hardware.

However:

> AI-generated geometry must be converted into inspectable MakerCAD
> geometry or a clearly marked reference asset.

It must never become an opaque object that cannot be understood, edited,
measured, or replaced.

------------------------------------------------------------------------

# 47. Hardware / Part Library

User-created hardware should be stored locally and optionally shared.

Example:

``` text
LOCAL PART LIBRARY

Vintage Drawer Pull
Created: Custom Desk Project
Source: User photographs
Model: Parametric / Mesh / Reference

[Use Part]
[Edit]
[Export]
[Share]
```

Right-click should support:

> **Use Part**

The part should become a normal reusable component/reference in the
design.

------------------------------------------------------------------------

# 48. Material Inventory

Inventory is a first-class project object.

Example:

``` text
MATERIAL INVENTORY

3/4" Plywood
4 × 8 sheets
Available: 3

Reclaimed boards
Available: 11

Drawer pulls
Available: 4
```

Geometry should be able to reference inventory.

------------------------------------------------------------------------

# 49. Material-Aware Design

MakerCAD should compare:

``` text
DESIGN REQUIREMENT
vs.
AVAILABLE INVENTORY
```

Example:

``` text
Plywood required: 4.7 sheets
Plywood available: 3 sheets

⚠ Not enough material
```

Possible solutions:

``` text
[Reduce desktop depth]
[Change panel thickness]
[Reconfigure cut layout]
[Add material]
[Ignore inventory]
```

This is a major differentiator from traditional CAD.

------------------------------------------------------------------------

# 50. Design Budget

MakerCAD should support multiple design budgets:

``` text
Material
Cost
Weight
Time
Complexity
Available inventory
Available tools
Workspace
```

Example:

``` text
DESIGN BUDGET

Material       ████████░░ 82%
Cost           ██████░░░░ 61%
Weight         ████░░░░░░ 44%
Build Time     █████░░░░░ 51%
Complexity     ███████░░░ 71%
```

------------------------------------------------------------------------

# 51. Workshop Inventory

Tools and workspace should be established early.

Example:

``` text
WORKSHOP

Tools:
✓ Table saw
✓ Drill press
✓ Circular saw
✗ CNC
✗ Laser cutter

Workspace:
8' × 10'
```

MakerCAD should use this context in the Build Planner.

If a design requires CNC:

``` text
This design requires an operation
your workshop cannot currently perform.

[Redesign for table saw]
[Find external service]
[Keep current design]
```

------------------------------------------------------------------------

# 52. Fit / Clearance as a First-Class Feature

MakerCAD should make physical fit a primary modeling concept.

Example:

``` text
CLEARANCE

Left:    2.1"
Right:   4.7"
Back:    0.8"

⚠ Recommended minimum: 2"

[Adjust Automatically]
```

This is especially important when designing inside scans/splats.

------------------------------------------------------------------------

# 53. Human-Scale Validation

The assistant character can provide intuitive scale validation.

Example:

``` text
Desk height: 42"

⚠ This is unusually high for a desk.

Typical range:
28–32"

[Keep 42"]
[Try 30"]
```

This should be advisory, not authoritative.

------------------------------------------------------------------------

# 54. Build Planner

Once design is sufficiently mature, MakerCAD should transform the design
into build steps.

Example:

``` text
BUILD PLAN

01 Break down plywood
02 Cut desktop
03 Cut side panels
04 Drill mounting holes
05 Assemble frame
06 Install drawers
07 Install hardware
08 Sand / finish
```

Each step should link to the relevant geometry.

------------------------------------------------------------------------

# 55. BOM / Cut List / Manufacturing Outputs

MakerCAD's final maker package should include:

-   3D model
-   drawings
-   BOM
-   cut list
-   materials
-   required tools
-   estimated time
-   cost
-   build sequence
-   warnings
-   assembly instructions
-   photos
-   design decisions
-   build corrections

The CAD file is only one output.

------------------------------------------------------------------------

# 56. Build Corrections

The physical build may differ from the digital design.

MakerCAD must support:

``` text
BUILD NOTE

Desktop changed:
60" → 59.5"

Reason:
Reclaimed board required trimming.

[Update design]
```

Corrections become part of project history.

------------------------------------------------------------------------

# 57. Project Map

A high-level project map should allow the user to zoom out from CAD
details.

``` text
CUSTOM DESK
│
├── Environment
│   ├── Room Scan
│   ├── Wall
│   └── Floor
│
├── Materials
│   ├── Plywood
│   └── Reclaimed Boards
│
├── Hardware
│   └── Vintage Pull
│
├── Design
│   ├── Desktop
│   ├── Frame
│   └── Drawers
│
└── Build
    ├── Cut List
    ├── BOM
    └── Instructions
```

This is a primary UX layer for non-CAD users.

------------------------------------------------------------------------

# 58. Workflow Packs

MakerCAD should not implement unrelated "modes" as separate
applications.

Instead, use workflow packs over a shared core.

## Maker

``` text
Environment
Materials
Workshop
Manufacturing
BOM
Build Plan
```

## Animator

``` text
Characters
Rigging
Timeline
Camera
Animation
Rendering
```

## Game Developer

``` text
Level
Assets
Collision
Materials
Optimization
Export
```

## Product Designer

``` text
Concept
Form
Surface
Engineering
Prototype
Manufacturing
```

All workflows share:

``` text
Select
Move
Transform
Measure
Model
Undo
History
Constraints
References
Materials
Export
```

------------------------------------------------------------------------

# 59. Expert Mode

Expert Mode is not a different CAD engine.

It is a presentation layer.

Recommended expert workspace:

``` text
Minimal UI
Maximum Canvas
Full Browser
Full Timeline
Keyboard-first operation
Marking Menus
Command Search
Command Line
Advanced constraints
Advanced selection filters
```

The underlying model is identical.

------------------------------------------------------------------------

# 60. Explore / Design / Expert Presentation Layers

Three presentation levels may be offered:

## Explore

For beginners:

-   visual commands
-   natural language
-   contextual explanations
-   automatic safe constraints
-   previews
-   guided workflows

## Design

Default:

-   conventional toolbar
-   dynamic input
-   Browser
-   Timeline
-   shortcuts
-   contextual assistance

## Expert

Minimal:

-   keyboard-first
-   marking menus
-   command line
-   compact panels
-   advanced constraints
-   dense information

These must never produce incompatible project files.

------------------------------------------------------------------------

# 61. Tool Discovery

MakerCAD should teach professional CAD vocabulary through use.

Example:

First time:

``` text
Extrude

Turn this flat shape into a solid.

Shortcut: E
```

Later:

``` text
Extrude
E
```

Eventually:

``` text
E
```

This is progressive disclosure based on demonstrated competence.

------------------------------------------------------------------------

# 62. Universal "Explain This"

Any command, feature, object, constraint, or warning should support:

``` text
Explain this
```

Example:

``` text
Extrude

What it does:
Turns a 2D profile into a 3D form.

Professional term:
Extrude

Shortcut:
E
```

This creates a bridge between plain-language understanding and
professional CAD terminology.

------------------------------------------------------------------------

# 63. Natural Language Should Be a Command Interface, Not a Second Modeling System

Architecture:

``` text
Natural Language
       ↓
Intent Resolver
       ↓
Existing CAD Command
       ↓
Normal Feature
       ↓
Parametric History
```

Never:

``` text
Natural Language
       ↓
Mystery AI Geometry
```

The former is predictable.

The latter creates a second modeling universe that professionals cannot
reliably inspect or repair.

------------------------------------------------------------------------

# 64. Safety Rule for AI

AI may propose:

> "I found a way to save 0.8 sheets of plywood."

Then show:

``` text
BEFORE              AFTER

24" depth           22" depth
3.8 sheets          3.0 sheets

[Preview]
[Apply]
```

AI must not silently optimize the design.

------------------------------------------------------------------------

# 65. Beginner Anxiety Model

The main beginner problem is not the geometry.

It is **state anxiety**.

The user does not know:

> "What state am I currently in?"

Therefore MakerCAD must continuously communicate:

-   current command
-   current selection
-   current input
-   current target
-   current preview
-   how to commit
-   how to cancel
-   what changed
-   whether the model is healthy

Without overwhelming the user.

------------------------------------------------------------------------

# 66. Command State HUD

A small status area should always communicate the current command state.

Example:

``` text
LINE
Click: place point
Enter: finish
Esc: cancel
```

When idle:

``` text
SELECT
Click an object to select it.
```

When a command is active, the state should be visible but visually
subordinate.

------------------------------------------------------------------------

# 67. Contextual Micro-Interactions

MakerCAD should use small visual explanations rather than large tutorial
dialogs.

Examples:

``` text
Horizontal
──────────────
```

``` text
Coincident
●
●
```

``` text
Perpendicular
└
```

``` text
Tangent
○────
```

These appear when relevant and fade when understood.

------------------------------------------------------------------------

# 68. Familiarity vs. Innovation Matrix

  Area           Preserve Familiar CAD   Improve MakerCAD
  -------------- ----------------------- -------------------------------
  Selection      Window/Crossing         Explain visually
  Mouse          MMB/Pan/Orbit/Zoom      Better onboarding
  ViewCube       Keep                    Human/environment orientation
  Toolbar        Keep                    Dynamic project tools
  Browser        Keep                    Friendly semantic layer
  Timeline       Keep                    Add decision/build history
  Dimensions     Keep                    Explain dependencies
  Constraints    Keep                    Explain conflicts
  Commands       Keep                    Natural-language entry
  Marking Menu   Keep                    Context-aware
  Undo           Keep                    Add visual safety
  Errors         Keep                    Explain causality
  AI             Add                     Never own geometry
  Splat          New                     Treat as reference world
  Inventory      New                     Link to geometry
  Workshop       New                     Link to manufacturing
  Build Plan     New                     Generate from design
  Project Map    New                     Goal-oriented navigation

------------------------------------------------------------------------

# 69. Recommended Default Workspace

The following should be the initial MakerCAD Design workspace
specification.

``` text
┌──────────────────────────────────────────────────────────────────────────────┐
│ MakerCAD | Project Name | Saved | Undo | Redo | Search                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Project │ Design │ Sketch │ Create │ Modify │ Assemble │ Inspect │ Tools     │
├──────────────┬─────────────────────────────────────────────┬─────────────────┤
│ PROJECT      │                                             │ CONTEXT         │
│ BROWSER      │                                             │ PANEL           │
│              │                                             │                 │
│ Environment  │                                             │ Selected object │
│ Components   │                                             │ Command inputs  │
│ Bodies       │                 3D CANVAS                   │ Guidance        │
│ Sketches     │                                             │ Parameters      │
│ Features     │                                  ViewCube   │ Warnings        │
│ References   │                                             │ Assistant       │
│ Materials    │                                             │                 │
│ Build        │                                             │                 │
├──────────────┴─────────────────────────────────────────────┴─────────────────┤
│ TIMELINE / HISTORY                                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ Command State | Selection | Units | Model Health | Navigation | Assistant   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------------------------------------------------------------------------

# 70. Workspace Persistence

MakerCAD should remember user layout choices.

Store:

-   panel widths
-   Browser state
-   Timeline height
-   tool visibility
-   pinned tools
-   workflow pack
-   assistant state
-   ViewCube visibility
-   navigation preferences
-   selection preferences
-   units
-   viewport display mode
-   keyboard shortcut customizations

Provide:

``` text
Reset Workspace Layout
```

and a professional shortcut for resetting layout.

Fusion itself documents a reset-layout command among its interface
shortcuts; MakerCAD should provide an equivalent recovery path.

------------------------------------------------------------------------

# 71. Keyboard Shortcut Customization

MakerCAD should eventually allow:

-   custom shortcuts
-   shortcut conflict detection
-   command search
-   shortcut display beside command names
-   import/export of shortcut profiles

However, the default profile should prioritize compatibility with
familiar Fusion/mechanical CAD conventions.

The system should warn if a custom shortcut conflicts with:

-   navigation
-   text entry
-   global commands
-   active sketch input
-   existing command shortcuts

------------------------------------------------------------------------

# 72. Command Search

A universal command search should exist.

Example:

``` text
Search commands...

extrude
```

Results:

``` text
Extrude
Extrude Mesh
Extrude Surface
```

For beginners:

``` text
Turn a flat shape into a solid
Extrude
E
```

For experts:

``` text
Extrude — E
```

Search should work even when a command is not currently visible in the
dynamic Toolbox.

------------------------------------------------------------------------

# 73. Command Line

MakerCAD should eventually provide an optional command line for:

-   expert users
-   automation
-   scripting
-   diagnostics
-   AI translation
-   reproducibility

Example:

``` text
> extrude DesktopProfile 25
> fillet Edge12 4
> select Body1
> hide Sketch003
```

Natural language may resolve into this layer, but the command layer
remains deterministic.

------------------------------------------------------------------------

# 74. Project Creation Flow

Recommended first-run project flow:

``` text
START
 ↓
What are you making?
 ↓
Describe the project
 ↓
Add photos / scans / models / measurements
 ↓
Select units
 ↓
Identify environment
 ↓
Inventory materials
 ↓
Inventory tools/workspace
 ↓
Review suggested project tools
 ↓
Enter project
```

The system should allow users to skip or return to any step.

------------------------------------------------------------------------

# 75. Custom Desk Example --- Complete MakerCAD Flow

## Stage 1 --- Project creation

User:

> Custom Desk Project

Description:

> A wooden desk for a small, strangely shaped space where a bookshelf
> used to be. I have a 3D scan, a splat file, hard measurements,
> plywood, reclaimed boards, and vintage drawer pulls.

## Stage 2 --- Project interpreter

Detects:

``` text
Desk
Wood
Plywood
Reclaimed
Measurements
Splat
3D scan
Drawer pulls
Workshop build
```

## Stage 3 --- Project Toolbox

Suggests:

``` text
Environment Scan
Splat Reference
Measurement Probe
Material Inventory
Parametric Boards
Hardware Library
Fit / Clearance
Material Estimator
Build Planner
BOM
Cut List
```

## Stage 4 --- Environment

Load scan/splat.

Calibrate known measurements.

Verify scale.

## Stage 5 --- Human scale

Assistant character enters the environment at approximately 5'6" scale.

User can immediately understand room and furniture proportions.

## Stage 6 --- Rough design

User creates a desk using primitive shapes.

No sketch knowledge required.

## Stage 7 --- Inventory check

System identifies material shortage.

User refines dimensions.

## Stage 8 --- Parametric refinement

Board thickness is linked to inventory.

Geometry remains professionally parametric.

## Stage 9 --- Vintage hardware

User photographs the hardware.

MakerCAD guides:

-   top
-   bottom
-   front
-   back
-   left
-   right

AI-assisted model generation is allowed as an experimental workflow.

## Stage 10 --- Hardware library

Model saved locally.

Right-click:

``` text
Use Part
```

## Stage 11 --- Workshop

User lists:

-   table saw
-   circular saw
-   drill
-   drill press
-   workspace constraints

## Stage 12 --- Manufacturing analysis

MakerCAD produces:

-   BOM
-   cut list
-   required tools
-   build sequence
-   material consumption
-   time estimate
-   warnings

## Stage 13 --- Build

User can follow generated instructions.

## Stage 14 --- Correction

User records real-world changes.

## Stage 15 --- Publication

Final design package can be shared.

------------------------------------------------------------------------

# 76. Final Product Output

For maker projects, the final output should be a **Project Package**.

``` text
PROJECT PACKAGE

3D Model
Drawings
BOM
Cut List
Materials
Tool List
Build Sequence
Instructions
Estimated Cost
Estimated Time
Design Decisions
Build Corrections
Photos
Source Models
Hardware Library Items
```

This is a core product concept, not an afterthought.

------------------------------------------------------------------------

# 77. Other User Types

MakerCAD should support multiple professional workflows without
fragmenting the platform.

## Video Effects / Animators

Primary workflow:

``` text
Concept
→ Model
→ Surface
→ Rig
→ Animate
→ Camera
→ Render
→ Export
```

## Game Developers

``` text
Concept
→ Model
→ Optimize
→ Materials
→ Collision
→ LOD
→ Scene
→ Export
```

## Product Designers

``` text
Concept
→ Form
→ Surface
→ Engineering
→ Prototype
→ Manufacturing
```

## Future workflow packs

The platform should allow additional niche workflows without rewriting
the core CAD engine.

------------------------------------------------------------------------

# 78. Design Principles --- Final Accepted Set

1.  **Do not hide professional CAD underneath a toy interface.**
2.  **Keep the professional CAD grammar intact.**
3.  **Make complexity progressively discoverable.**
4.  **Let users begin with goals, not commands.**
5.  **Treat project context as first-class data.**
6.  **Use a dynamic Toolbox without making the system a black box.**
7.  **Keep all professional commands accessible.**
8.  **Preserve keyboard shortcuts and mouse conventions.**
9.  **Preserve the Browser and Timeline.**
10. **Preserve the ViewCube and Navigation Bar.**
11. **Preserve right-click contextual/marking menus.**
12. **Preserve Enter/Enter-to-commit and Escape-to-cancel semantics.**
13. **Make dynamic numeric input first-class.**
14. **Make previewing standard.**
15. **Make Undo psychologically visible and safe.**
16. **Explain errors causally.**
17. **Explain constraints in human language.**
18. **Treat scans and splats as spatial reference evidence, not failed
    CAD.**
19. **Treat material inventory as part of the design system.**
20. **Treat workshop tools as part of the design context.**
21. **Make fit and clearance first-class.**
22. **Use human scale alongside numerical dimensions.**
23. **Use the assistant as a guide, not as the interface.**
24. **Never let AI silently alter consequential geometry.**
25. **Map natural language into deterministic CAD commands.**
26. **Keep beginner, normal, and expert workflows on the same model.**
27. **Add project history alongside technical model history.**
28. **Generate build instructions from the actual design.**
29. **Allow build corrections to become part of project history.**
30. **Treat the final deliverable as a project package, not merely a CAD
    file.**
31. **Support workflow packs rather than isolated application modes.**
32. **Allow experts to customize and pin tools.**
33. **Never remove a professional capability merely because beginners do
    not need it yet.**
34. **Make the software explain itself at the moment of need.**
35. **Make experimentation feel safe.**

------------------------------------------------------------------------

# 79. Final UX North Star

MakerCAD should make the beginner feel:

> **"I can make something without already knowing CAD."**

The intermediate user should feel:

> **"I understand why the model works this way."**

The professional should feel:

> **"This is a serious CAD system, and it doesn't get in my way."**

The final product should therefore feel like:

``` text
                     MAKERCAD
                         │
             ┌───────────┴───────────┐
             │                       │
        HUMAN WORKFLOW          PROFESSIONAL CAD
             │                       │
        Goal / Context           Sketch / Feature
        Environment              Constraint
        Materials                Parameter
        Workshop                 Component
        Build                    Timeline
        Documentation            Precision
             │                       │
             └───────────┬───────────┘
                         ↓
                 ONE SHARED MODEL
                         ↓
              DESIGN → BUILD → SHARE
```

The central product promise is:

> **MakerCAD lets people enter through the way they naturally think
> about making things, while allowing them to grow into the precise
> professional CAD workflows that power the same model underneath.**

That is the design direction this specification establishes.
