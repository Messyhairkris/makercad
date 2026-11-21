# MakerCAD Proposed Improvements V1.3

Based on research from PDF inspiration materials:
- IntroductiontoSolidModeling.pdf
- SPM.pdf (Rossignac - Solid and Physical Modeling)
- Suggestive Interface for 3D Drawing.pdf (Igarashi)

## Section 1: Existing Proposals (V1.2)

### Start Menu & Project Management
- Welcome screen with recent projects
- New/Open/Save project workflow
- Project templates

### Circle Tool Enhancement
- 2-Point mode (endpoints define diameter)
- 3-Point mode (arc through 3 points)

### Additional Drawing Tools
- Arc (3-point, tangent)
- Polygon (N-sided regular)
- Ellipse
- Fillet/Chamfer

### UX Improvements
- Toolbar organization by category
- Keyboard shortcuts (Ctrl+Z undo, Delete, etc.)
- Dynamic dimensions

---

## Section 2: New Ideas from Research

### Suggestion Engine System (from Igarashi's Chateau)

Core idea: When user selects/highlights geometry, system suggests contextual operations as thumbnails.

**Implementation:**
```
SELECT geometry → SYSTEM analyzes context → SHOW thumbnail suggestions → USER picks
```

**Example scenarios:**
1. Select two perpendicular lines → Suggest "Make Rectangle"
2. Select closed loop → Suggest "Create Extrusion"
3. Select edge → Suggest "Fillet" or "Chamfer"
4. Select flat surfaces → Suggest "Pull/Extrude"
5. Select parallel lines → Suggest "Linear Pattern"

**Benefits:**
- Reduces menu complexity
- Learns user preferences over time
- Extensible via plugins

### Feature-Based Modeling (from Rossignac)

Higher-level operations beyond primitives:

**Standard Features:**
- Hole (counterbored, countersunk)
- Slot (rectangular, circular)
- Pocket (blind, through)
- Rib
- Boss/Pad
- Blend/Fillet
- Chamfer

**Feature Parameters:**
- Depth, diameter, angle
- Termination (blind, through, tangent)
- Anchor point

### Constraint-Based Dimensions

Instead of exact values, specify:
- Equality constraints (same as selected edge)
- Reference dimensions (e.g., "2x length of edge A")
- Geometric relationships (parallel, perpendicular, tangent)

### Advanced Operations (from Rossignac)

**Sweep Operations:**
- Extrude along path
- Revolve around axis
- Helix sweep

**Mesh Operations:**
- Subdivision (for smooth surfaces)
- Simplification (performance)
- Boolean operations on meshes

**Offset/Grow-Shrink:**
- Positive offset (grow)
- Negative offset (shrink)
- Constant-radius fillet via offset combination

### Enhanced UI Patterns

**Smart Snapping:**
- Highlight target → Guide snapping to parallel/congruent
- Touch midpoint → Perpendicular bisector snap
- Touch endpoint → Endpoint extension

**Prediction Lines:**
- Show ghost preview before committing
- Multiple candidates shown simultaneously
- Click to select, move to ignore

---

## Section 3: Implementation Priority

### Phase 1 (Quick Wins)
- [ ] Rectangle tool improvement (any 2 points → rectangle)
- [ ] Undo/Redo state machine
- [ ] Delete with confirmation

### Phase 2 (Core Features)
- [ ] Circle 2-Point and 3-Point modes
- [ ] Basic Fillet/Chamfer
- [ ] Grid snap toggle

### Phase 3 (Advanced)  
- [ ] Suggestion Engine prototype
- [ ] Feature-based operations
- [ ] Parametric constraints

---

## Technical Notes

### State Management
- CSG tree for boolean history
- Transaction-based undo
- Dirty flag for unsaved changes

### Performance
- Progressive mesh refinement
- View-dependent detail (LOD)

### Export Formats
- Standard export (OBJ, STL)
- CAD export (STEP, IGES)