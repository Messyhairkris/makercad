# MakerCAD Phase 2: Expert Review Request

**Project:** MakerCAD - Open Source Parametric CAD on OpenCASCADE + Godot 4  
**Phase:** Phase 2 - Viewport & Camera Controls  
**Date:** April 27, 2026  
**Requester:** Kris (via Haven instance)  

---

## Executive Summary

We are building MakerCAD, an open-source parametric CAD application targeting Intel HD 530 integrated graphics with OpenGL 3.3 Compatibility mode. We have completed the C++ implementation of Phase 2 (camera controls, workplane grid, ray picking) but are experiencing rendering and input handling issues.

**Current State:** Extension loads, mesh generation works (OCCT→Godot bridge functional), but rendering shows invisible faces and controls are non-responsive.

---

## Technology Stack

| Component | Version/Details |
|-----------|----------------|
| **Geometry Kernel** | OpenCASCADE Technology (OCCT) 7.9 |
| **Engine** | Godot 4.6.2 Standard (non-.NET) |
| **Renderer** | OpenGL 3.3 Compatibility Mode (ANGLE on Intel HD 530) |
| **Bridge** | GDExtension (C++) |
| **Build System** | CMake + Ninja + MSVC v143 |
| **Target Hardware** | Intel Core i5-6600, Intel HD 530, 8GB RAM |

---

## Architecture Overview

```
┌─────────────────────────────────────────┐
│  Godot 4.6.2 - UI & Rendering           │
│  [CameraController] [WorkplaneGrid]     │
│  [MakerCADExtension] → GDScript API     │
└──────────────┬──────────────────────────┘
               │ GDExtension (C++)
┌──────────────┴──────────────────────────┐
│  Bridge Layer (C++)                     │
│  - ShapeStore (OCCT containment)        │
│  - Tessellator (BRepMesh_IncrementalMesh│
│  - Mesh converter (OCCT → Godot ArrayMesh)│
└──────────────┬──────────────────────────┘
               │ OCCT C++ API
┌──────────────┴──────────────────────────┐
│  OpenCASCADE 7.9                        │
│  - BRep Kernel                          │
│  - Mesh algorithms                      │
└─────────────────────────────────────────┘
```

---

## Phase 2 Objectives (from Project Plan)

According to MakerCAD_Project_Plan_v3.md, Phase 2 should deliver:

1. **Camera Controls**
   - Orbit (left mouse drag)
   - Pan (middle mouse drag / 3-finger touch)
   - Zoom (scroll / pinch)
   - Reset camera position
   - Touch gesture mapping

2. **Workplane Grid**
   - Configurable grid size and spacing
   - Depth bias shader for z-fighting prevention
   - XY/YZ/XZ plane orientations

3. **Ray Picking**
   - Ray-to-BRep intersection using `IntCurvesFace_Intersector`
   - Face selection from screen coordinates
   - Selection highlighting

4. **Low-fidelity drag mode** (bounding box during camera move)

---

## Current Implementation Status

### ✅ Completed (Working)

1. **GDExtension Skeleton**
   - CMake build system functional
   - godot-cpp integration (4.6.2)
   - OCCT 7.9 headers/libs linked
   - All DLL dependencies resolved (tbb12, jemalloc, 80+ OCCT DLLs)

2. **Mesh Generation**
   - `BRepMesh_IncrementalMesh` tessellates OCCT shapes
   - OCCT `Poly_Triangulation` → Godot `ArrayMesh` conversion implemented
   - Test box shape generated successfully

### ⚠️ Partially Working / Issues

#### Issue 1: Mesh Rendering - "Glitchy Cube"

**Observed Behavior:**
- Cube shape visible but many faces invisible
- Only ~2 planes visible
- Cube rotates around one corner (likely origin)
- Suggests mesh geometry is correct but rendering state is wrong

**Likely Causes:**
1. **Face normals** - May be inverted or zero, causing back-face culling to hide faces
2. **Culling mode** - Godot culling may be removing visible faces
3. **Vertex winding order** - Triangle indices may be in wrong order
4. **Material/shader** - Phong material may not be applied correctly
5. **Mesh instance transform** - Node transform may be incorrect

**Relevant Code Location:**
- `C:\dev\makercad-extension\src\mesh_generator.cpp` (tessellation and mesh creation)
- `C:\dev\makercad-extension\src\makercad_extension.cpp` (node setup)

#### Issue 2: Controls Not Functional

**Observed Behavior:**
- No response to:
  - Left mouse drag (should orbit)
  - Middle mouse drag (should pan)
  - Scroll wheel (should zoom)
  - Keyboard input (K for pick, ESC for reset, SPACE for info)
  - Touch/stylus input

