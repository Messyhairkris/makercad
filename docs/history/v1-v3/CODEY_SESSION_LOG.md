# Codey Session Log - MakerCAD Development

**Session Date:** 2026-01-12
**Instance:** Codey
**Project:** MakerCAD - Three.js Browser-Based CAD Tool

---

## Session Overview

Worked on MakerCAD, a low-cost, low-system-demand solid modeling CAD tool using Three.js designed to run on 10-year-old computers without GPU requirements.

---

## Research Phase

### Initial Task
- Researched 20+ Three.js CAD projects and solutions
- Analyzed WebAssembly OCCT (Open Cascade) integration feasibility
- Studied constraint solvers, CSG libraries, performance strategies

### Key Research Findings

#### Existing Three.js CAD Projects
- **Three.cad**: Full parametric CAD with WASM constraint solving
- **SimCAD 3D**: Browser-based with THREE-BVH-CSG for booleans
- **Bitbybit.dev**: OCCT + Three.js integration library
- **Polygonjs**: Node-based procedural CAD
- **React Three Fiber + Drei**: Modern declarative Three.js

#### Solid Modeling Kernels
- **Open Cascade (OCCT)**: FREE under LGPL, industrial-strength
- **BRL-CAD**: Military-grade, since 1979
- **SolveSpace**: GPL parametric constraint solver
- **PicoGK**: Apache-2.0, minimal instruction set

#### Feasibility Verdict
✅ POSSIBLE - Can build SolidWorks-quality CAD on 10-year hardware using:
- Three.js for rendering
- OCCT compiled to WASM for geometry
- WebWorkers for parallel computation
- Instanced meshes for performance

---

## Implementation Phase

### Project Structure Created
```
MakerCAD/
├── index.html
├── package.json (Vite, React, Three.js, Zustand)
├── vite.config.ts
├── tsconfig.json
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── components/
    │   ├── Canvas.tsx (3D viewport)
    │   ├── Scene.tsx (CAD objects)
    │   ├── ContextMenu.tsx (right-click tools)
    │   ├── UnitSelector.tsx (scale selection)
    │   ├── ObjectList.tsx (BOM)
    │   ├── HistoryPanel.tsx (undo/redo)
    │   ├── UIToolbar.tsx (mode switching)
    │   ├── SidePanel.tsx (properties)
    │   ├── DrawingTools.tsx (draw mode tools)
    │   ├── ErrorBoundary.tsx
    │   ├── LoadingSpinner.tsx
    │   ├── icons.tsx (SVG icons)
    │   └── useCADInteraction.ts (raycasting)
    ├── store/
    │   ├── cadStore.ts (scene objects, selection)
    │   └── modeStore.ts (modes, units, drawing)
    └── styles/
        ├── globals.css (dark theme)
        └── wii-theme.css (light Wii-style)
```

### Key Features Implemented

