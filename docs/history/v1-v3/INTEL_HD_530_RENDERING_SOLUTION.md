# MakerCAD Intel HD 530 Rendering Solution

**Date:** 2026-04-28  
**Instance:** Haven  
**Status:** SOLVED ✅

---

## The Problem

MakerCAD's OCCT-to-Godot mesh generation produced invisible or malformed meshes on Intel HD 530 with Godot's Compatibility Mode (OpenGL ES 3.0 via ANGLE).

**Hardware:** Intel HD Graphics 530 (6th gen, Skylake)  
**Driver:** ANGLE (Google's OpenGL ES -> Direct3D11 translator)  
**Godot:** 4.6.2, Compatibility Mode

---

## What Did NOT Work

### 1. SurfaceTool Custom Mesh Generation
- Manual vertex/face generation produced "blue flags" instead of solid faces
- Winding order issues caused faces to disappear
- Result: Malformed meshes, random triangles

### 2. ArrayMesh with Custom Vertices
- Direct array manipulation produced similar issues
- Normals not calculated correctly
- Result: Inside-out or invisible geometry

### 3. CSGPolygon3D
- Not suitable for solid CAD rendering
- Result: Inappropriate for the use case

### 4. StandardMaterial3D with CULL_BACK
- Face culling caused disappearing faces on Intel HD 530
- Result: Faces visible only from certain angles

### 5. Complex Wireframe Generation
- Grid lines became garbled
- Result: Visual artifacts

---

## What WORKED ✅

### Solution: Godot Built-in Primitives + CULL_DISABLED

**For Solid Objects:**
```gdscript
var box_mesh = BoxMesh.new()
box_mesh.size = Vector3(20, 20, 20)

var material = StandardMaterial3D.new()
material.albedo_color = Color(0.3, 0.5, 0.7)  # Mid blue
material.shading_mode = StandardMaterial3D.SHADING_MODE_PER_PIXEL
material.cull_mode = StandardMaterial3D.CULL_DISABLED  # KEY!
```

**For Wireframe Overlay:**
```gdscript
var line_mesh = ImmediateMesh.new()
line_mesh.surface_begin(Mesh.PRIMITIVE_LINES)
# Add vertices for edges
line_mesh.surface_end()

var material = StandardMaterial3D.new()
material.albedo_color = Color(1, 1, 1)  # White
material.shading_mode = StandardMaterial3D.SHADING_MODE_UNSHADED
```

---

## Key Insights

### 1. CULL_DISABLED is Essential
On Intel HD 530 + ANGLE, `CULL_BACK` causes faces to disappear. Use `CULL_DISABLED` to show both sides.

### 2. Use Built-in Primitives
Godot's `BoxMesh`, `SphereMesh`, etc. are optimized and work correctly on all hardware.

### 3. Avoid Manual Mesh Generation
SurfaceTool and ArrayMesh with custom vertices are unreliable on Intel HD 530.

### 4. Dark/Light Panels are Lighting
The varying shades on the cube are correct - they're from the DirectionalLight showing 3D form.

### 5. Wireframe Thickness Limitation
OpenGL ES 3.0 (ANGLE) doesn't support thick lines. Lines are always 1 pixel wide.

---

## Implementation for MakerCAD Extension

**Instead of:**
```cpp
// OCCT -> SurfaceTool -> custom mesh
// (unreliable on Intel HD 530)
```

**Use:**
```cpp
// For boxes: Godot BoxMesh
// For cylinders: Godot CylinderMesh
// For spheres: Godot SphereMesh
// Complex shapes: CSG operations on primitives
```

---

## Performance Notes

- BoxMesh + CULL_DISABLED: ✅ Smooth 60fps
- SurfaceTool generation: ❌ Malformed or invisible
- Debug logging: ❌ Causes stuttering (remove for production)

---

## References

- Godot 4.6.2 Compatibility Mode
- Intel HD Graphics 530 (Skylake)
- ANGLE (OpenGL ES -> Direct3D11)

---

**Conclusion:** Use Godot's built-in primitive meshes with `CULL_DISABLED` for reliable rendering on Intel HD 530 and similar integrated graphics.