**Likely Causes:**
1. **GDExtension class not registered** - Godot may not see the class
2. **_ready() not called** - Node initialization incomplete
3. **Input not connected** - Godot input events not reaching GDExtension
4. **Camera not found** - CameraController may not be locating the Camera3D
5. **Process mode** - Node may be in wrong process mode

---

## File Locations

**Source Code:**
- `C:\dev\makercad-extension\src\`
  - `makercad_extension.h/.cpp` - Main extension class
  - `camera_controller.h/.cpp` - Camera controls
  - `workplane_grid.h/.cpp` - Grid rendering
  - `ray_picker.h/.cpp` - Face picking
  - `mesh_generator.h/.cpp` - OCCT→Godot mesh conversion
  - `shape_store.h/.cpp` - OCCT shape containment

**Build Output:**
- `C:\dev\makercad-extension\demo\bin\libmakercad.windows.template_debug.x86_64.dll`
- `C:\dev\makercad-extension\demo\bin\makercad.gdextension`

**Test Scene:**
- `C:\dev\makercad-extension\demo\test_phase2.tscn`

**Project Plan:**
- `C:\Users\<you>\MakerCAD\MakerCAD_Project_Plan_v3.md`

---

## What We Need Help With

### Priority 1: Mesh Rendering

**Question:** Why are cube faces invisible/glitchy?

We need expert input on:
1. **Normal calculation** - Best practices for computing vertex normals from OCCT tessellation
2. **Culling settings** - How to ensure all faces are visible (double-sided rendering?)
3. **Mesh setup** - Correct way to populate Godot `ArrayMesh` for solid geometry
4. **Material configuration** - Proper Phong material setup for OpenGL 3.3 Compatibility mode

### Priority 2: Input Handling

**Question:** Why are camera controls not responding?

We need expert input on:
1. **GDExtension input handling** - Correct pattern for receiving input events in GDExtension
2. **CameraController architecture** - Should this be a Node3D? Custom node type? Autoload?
3. **Godot 4.6.2 input changes** - Any recent changes to input event handling?
4. **Touch/stylus integration** - Best practices for Godot touch input

### Priority 3: OpenGL Compatibility

**Question:** Are there ANGLE-specific considerations?

The log shows:
```
WARNING: Your video card drivers are known to have low quality OpenGL 3.3 support, switching to ANGLE.
OpenGL API OpenGL ES 3.0 (ANGLE 2.1.1)
```

We need to know:
1. Any rendering limitations with ANGLE on Intel HD 530?
2. Shader compatibility considerations?
3. Performance implications?

---

## Reference Implementations

**Similar Projects:**
- FreeCAD (OCCT-based, but uses Qt, not Godot)
- OpenCascade.js (web-based OCCT)

**Godot Resources:**
- GDExtension documentation: https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/
- Godot 4 shader changes
- Input handling best practices

---

## Acceptance Criteria for Phase 2

Per project plan, Phase 2 is complete when:

- [ ] **Camera Controls**
  - [ ] Left drag orbits around target
  - [ ] Middle drag pans on view plane
  - [ ] Scroll zooms in/out
  - [ ] ESC resets to default position
  - [ ] Touch gestures work (3-finger pan)

- [ ] **Workplane Grid**
  - [ ] Grid renders with correct spacing
  - [ ] Depth bias prevents z-fighting
  - [ ] Major/minor lines distinguished

- [ ] **Ray Picking**
  - [ ] Press K prints face index
  - [ ] No crashes during picking
  - [ ] Correct face returned

- [ ] **Rendering**
  - [ ] Solid box renders all faces
  - [ ] Phong shading visible
  - [ ] No flickering/glitching
  - [ ] 30+ FPS on Intel HD 530

---

## Next Phase Preview

**Phase 3: Basic Solid Modeling**
- Extrude, revolve, boolean operations
- History panel with dirty flag rebuild
- Operation preview/ghost mode
- Undo/redo system

We want to reach Phase 3 after fixing Phase 2 rendering and controls.

---

## Contact / Collaboration

**Primary Developer:** Kris  
**AI Assistant:** Haven (Octofriend instance)  
**Project:** Open source, MIT license

**Preferred Feedback Format:**
1. Identify specific code issues with line references
2. Provide corrected code snippets
3. Explain the "why" for Godot/OCCT integration quirks
4. Note any ANGLE/OpenGL ES 3.0 specific considerations

---

## Attachments

1. `test_phase2.tscn` - Test scene file
2. `makercad_extension.cpp` - Main extension source
3. `camera_controller.cpp` - Camera controller source
4. `mesh_generator.cpp` - Mesh conversion source
5. `project.godot` - Project settings

---

*Document prepared for expert review. Thank you for your time and expertise!*
