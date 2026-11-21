# MakerCAD Build Instructions - Fix Error 126

## Problem

Godot shows this error:
```
ERROR: Can't open dynamic library: libmakercad.windows.template_debug.x86_64.dll
Error: Error 126: The specified module could not be found
```

**Why this happens:**
The DLL exists in `demo/bin/` but was built **before** today's Phase 2 code changes. The .gdextension manifest expects today's API (with camera pan, ray picking, etc.) but the DLL only contains old symbols.

## Solution: Rebuild the Extension

### Step 1: Check Your Build Environment

Run the diagnosis script:
```batch
cd C:\dev\makercad-extension
diagnose_build.bat
```

Look for:
```
[OK] cl.exe found
[OK] cmake found
[OK] nmake found
[OK] godot-cpp lib found
[OK] OpenCASCADE libraries found
[OK] Source files found
[OK] OCCT DLLs found
```

If any check fails, install the missing tool first.

### Step 2: Clean Rebuild

Run the rebuild script:
```batch
cd C:\dev\makercad-extension
rebuild_clean.bat
```

This script:
1. Removes the old build directory
2. Reconfigures CMake
3. Compiles all source files with NMake
4. Copies the new DLL to `demo/bin/`
5. Launches Godot for you

### Step 3: Verify in Godot

When Godot opens, open this project:
```
C:\Users\<you>\MakerCAD\maker-cad-program\project.godot
```

Check the **Output** panel (bottom of editor):
```
MakerCADExtension: Constructor
MakerCADExtension: _ready
MakerCADExtension: Creating test box...
Shape stored with ID: shape_xxx
Mesh created with 1 surfaces
```

You should see an **orange 20×20×20mm box** in the viewport.

### Step 4: Test Controls

Run the scene (F5) and test:

| Control | Action | Expected Result |
|---------|--------|-----------------|
| **Left Drag** | Orbit | Camera rotates around origin |
| **Middle Drag** | Pan | Camera target moves on view plane |
| **Scroll Wheel** | Zoom | Camera distance changes |
| **Press ESC** | Reset | Camera returns to start position |
| **Press K** | Face Pick Test | Console shows "Picked face: 0" |
| **Press SPACE** | Camera Info | Prints position, target, distance |

### Step 5: Verify File Size

A successful rebuild DLL should be **~600 KB or larger** (not 507 KB).

Check the file size in `demo\bin\`:
```batch
dir C:\dev\makercad-extension\demo\bin\libmakercad.windows.template_debug.x86_64.dll
```

Smaller file size indicates incomplete build.

---

## Troubleshooting

### Build fails with "cannot open file .lib"
**Solution:** Check `godot-cpp` library path and verify it was built with `/MT` runtime.

### Build fails with "LNK2019: unresolved external symbol"
**Solution:** Verify all required OCCT libraries listed in CMakeLists.txt:
- TKPrim
- TKBRep
- TKG3d
- TKG2d
- TKGeomBase
- TKTopAlgo
- TKernel
- TKMath
- TKMesh

### Godot says "GDExtension dynamic library not found"
**Solution:** Ensure `demo/bin/makercad.gdextension` exists and points to correct DLL path.

### OCCT DLL not found at runtime
**Solution:** Run `copy_dlls.ps1` to copy all 9 OCCT DLLs to `demo/bin/`:
```batch
powershell -ExecutionPolicy Bypass -File copy_dlls.ps1
```

---

## What Changed Today

If you see these new features working, rebuild succeeded:

1. **Camera Pan** (Middle Drag) - NEW TODAY
   ```cpp
   if (m_panning) {
       // Pan in view plane using camera basis
       Vector3 forward = ...;
       Vector3 right = ...;
       Vector3 up = ...;
       m_target -= right * delta.x * pan_speed;
       m_target += up * delta.y * pan_speed;
   }
   ```

2. **Enhanced Ray Picking** - NEW TODAY
   ```cpp
   // Uses IntCurvesFace_Intersector for intersection
   IntCurvesFace_Intersector intersector(face, Precision::Angular());
   intersector.Perform(line, 0.0, 1e6);
   ```

3. **Screen-based Face Picking** - NEW TODAY
   ```cpp
   // New method exposed to GDScript
   int pick_face(const Vector2& screen_pos);
   ```

4. **Camera Finder** - NEW TODAY
   ```cpp
   void _find_camera(Node* node, Camera3D*& out_camera);
   ```

---

## Success Criteria

After rebuild, verify all of these work:

- [ ] Godot loads extension without errors in Output panel
- [ ] Orange test box renders with Phong shading
- [ ] Left drag orbits camera smoothly
- [ ] Middle drag pans camera correctly  ← NEW
- [ ] Scroll wheel zooms in/out
- [ ] Press K: Console prints "Picked face: 0"
- [ ] Press SPACE: Prints camera information

---

## Next Steps After Successful Build

Once Phase 2 is validated, start Phase 3:

1. Implement dirty flag rebuild system
2. Add history panel UI
3. Create extrude operation
4. Add operation preview (ghost mode)

---

**Rebuild Instructions End** - Run `rebuild_clean.bat` to fix Error 126