#### 1. Selection System ✅
- Raycasting for object selection
- Left click = select object
- Shift + click = multi-select
- Blue outline (#0099cc) on selected objects
- Outline mesh scaled 1.005 to avoid z-fighting

#### 2. Context Menu (Autodesk-style) ✅
- Right-click on object or empty space
- Mode switching: Select / Edit / Draw
- Face tools: Extrude, Cut, Rotate, Offset
- Object tools: Duplicate, Delete, Lock, Hide
- Transform tools: Move, Rotate, Scale
- Grid tools: Draw Rectangle, Settings

#### 3. Grid Improvements ✅
- "Grids of ten" approach (10 divisions)
- Subtle colors: #c5cbd3 (cells), #a0a8b0 (sections)
- Unit-aware: 10" default, scales per unit
- Fades at distance with fadeStrength=2
- Section lines at major divisions

#### 4. Demo Scene (Chair) ✅
- Seat: 18" × 16" × 1.5"
- Four legs: 1.5" × 1.5" × 15"
- All objects selectable
- Proper furniture scale

#### 5. Store Architecture ✅
**cadStore:**
- Scene objects array
- SelectedObjects array
- addObject, removeObject, updateObject
- selectObject (with multi-select)
- duplicateObject
- History tracking (canUndo, canRedo, undo, redo)

**modeStore:**
- Active modes: select, edit, draw, extrude, sketch, measure
- Draw tools: rectangle, line, circle, polygon
- Units: mm, in, ft, m (with conversion factor)
- Grid size, snapToGrid, showGrid
- Drawing state (drawingStart, drawingEnd, dimensions)

#### 6. UI Components ✅
- **UnitSelector**: Compact dropdown for units
- **ObjectList**: BOM with icons, count, delete
- **HistoryPanel**: Undo/redo buttons, action timeline
- **DrawingTools**: Rectangle, circle, polygon, line tools
- **UIToolbar**: Top mode switching (Select/Edit/Draw)

#### 7. Theme System ✅
**Wii Theme (Light):**
- Background: #f5f7fa gradient
- White panels: #ffffff
- Cream accents: #f5f5f0
- Blue primary: #0099cc
- Rounded corners: 12-20px
- Soft shadows: 0 4px 8px rgba(0,0,0,0.08)

### UI Layout Implemented
```
┌──────────────────────────────────────────────┐
│ Units Dropdown     │      Canvas              │
├────────────────────┤      (3D viewport)       │
│ Objects List       │                          │
│ (BOM)              │                          │
├────────────────────┤                          │
│ Drawing Tools      │                          │
│ (floating)         │                          │
├────────────────────┼──────────────────────────┤
│ History Panel      │ (Top Toolbar: modes)    │
│ (bottom)           │                          │
└────────────────────┴──────────────────────────┘

Right Panel: Inspector (when object selected)
```

---

## Controls Implemented

| Action | Input | Result |
|--------|-------|--------|
| Select Object | Left Click | Highlight object |
| Multi-Select | Shift + Click | Add to selection |
| Context Menu | Right Click | Tool menu |
| Orbit | Middle Drag | Rotate camera |
| Pan | Shift + Middle | Move camera |
| Zoom | Scroll Wheel | Zoom in/out |

---

## Next Steps (Pending)

### Immediate Priority
1. **Face Selection** - Raycast to detect clicked face
2. **Face Highlighting** - Visual feedback for selected face
3. **Extrude Operation** - Pull face to create 3D geometry
4. **Constraint Solving** - Distance, parallel, perpendicular

### Future Features
1. 2D Sketching on planes
2. Boolean operations (union, subtract, intersect)
3. Fillet/chamfer operations
4. Pattern tools (linear, circular)
5. Assembly system with joints
6. STL/STEP export

---

## User Requirements Captured

### Core Needs
- Run on 10-year-old computers (i5-4xxx, 8GB RAM, Intel HD 4600)
- No GPU acceleration required
- Browser-based (no installation)
- Full solid modeling (not just mesh)
- Parametric editing (history tree)
- Constraint solver for sketches
- Comparable to SolidWorks/AutoCAD for typical use

### Design Preferences
- Wii-style UI (light, airy, non-intrusive)
- Grid as design floor (not vertical backdrop)
- Unit selection at start (mm, in, ft, m)
-_right-click tools (Autodesk-style)
- Face-first workflow (select face, then operation)
- Named, reusable components

---

## Technical Decisions

### Architecture
- **Frontend**: React 18 + TypeScript
- **3D Engine**: Three.js + React Three Fiber
- **State**: Zustand (stores)
- **Build**: Vite (fast dev, optimized prod)
- **Styling**: CSS Modules (globals.css + wii-theme.css)

### Performance Strategy
- Built-in Three.js geometries (Box, Sphere, Cylinder)
- No custom ArrayMesh/SurfaceTool (fails on Intel HD 530)
- DPR limited to [1, 1.5] for low-end hardware
- Instanced meshes for repeated geometry
- WebWorkers for geometry operations (future)
- OCCT WASM for booleans/fillets (future)

### Error Handling
- React ErrorBoundary around Canvas
- Global error listener for uncaught exceptions
- User-friendly error overlay
- Retry