# MakerCAD Phase 1 Addendum

**Date:** April 26, 2026  
**Status:** Phase 1 Complete — GDExtension Skeleton + First Shape  

---

## Summary

Phase 1 has been successfully completed. The GDExtension skeleton is built and functional, and a test box is rendering in Godot with Phong shading.

---

## Changes from Original Plan

### Change 1: godot-cpp Version

**Original Plan:** Use godot-cpp tag `godot-4.6.2-stable`  
**Actual:** Used `godot-4.5-stable` (latest available at time of build)  
**Reason:** The `godot-4.6.2-stable` tag does not exist in the godot-cpp repository. The godot-4.5-stable tag is compatible with Godot 4.6.2.

**Impact:** Minimal. API compatibility between 4.5 and 4.6 GDExtension is maintained. The extension loads and functions correctly.

---

### Change 2: Build Configuration — Static Runtime

**Original Plan:** Standard CMake Debug configuration  
**Actual:** Configured for static runtime (`/MT`) with Release configuration  
**Reason:** The godot-cpp library was built with `/MT` (Multi-threaded Static) runtime. Linking with `/MDd` (Multi-threaded Debug DLL) caused 110 linker mismatches.

**Solution Applied:**
```cmake
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")
set(CMAKE_BUILD_TYPE "Release")
```

**Impact:** Extension links successfully. For future development, consider rebuilding godot-cpp with debug symbols if debugging becomes critical.

---

### Change 3: OCCT Version

**Original Plan:** OpenCASCADE 7.9.0  
**Actual:** OpenCASCADE 7.9.3  
**Reason:** 7.9.3 was the latest patch release available at download time with prebuilt binaries.

**Impact:** None. 7.9.3 is a bugfix release in the 7.9 series. All APIs used remain compatible.

---

## Files Created

### Build System
- `C:\dev\makercad-extension\CMakeLists.txt` — Main CMake configuration
- `C:\dev\makercad-extension\configure.bat` — Environment setup + CMake configure
- `C:\dev\makercad-extension\build.bat` — Build script
- `C:\dev\makercad-extension\copy_dlls.ps1` — OCCT DLL deployment

### Source Code
- `src/register_types.h` / `register_types.cpp` — GDExtension entry point
- `src/shape_store.h` / `shape_store.cpp` — OCCT containment boundary
- `src/history_engine.h` / `history_engine.cpp` — History management (skeleton)
- `src/tessellator.h` / `tessellator.cpp` — OCCT → Godot mesh conversion
- `src/makercad_extension.h` / `makercad_extension.cpp` — Main Godot node class

### GDExtension Manifest
- `demo/bin/makercad.gdextension` — Extension registration

### Godot Test Project
- `demo/project.godot` — Godot project file
- `demo/test.tscn` — Test scene with MakerCADExtension node

---

## Build Output

```
C:\dev\makercad-extension\demo\bin\
├── libmakercad.windows.template_debug.x86_64.dll  (507 KB)
├── libmakercad.windows.template_debug.x86_64.pdb  (2.6 MB)
├── makercad.gdextension
├── TKernel.dll, TKMath.dll, ...                   (OCCT runtime)
└── (9 OCCT DLLs total)
```

---

## Architecture Verification

### OCCT Boundary Rule: ✅ PASS
- All `TopoDS_Shape` objects live in `ShapeStore`
- Only `ShapeID` strings (std::string) cross component boundaries
- No raw OCCT types in GDScript-facing API

### History Node Structure: ✅ IMPLEMENTED
- Basic `HistoryNode` struct defined per Section 5 of project plan
- `HistoryEngine` skeleton with add/serialize operations
- Full implementation deferred to Phase 3 (as planned)

### Tessellation: ✅ WORKING
- `BRepMesh_IncrementalMesh` generates triangulation
- `Poly_Triangulation` converted to Godot `ArrayMesh`
- Vertex normals calculated for Phong shading

---

## Known Limitations (Phase 1)

1. **History is not yet functional for modeling** — only the skeleton exists. Operations do not yet rebuild or create geometry through history.

2. **SelectionIntent not yet implemented** — geometric heuristics for persistent selection will be built in Phase 3.

3. **No dirty flag rebuild** — Phase 3 feature. Currently only synchronous test shape creation.

4. **Tessellation quality is basic** — Linear deflection of 0.5mm. Configurable per Phase 6.

5. **No error recovery UX** — Phase 3.5 feature. OCCT failures currently only log to console.

---

## Next Steps (Phase 2)

Per Section 10 of the project plan:

1. **Viewport & Camera**
   - Orbit (left drag / 2-finger rotate)
   - Pan (middle drag / 3-finger)
   - Zoom (scroll / pinch)

2. **Workplane Grid**
   - Shader with depth bias for Intel HD 530
   - XY, YZ, ZX plane selection

3. **Ray-to-BRep Picking**
   - `BRepIntCurveSurface_Inter` for face selection
   - Hover highlighting
   - Click selection

4. **Rendering Validation**
   - Thick wireframe approach (quad strips)
   - Z-fighting mitigation confirmed on HD 530
   - Edge highlight selection

---

## Verification Commands

To verify the build:

```powershell
# Check DLL exists
Test-Path "C:\dev\makercad-extension\demo\bin\libmakercad.windows.template_debug.x86_64.dll"

# Check OCCT DLLs copied
Test-Path "C:\dev\makercad-extension\demo\bin\TKernel.dll"

# Launch Godot project
cd C:\dev\makercad-extension\demo
C:\Users\<you>\Godot\Godot_v4.6.2-stable_win64.exe --path .
```

---

**End of Phase 1 Addendum**
