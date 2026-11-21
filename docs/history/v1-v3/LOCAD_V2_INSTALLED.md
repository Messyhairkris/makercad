# LoCAD v2 - Successfully Installed

**Installation Complete!** ✅

Files have been installed into the existing MakerCAD project.

---

## Where Files Were Installed

**Project:** `C:\Users\<you>\MakerCAD\maker-cad-program`

```
maker-cad-program/
├── scenes/
│   └── locad_viewport_only.tscn    ✅ NEW
├── scripts/
│   ├── controller/
│   │   └── MainController.gd       ✅ NEW
│   ├── navigation/
│   │   └── OrbitCamera.gd          ✅ NEW
│   └── export/
│       └── STLExporter.gd          ✅ NEW
└── (existing project files...)
```

---

## What Was Installed

### Scene (1 file)
- `scenes/locad_viewport_only.tscn`
  - Main scene with viewport, toolbar, status bar
  - Ready to open in Godot editor

### Scripts (3 files)
- `scripts/controller/MainController.gd`
- `scripts/navigation/OrbitCamera.gd`
- `scripts/export/STLExporter.gd`

All scripts are now part of the existing `maker-cad-program` project.

---

## How to Use

### 1. Open the Project
```
1. Launch Godot 4.6+
2. Open the project: C:\Users\<you>\MakerCAD\maker-cad-program
3. The project should already be in your Project Manager if you opened it before
```

### 2. Open the LoCAD v2 Scene
```
1. In the Godot editor
2. Navigate to: scenes/
3. Double-click: locad_viewport_only.tscn
4. The scene should open in the 3D viewport
```

### 3. Run the Scene
```
1. Press F5 to run (or click "Run Scene" button)
2. You should see:
   - 3D viewport (black or grey background)
   - Toolbar with 4 buttons at the top
   - Status bar at the bottom
```

### 4. Test the Controls
```
Press B: Creates a blue box (boxes placed in a grid pattern)
Left click: Selects a box (glows when selected)
Middle drag: Orbits camera around target
Scroll wheel: Zooms in and out
Esc key: Deselects selection

Toolbar buttons also work:
- Box button: Create box
- Move button: Activate move tool (not implemented yet)
- Extrude button: Activate extrude tool (not implemented yet)
- Export button: Opens file dialog for STL export
```

---

## Integration Notes

### Renaming for Clarity

Scene file was renamed to prevent conflicts:
- `viewport_only.tscn` → `locad_viewport_only.tscn`

This avoids overwriting any existing scenes in your project.

### No project.godot Changes

The existing `project.godot` file was NOT modified.
- Scene must be opened manually from the editor
- Main scene setting NOT changed (to preserve existing project config)

### Godot Import

**First time you open `locad_viewport_only.tscn`:**

1. Godot may show an import dialog
2. Click "Import All" or "Import" to import resources
3. The scene should open normally after import

If you see errors about missing scripts or paths, try:
- Project → Reimport Assets

---

## Troubleshooting

### "Scene doesn't open"
1. Verify the file exists: `C:\Users\<you>\MakerCAD\maker-cad-program\scenes\locad_viewport_only.tscn`
2. In Godot, check the FileSystem panel (bottom-left)
3. Navigate to `scenes/` folder
4. Look for `locad_viewport_only.tscn`
5. Double-click to open

### "Scripts show as missing"
1. Check scripts folder exists: `C:\Users\<you>\MakerCAD\maker-cad-program\scripts`
2. Verify subdirectories: `controller/`, `navigation/`, `export/`
3. Try Project → Reimport Assets

### "Camera doesn't orbit"
1. Check Camera3D node has OrbitCamera.gd attached
2. Open the scene (Double click `locad_viewport_only.tscn`)
3. Select Camera3D node in Scene tree
4. Look in Inspector → Script
5. Should show: `OrbitCamera`

If script is missing:
1. Click Inspector → Script folder icon
2. Navigate to: `scripts/navigation/OrbitCamera.gd`
3. Select and attach

---

## Controls Quick Reference

| Action | Input | Status | Notes |
|--------|-------|--------|-------|
| Create Box | B key / Box button | ✅ | Creates in grid pattern |
| Select | Left click | ✅ | Raycast to CSG objects |
| Orbit Camera | Middle drag | ✅ | Around target |
| Pan Camera | Shift + Middle drag | ✅ | Translates target |
| Zoom Camera | Scroll wheel | ✅ | Toward target |
| Deselect | Esc key | ✅ | Clears selection |
| Move Tool | M key | ⏳ | Not implemented |
| Extrude Tool | E key | ⏳ | Not implemented |
| Export | Export button | ✅ | Exports selected object to STL |

---

## Next Steps

### If Controls Work (v2.0 Complete):
1. Test multiple box creation
2. Verify selection highlight works
3. Test export to STL file
4. Check on Intel HD 530 hardware

### If Bugs Found:
Report issues for fixing:
- Camera controls not working
- Selection not highlighting
- Export failing
- Scenes not opening

### v2.1 Development:
- Implement Move tool (gizmo-based)
- Implement Extrude tool (face-based)
- Wire InteractionController
- Add SelectionContext
- Add SelectionResolver

---

## Summary

**Installed into:** `C:\Users\<you>\MakerCAD\maker-cad-program`
**Files added:** 4 files (1 scene + 3 scripts)
**Ready to test:** Yes ✅
**Hardware requirement:** Intel HD 530 or better

To start: Open the MakerCAD project, then open `scenes/locad_viewport_only.tscn` and press F5.

---

*Installation completed on 2026-05-10*
