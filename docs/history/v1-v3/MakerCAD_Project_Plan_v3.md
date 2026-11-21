# MakerCAD — Project Architecture & Planning Document
**A professional-grade, open CAD application built on OpenCASCADE Technology + Godot 4**
*Version 3.0 — April 2026 | Final pre-build design review*
*For expert comparison and ongoing reference*

---

## Document History

| Version | Date | Summary |
|---|---|---|
| 1.0 | April 2026 | Initial architecture plan |
| 2.0 | April 2026 | Linear history adopted, topological naming documented, AI timing revised, UX strategy added, rendering challenges expanded |
| 3.0 | April 2026 | Dirty flag rebuild system, functional pipeline hashing, selection intent recording, failure recovery UX, file versioning, preview mode, freeze history, low-fidelity drag, logging system, OCCT boundary rule, AI reframed as privileged client |

---

## Table of Contents

1. [Project Mission & Design Mantra](#1-project-mission--design-mantra)
2. [Strategic Framing](#2-strategic-framing)
3. [Technology Stack](#3-technology-stack)
4. [Hardware Baseline & Constraints](#4-hardware-baseline--constraints)
5. [Core Data Structures](#5-core-data-structures)
6. [System Architecture](#6-system-architecture)
7. [Known Hard Problems & Solutions](#7-known-hard-problems--solutions)
8. [MCP Ecosystem](#8-mcp-ecosystem)
9. [Phase 0 — Environment Setup](#9-phase-0--environment-setup)
10. [Development Phases](#10-development-phases)
11. [UX Strategy](#11-ux-strategy)
12. [Open Decisions](#12-open-decisions)
13. [Expert Review Q&A](#13-expert-review-qa)
14. [Questions for Further Review](#14-questions-for-further-review)

---

## 1. Project Mission & Design Mantra

### Name
**MakerCAD** — signals the target user (makers, learners, non-professionals) and the philosophy (accessible, empowering, community-oriented).

### Vision
A free, open-source, parametric CAD application that runs on modest consumer hardware, empowering non-professionals to learn and use solid modeling in the style of Fusion 360 — without Fusion 360's pricing, cloud dependency, or hardware requirements.

### Core Goals
- Runs on 8 GB RAM + integrated Intel graphics (HD 530-class or better)
- Parametric history-based solid modeling, scoped for clarity and stability over completeness
- First successful model in under 10 minutes for a user with no CAD experience
- Full touchscreen + pen support alongside mouse/keyboard
- AI-assisted modeling — implemented late in development, designed for from the start
- Open source, no licensing fees

### Non-Goals (v1.0)
Feature parity with Fusion 360 · Cloud collaboration · Simulation/FEA · CAM/toolpath · Photorealistic rendering · Multi-part assembly constraints

### Design Mantras
> "Ship a dumb but stable CAD before a smart but fragile one."
> "Every decision must reduce future complexity, not increase capability."

If a proposed feature makes the system harder to reason about, introduces edge cases, or complicates rebuild logic — it is wrong for v1.

---

## 2. Strategic Framing

**MakerCAD is not a CAD engine.**

It is: **a simplified parametric modeling experience built on OCCT.**

The distinction is critical. A CAD engine tries to match professional tools feature-for-feature. A simplified modeling experience is curated — it exposes the right operations in the right order for a learner to succeed, deliberately withholding complexity that causes confusion or failure.

Scope creep driven by "but Fusion 360 can do X" is the primary project risk. Every proposed feature must pass the filter: *does this help a non-professional succeed faster?* If not, defer it.

---

## 3. Technology Stack

### Geometry Kernel — OpenCASCADE Technology (OCCT)
- **Version:** 7.9.x — prebuilt Windows binary (VC143). Do NOT build from source on the reference machine.
- **Stay on 7.9, not 8.x.** Stability, documentation, and binary availability outweigh any 8.x improvements. Bleeding edge is wrong for this project.
- **Role:** All solid modeling, boolean operations, tessellation, and file I/O. OCCT is used as a pure geometry engine — its AIS/V3d visualization system is explicitly NOT used.
- **Hard rule:** No raw OCCT types (`TopoDS_Shape`, `BRep_Builder`, etc.) cross the GDExtension API boundary. Everything that crosses the boundary is expressed as IDs, serialized parameter structs, or plain data. This rule is what makes a future IPC migration possible.
- **Reference implementation:** FreeCAD source code. Nearly every OCCT operation MakerCAD needs is already implemented there.

### Rendering & UI — Godot 4.6.2 (Standard / Non-.NET)
- **Version:** 4.6.2 standard build. The .NET/C# version is NOT required. GDExtension is a pure C++ API.
- **Renderer:** **Compatibility mode (OpenGL 3.3).** Non-negotiable for the hardware baseline. Must be set at project creation — cannot be changed afterward.
- **Role:** 3D viewport, all UI panels, touch/pen input, HTTP client for AI

#### Godot Rendering Challenges (require early resolution in Phase 1)

| Problem | Root Cause | Required Solution |
|---|---|---|
| Thick wireframes | `glLineWidth` > 1px is undefined behavior in OpenGL 3.3 on many drivers | Geometry-based line rendering: quad strips extruded along edges in a vertex shader |
| Z-fighting on overlays | Sketch lines and grid share depth buffer with solid faces | Polygon offset (`DEPTH_OFFSET` in Godot shader) or separate depth pass |
| Edge highlighting | Selected edges must visually "pop" above face geometry | Stencil buffer outline pass or geometry duplication with scale offset |
| Depth bias for guides | Workplane grid must appear "above" geometry | Controlled depth bias in grid shader; test on HD 530 Compatibility mode specifically |
| No compute shaders | Compatibility renderer excludes compute | Not needed for CAD — tessellation stays on CPU |
| Line rendering on Intel | Intel HD drivers have known quirks with OpenGL line primitives | Avoid `GL_LINES` entirely; use geometry-based quads |

All rendering approaches must be validated on the reference hardware (Intel HD 530, Compatibility mode) during Phase 1 before the codebase grows around assumptions.

### Bridge — GDExtension (C++)
- **Library:** godot-cpp, checked out to the exact tag matching Godot 4.6.2. Version mismatch = silent crashes.
- **Build system:** CMake + Ninja + MSVC v143
- **API boundary rule:** No raw OCCT types cross this boundary. See Section 3 above.

#### GDExtension vs. IPC Decision

| Approach | Pros | Cons |
|---|---|---|
| GDExtension (in-process) — **v1** | Fast, simple, no serialization overhead | OCCT crash = entire app crash |
| IPC (separate OCCT process) — **future** | Isolated crash, modular, independently testable | Significant complexity, per-operation latency |

**v1 decision: GDExtension.** The API is designed as a clean logical boundary from day one. The IPC migration trigger is: frequent OCCT-caused crashes in user hands, OR operations regularly exceeding 500ms and blocking the UI despite async mitigations.

### Constraint Solver — planegcs (NOT OCCT's solver)
- **Source:** `FreeCAD/FreeCAD/src/Mod/Sketcher/App/planegcs/` (LGPL)
- **Integration:** Git submodule, compiled into the GDExtension
- **v1 scope:** Limited constraint set only (see Phase 4). planegcs can fail silently on over-constrained systems; the UX layer must detect and surface this.

### AI Integration — Anthropic API (privileged client, not core system)
- **Architectural framing:** AI is a privileged client of the operation API — it calls the same operations any other client would call, through the same interface. It is not a core subsystem. The architecture is not constrained to serve AI; AI is served by a well-designed architecture.
- **Implementation timing:** Phase 7 — after file I/O is stable and the system has been robustness-hardened
- **Design timing:** Phase 1 — history model and operation API are designed to be AI-legible from the start (clean JSON, deterministic, well-named), because retrofitting this later is painful
- **Transport:** GDScript HTTPRequest → api.anthropic.com/v1/messages
- **Model:** claude-sonnet-4

---

## 4. Hardware Baseline & Constraints

### Reference Machine (minimum viable target)

| Component | Specification |
|---|---|
| CPU | Intel Core i5-6600 @ 3.30 GHz — 4 cores, no HT, Skylake 2015 |
| RAM | 8 GB (7.70 GB usable), shared with iGPU |
| GPU | Intel HD Graphics 530 — no Vulkan, OpenGL 4.4 ✓ |
| OS | Windows 10 64-bit |
| Input | 10-point multitouch + pen (Godot native support) |

### GPU Constraint
Intel HD 530 has no Vulkan support on Windows (Intel official policy for 6th-gen). Godot's Forward+ and Mobile renderers require Vulkan. **Compatibility (OpenGL 3.3) is mandatory and sufficient** — CAD rendering needs solid faces, wireframes, selection highlighting, and a grid. No ray tracing or GI required. The OpenGL constraint is a strategic asset: MakerCAD runs on hardware that excludes most competing tools.

### RAM Budget

| Context | Estimate |
|---|---|
| Windows 10 background | ~2.5 GB |
| VS Code | ~400 MB |
| Godot editor | ~300 MB |
| GDExtension + OCCT runtime | ~300 MB |
| Available headroom (dev) | ~4 GB ✓ |
| Peak during MSVC linking | ~3–4 GB — close all other apps |

**Rule:** Never compile OCCT from source on this machine. Use the prebuilt binary.

### Runtime Performance Targets

| Metric | Target |
|---|---|
| Viewport frame rate | 30+ FPS at 1080p |
| Max tessellated mesh (interactive) | ~100,000 triangles |
| App RAM at runtime | < 600 MB |
| Cold start | < 3 seconds |
| Simple boolean op | < 1 second user-visible feedback |
| STEP file load (typical part) | < 5 seconds |
| Rebuild of 50-op history | < 2 seconds with dirty flag system |

### Performance Interaction Modes

| Mode | Trigger | Geometry Quality |
|---|---|---|
| **Full quality** | Mouse/pen released, operation committed | Full tessellation at configured deflection |
| **Low-fidelity** | Any drag operation in progress | Bounding box or aggressively decimated mesh |
| **Preview/ghost** | Operation parameter being set (pre-commit) | Mid-quality tessellation, semi-transparent |

The low-fidelity drag mode is non-negotiable for the HD 530 + 8 GB target. Without it, interactive manipulation will feel broken.

---

## 5. Core Data Structures

*These must be fully designed before any modeling operation code is written. The data structures are the product.*

### HistoryNode — The Fundamental Unit

```cpp
struct HistoryNode {
    // Identity
    std::string id;           // "op_003" — stable, never reused
    std::string type;         // "extrude", "fillet", "boolean_cut", etc.

    // Parameters (all serializable to JSON, no OCCT types)
    nlohmann::json params;    // {"distance_mm": 20.0, "direction": [0,0,1]}

    // Input references (IDs only — no raw shapes)
    std::vector<std::string> input_ids;       // upstream operation IDs
    std::vector<SelectionIntent> edge_refs;   // geometric intent, not OCCT indices
    std::vector<SelectionIntent> face_refs;

    // Rebuild state (dirty flag system)
    bool dirty;               // true = must rebuild before use
    std::string output_shape_id;  // ID into the shape store (never a raw TopoDS_Shape)
    std::string output_hash;      // functional pipeline hash (see Section 7)

    // UI state
    bool broken;              // true = rebuild failed or reference unresolvable
    std::string broken_reason;    // human-readable, shown in history panel
    bool visible;             // user can hide operations

    // Metadata
    std::string created_at;
    std::string modified_at;
};
```

### SelectionIntent — Geometric Heuristic Record

Instead of storing unstable OCCT topology indices, store the geometric intent of what the user selected. This is what allows the heuristic matcher to find the "same" entity after a rebuild.

```json
{
  "type": "edge_loop",
  "criteria": {
    "normal": [0.0, 0.0, 1.0],
    "position_hint": [12.5, 8.0, 20.0],
    "loop_type": "outer",
    "approximate_length": 52.4,
    "approximate_centroid": [12.5, 8.0, 20.0]
  },
  "confidence_threshold": 0.85
}
```

On rebuild, the heuristic matcher finds the entity that best matches all criteria. If no candidate exceeds `confidence_threshold`, the node is marked `broken`.

### The Linear History Model

The model is a flat ordered list of `HistoryNode` entries. The list is the truth of the model — geometry is always derived by executing the list.

```
history[0]: create_sketch  (plane=XY)
history[1]: extrude         (sketch_ref=0, distance=20mm)
history[2]: fillet          (edge_refs=[intent_A], radius=2mm)
history[3]: boolean_cut     (body=1, tool=new_sketch_extrude)
```

**Rebuild rules:**
- Editing `history[1]` marks `history[1]`, `history[2]`, `history[3]` as `dirty`
- Rebuild walks the list from index 0, skipping non-dirty nodes, executing dirty ones
- Each executed node stores its output shape ID and updates its hash
- Non-dirty nodes use their cached output — O(k) rebuild where k = number of dirty nodes, not O(N)

### Functional Pipeline Hash

OCCT shapes do not have stable hashes. Do not attempt to hash geometry directly. Instead, use a functional pipeline hash:

```
hash(node_n) = hash(
    node_n.type +
    serialize(node_n.params) +
    hash(node_n-1)   // upstream hash, recursively
)
```

This means:
- Identical operation chains with identical parameters produce identical hashes
- Any parameter change propagates hash changes downstream automatically
- No floating-point geometry comparison required
- Cache key is `hash(node_n)` → cached `ArrayMesh`

Cache miss = re-tessellate. Cache hit = return stored `ArrayMesh`. This is correct and efficient.

### File Format — Versioned from Day One

```json
{
  "format": "makercad",
  "version": 1,
  "created_with": "MakerCAD 0.1.0",
  "history": [
    {
      "id": "op_001",
      "type": "create_sketch",
      "params": { "plane": "XY" },
      "input_ids": [],
      "edge_refs": [],
      "face_refs": []
    }
  ],
  "brep_cache": [
    {
      "shape_id": "shape_a3f9c2",
      "encoding": "brep_base64",
      "data": "..."
    }
  ]
}
```

**Migration handler system** — stub this from day one, even if it does nothing:

```cpp
class ProjectMigrator {
public:
    static Project migrate(const nlohmann::json& raw) {
        int version = raw["version"].get<int>();
        switch (version) {
            case 1: return migrate_v1(raw);
            // case 2: return migrate_v2(migrate_v1(raw)); etc.
            default: throw UnsupportedVersionError(version);
        }
    }
private:
    static Project migrate_v1(const nlohmann::json& raw);
};
```

Future versions modify the migration chain, never the loading code. This prevents version rot.

---

## 6. System Architecture

### Layer Overview

```
┌──────────────────────────────────────────────────────────────┐
│                   Godot 4 — UI & Rendering                    │
│  [Viewport]  [History Panel]  [Sketch Tools]  [AI Panel]    │
│  [Preview/Ghost mode]  [Failure Recovery UX]  [Touch Input] │
└────────────────────────┬─────────────────────────────────────┘
                         │ GDExtension API
                         │ (IDs + plain data only — no OCCT types)
┌────────────────────────┴─────────────────────────────────────┐
│                  GDExtension Bridge — C++                     │
│  [HistoryEngine]  [ShapeStore]  [Tessellator (async)]       │
│  [HeuristicMatcher]  [FileIO]  [OperationPreview]           │
│  [Logging]                                                   │
└────────────────────────┬─────────────────────────────────────┘
                         │ OCCT C++ API (internal only)
┌────────────────────────┴─────────────────────────────────────┐
│          OpenCASCADE Technology — Geometry Kernel             │
│  [BRep Kernel]  [Boolean Ops]  [Modeling Ops]  [Mesh]       │
│  [planegcs — embedded]  [STEP/IGES/STL readers/writers]     │
└──────────────────────────────────────────────────────────────┘
```

### HistoryEngine — The Core Controller

The `HistoryEngine` owns the history list and the rebuild algorithm:

```cpp
class HistoryEngine {
public:
    // Mutating operations — always mark dirty and trigger rebuild
    std::string add_operation(HistoryNodeSpec spec);
    void modify_parameter(const std::string& node_id, const std::string& param, nlohmann::json value);
    void delete_operation(const std::string& node_id);
    void undo();
    void redo();

    // Rebuild — partial, dirty-flag driven
    void rebuild();   // called after any mutation; async in Phase 6+

    // Read-only state for UI and AI
    nlohmann::json serialize_history() const;
    std::vector<HistoryNodeView> get_history_view() const;

    // Preview — does not commit to history
    ShapeID preview_operation(HistoryNodeSpec spec) const;

private:
    std::vector<HistoryNode> history_;
    ShapeStore shape_store_;

    void mark_dirty_from(int index);
    void rebuild_dirty_nodes();
};
```

### ShapeStore — The OCCT Containment Boundary

All `TopoDS_Shape` objects live inside the `ShapeStore` and never leave it. External code only ever holds `ShapeID` strings.

```cpp
class ShapeStore {
public:
    ShapeID store(TopoDS_Shape shape, const std::string& hash);
    const TopoDS_Shape& get(ShapeID id) const;
    bool has(const std::string& hash) const;
    void evict_unused();

private:
    std::unordered_map<ShapeID, TopoDS_Shape> shapes_;
    std::unordered_map<std::string, ShapeID> hash_to_id_;
};
```

This is the architectural enforcement of the "no OCCT types cross the boundary" rule.

### Logging System (from day one)

Debugging OCCT failures is painful without logs. Structured logging from the first line of code:

```cpp
class MakerLog {
public:
    static void op_start(const std::string& op_type, const nlohmann::json& params);
    static void op_success(const std::string& node_id, double duration_ms);
    static void op_failure(const std::string& node_id, const std::string& reason);
    static void rebuild_start(int dirty_count, int total_count);
    static void rebuild_complete(double duration_ms);
    static void tessellation_start(const std::string& shape_id, double deflection);
    static void tessellation_complete(const std::string& shape_id, int triangle_count, double duration_ms);
    static void cache_hit(const std::string& hash);
    static void cache_miss(const std::string& hash);
};
```

Log to a rotating file in `%APPDATA%\MakerCAD\logs\`. Include in crash reports. Essential for diagnosing OCCT-specific failures that are otherwise opaque.

### Operation Preview Mode

Before an operation is committed to history, the user sees a "ghost" preview:

1. User selects operation type and begins entering parameters
2. `HistoryEngine::preview_operation(spec)` executes the operation against the current model state but does NOT add it to history
3. Result shape is tessellated at mid-quality and rendered semi-transparent (the "ghost")
4. User adjusts parameters; preview updates in near-real-time (debounced at ~150ms)
5. User confirms → operation added to history; ghost replaced by real mesh
6. User cancels → ghost discarded; history unchanged

This dramatically improves UX — users see what they're about to create before committing.

---

## 7. Known Hard Problems & Solutions

### Problem 1 — Rebuild Cost Explosion (CRITICAL — new in v3.0)

**The problem:** A linear history + full replay means N operations = O(N) rebuild on every edit. At 10 ops: fine. At 50 ops: noticeable. At 100+ ops: painful on the HD 530.

**Mesh caching (already planned) does NOT solve this.** Mesh caching prevents unnecessary re-tessellation, but OCCT solid modeling operations still run. Each OCCT boolean op, fillet, or extrude takes real CPU time. Caching helps the GPU; it does not help the rebuild.

**Solution: Dirty flag partial rebuild.** Described in Section 5 (HistoryNode `dirty` field). Only dirty nodes rebuild. An edit to operation [47] in a 100-op model rebuilds ops [47]–[99], not ops [0]–[99]. With good caching, most models rebuild only the last few operations even after complex edits.

**Rebuild performance targets:**
- 10-op model: < 200ms (synchronous acceptable)
- 50-op model: < 1 second (async preferred)
- 100-op model: < 3 seconds (async required)

### Problem 2 — Topological Naming (CRITICAL)

OCCT does not guarantee stable face/edge/vertex identifiers after operations. The "top edge" selected in op [2] may be a different entity after op [3] modifies the shape.

**MakerCAD v1 strategy: SelectionIntent + geometric heuristic + fail gracefully.**

The `SelectionIntent` struct (see Section 5) stores a rich geometric description of the selected entity at the time of selection. On rebuild, the `HeuristicMatcher` finds the entity that best matches the stored criteria. If confidence falls below threshold, the operation is marked `broken`.

This is the difference between "probably works" and "usually works."

**What is NOT attempted in v1:** A full persistent naming algorithm (as attempted by FreeCAD and others). This is a research problem. The heuristic approach is honest about its limitations and fails safely.

### Problem 3 — Feature Tree Retrofit Impossibility (CRITICAL)

The `HistoryNode` structure and rebuild algorithm must be finalized before any modeling operation code is written. The data structure IS the product. Retrofitting parametric behavior after the fact requires rewriting everything.

**Resolution:** Section 5 of this document defines the complete structure. This is the binding design contract. No modeling operation code is written until this section has been reviewed and accepted.

### Problem 4 — Model Failure Recovery UX (CRITICAL — new in v3.0)

**The problem:** A user sees a red "broken" indicator on a history operation. Then what? Without a recovery flow, they are stuck.

**Required UX flow:**
```
1. User sees operation marked broken (red indicator) in history panel
2. History panel shows: broken_reason text + "Fix" button
3. User clicks "Fix"
4. App enters re-selection mode:
   - All parameters from the broken operation are preserved and shown
   - The user is prompted only to re-select the geometry reference(s)
   - Ghost preview shows what the operation would produce with new selection
5. User selects new geometry → SelectionIntent is updated → operation re-runs
6. If successful: operation returns to normal state
7. If unsuccessful: stays broken, user can try again or delete the operation
```

Parameters are never lost during failure recovery. Only geometry references need re-selection.

### Problem 5 — Constraint Solver UX (HIGH)

**Known planegcs failure modes:**
- Silent failure on certain over-constrained configurations (no exception thrown, solver returns unchanged state)
- Oscillation in edge cases with conflicting distance and angle constraints simultaneously

**MakerCAD v1 constraint strategy:**
- Implement only: coincident, horizontal, vertical, distance, angle
- Defer: tangent, equal, parallel, perpendicular, symmetric
- **No constraint editing graph.** Constraints are applied in the order the user adds them. No re-solving of the entire constraint system on every change — sequential application only. This limits solver complexity at the cost of some flexibility. Acceptable for v1.
- **Auto-constrain lite (suggest, don't enforce):** After each drawn stroke, suggest the most likely constraint (horizontal, vertical, or coincident with an existing point). User taps to accept. This reduces error cases significantly.
- **Visual state feedback:** Green = fully constrained, Blue = under-constrained, Red = over-constrained. Per-constraint color coding on the canvas.
- **Conflict detection:** Poll planegcs return state after every constraint add. If solver signals failure, highlight the new constraint in red immediately and offer to remove it.
- **planegcs silent failure detection:** Compare vertex positions before and after solve. If positions are unchanged but the model is not fully constrained, flag a solver warning.

### Problem 6 — Freeze History (NEW — scalability escape valve)

When a model accumulates 100+ operations, rebuild time becomes a UX liability even with dirty flags. The "freeze history" operation provides a scalability escape:

1. User selects "Freeze History" from the menu
2. All operations up to the current state are collapsed into a single "frozen" node
3. The frozen node stores the final `TopoDS_Shape` directly — no replay required
4. The user continues adding operations from the frozen state
5. Rebuild only touches operations after the freeze point

This is the v1-appropriate alternative to a full DAG system. It sacrifices the ability to edit pre-freeze operations (they are permanently baked) in exchange for eliminating long rebuild chains.

**Freeze is presented as a tool, not an automatic system.** The user decides when to freeze.

---

## 8. MCP Ecosystem

### Available Godot MCP Servers

| Server | Author | Capabilities | Role |
|---|---|---|---|
| `godot-mcp` | Coding-Solo | Launch editor, run projects, create scenes, add nodes | Primary dev tooling |
| `Claude-GoDot-MCP` | DaRealDaHoodie | 170 tools: scene ops, script editing, signals, shaders, audio | Comprehensive dev assistant |
| `godot-mcp-docs` | Nihilantropy | Serves Godot API docs to LLMs | AI has accurate Godot reference |

### Available CAD/3D MCP Servers

| Server | Author | Capabilities | Role |
|---|---|---|---|
| `openscad-mcp` | quellant | OpenSCAD 3D modeling + rendering | CAD MCP architecture reference |
| `CAD-MCP` | daobataotie | DXF/CAD workflow tools | Tool naming reference |
| Revit MCP | Various | Autodesk Revit via natural language | Tool schema design reference |

### MakerCAD Custom CAD MCP Server (Phase 7)

Tool definitions are designed in Phase 1 alongside the history model. Implementation is Phase 7.

```
get_model_state()              → full history JSON for AI context
create_sketch(plane)           → sketch_id
add_constraint(type, refs)     → constraint_id
close_sketch()                 → validates and closes active sketch
extrude(sketch_id, dist_mm)    → op_id
revolve(sketch_id, axis, deg)  → op_id
boolean_cut(body_id, tool_id)  → op_id
boolean_union(body_id, tool_id)→ op_id
add_fillet(edge_refs, r_mm)    → op_id
add_chamfer(edge_refs, d_mm)   → op_id
modify_param(op_id, key, val)  → triggers dirty rebuild
undo(steps)                    → list of reverted op_ids
redo(steps)                    → list of restored op_ids
export_step(filepath)          → success/error
import_step(filepath)          → list of imported op_ids
```

---

## 9. Phase 0 — Environment Setup (Windows 10)

### Installation Order

| # | Tool | Source | Default Path | Verify |
|---|---|---|---|---|
| 1 | Intel GPU driver (latest DCH) | intel.com | System driver | Device Manager |
| 2 | Git | git-scm.com | `C:\Program Files\Git\` | `git --version` |
| 3 | Visual Studio 2022 Community | visualstudio.microsoft.com | `C:\Program Files\Microsoft Visual Studio\2022\Community\` | `cl` in VS Dev Prompt |
| 4 | CMake 3.27+ | cmake.org | `C:\Program Files\CMake\bin\` | `cmake --version` |
| 5 | Ninja | github.com/ninja-build | `C:\ninja\ninja.exe` (add to PATH) | `ninja --version` |
| 6 | Python 3.11+ | python.org | `%LOCALAPPDATA%\Programs\Python\Python311\` | `python --version` |
| 7 | SCons | `pip install scons` | Python scripts dir | `scons --version` |
| 8 | Node.js LTS | nodejs.org | `C:\Program Files\nodejs\` | `node --version` |
| 9 | **Godot 4.6.2 Standard** | godotengine.org | `C:\Godot\` | Launch → About |
| 10 | godot-cpp | github.com/godotengine/godot-cpp | `C:\dev\godot-cpp\` | `git tag` (match 4.6.2) |
| 11 | OCCT 7.9 Prebuilt Binary | dev.opencascade.org | `C:\OpenCASCADE\` | `dir C:\OpenCASCADE\inc` |
| 12 | Claude Desktop | claude.ai/download | `%LOCALAPPDATA%\AnthropicClaude\` | Launch → version |
| 13 | VS Code | code.visualstudio.com | `%LOCALAPPDATA%\Programs\Microsoft VS Code\` | `code --version` |
| 14 | FreeCAD (reference) | freecad.org | `C:\Program Files\FreeCAD 1.0\` | Launch → About |

### Critical Notes

**VS 2022:** "Desktop development with C++" workload + MSVC v143 only. No additional workloads.

**Godot renderer:** Choose "Compatibility (OpenGL 3.3)" at project creation. Cannot be changed after.

**godot-cpp:** Must be checked out to exact `godot-4.6.2` tag. Run `git checkout godot-4.6.2` inside the clone.

**OCCT:** Prebuilt binary only. Never `cmake --build` OCCT on the reference machine.

**MCP config:** `%APPDATA%\Claude\claude_desktop_config.json`

### Verification Script

```powershell
Write-Host "=== Tools ===" -ForegroundColor Cyan
git --version
cmake --version | Select-Object -First 1
ninja --version
python --version
scons --version | Select-Object -First 1
node --version
npm --version
Write-Host "=== Paths ===" -ForegroundColor Cyan
Test-Path "C:\OpenCASCADE"
Test-Path "C:\dev\godot-cpp"
Test-Path "C:\Godot"
Write-Host "=== Done ===" -ForegroundColor Cyan
```

**Phase 0 exit criteria:** All checks green. Blank Godot project opens with "Compatibility" shown in the renderer label. No errors on launch.

---

## 10. Development Phases

### Phase 0 — Environment & Verification *(current)*
Install all tools. Run verification. Create Godot 4.6.2 Compatibility project. Build godot-cpp static libraries.

**Exit:** `hello_extension.dll` loads in Godot, prints "Hello from OCCT" to output panel.

---

### Phase 1 — GDExtension Skeleton + First Shape + Design Lock

**Technical work:**
- CMake build system: godot-cpp + OCCT headers + OCCT .lib files
- `ShapeStore` class (OCCT containment boundary)
- Basic `HistoryEngine` skeleton (add/serialize — no rebuild yet)
- OCCT box → `BRepMesh_IncrementalMesh` → Godot `ArrayMesh`
- Phong-shaded box displayed in Godot viewport

**Rendering tests (mandatory this phase):**
- Thick line rendering approach validated on HD 530
- Z-fighting mitigation confirmed working
- Edge highlight approach selected and tested
- Depth bias for grid confirmed

**Design work (mandatory before Phase 3):**
- `HistoryNode` structure finalized (see Section 5)
- `SelectionIntent` structure finalized
- Functional pipeline hash algorithm documented
- MCP tool API schema documented
- File format schema documented with migration stub

**Exit:** Shaded box in Godot, correct lighting. Selected face highlights. All rendering approaches confirmed on reference hardware. History model spec signed off.

---

### Phase 2 — Viewport & Camera
Orbit (left drag) / pan (middle drag / 3-finger) / zoom (scroll / pinch). Workplane grid with depth bias shader. Ray-to-BRep picking using `BRepIntCurveSurface_Inter`. Hover and selection highlighting. Touch gesture mapping. Low-fidelity drag mode (bounding box during camera move).

**Exit:** Click face → highlights. All camera modes work with mouse and touch. Bounding box shown during drag, full mesh on release.

---

### Phase 3 — Basic Solid Modeling
Implement per the Phase 1 history spec:
- Extrude, revolve, boolean union/cut/common, fillet, chamfer
- Full dirty-flag partial rebuild system
- Functional pipeline hash caching
- History panel UI: scrollable list, click to inspect params, broken indicator
- Synchronous undo/redo
- Operation preview / ghost mode (pre-commit preview)
- MakerLog logging system

**Exit:** 10-operation model (sketch → extrude → fillet → boolean cut chain) builds in < 200ms. History panel shows all ops. Undo/redo works. Editing op [3] in a 7-op model rebuilds only ops [3]–[7]. Ghost preview shown before commit.

---

### Phase 3.5 — Stability Layer
- Mesh cache: pipeline hash → `ArrayMesh` lookup, evict unused
- Rebuild validation: every output shape checked for null, finite bounds, no degenerate faces
- Error handling: every OCCT call wrapped in try/catch; failures surface as `broken` nodes
- Failure recovery UX: "Fix" button, re-selection mode, parameter preservation
- Heuristic matcher: `SelectionIntent` → best-match entity after rebuild, confidence threshold
- Freeze history: implementation and UI
- Performance baseline: document rebuild time and frame time on HD 530 for 10/25/50-op models
- Logging: all operations, rebuilds, tessellations, cache hits/misses

**Exit:** Deliberate errors (impossible fillet, invalid boolean) handled without crash, surfaced gracefully. All performance baseline numbers documented. Freeze history reduces 50-op rebuild to < 200ms from freeze point.

---

### Phase 4 — 2D Constraint Sketcher
Plane selection for sketch origin. Drawing tools: line, arc, circle, rectangle. planegcs embedded and integrated. v1 constraint set only: coincident, horizontal, vertical, distance, angle. No constraint editing graph. Auto-constrain lite (suggest on stroke completion). Visual state feedback (green/blue/red). Conflict detection and highlight. Touch/pen drawing. Sketch → closed profile → feeds Phase 3 operations.

**Exit:** Draw a constrained profile (≥ 3 constraint types), fully constrain it to green state, extrude it. Over-constrain a sketch — conflict highlights correctly, suggestion offered.

---

### Phase 5 — File I/O
STEP export (`STEPControl_Writer`) and import (`STEPControl_Reader`). IGES export/import. STL export (`StlAPI_Writer`). Native `.makercad` format (JSON history + BRep blobs, version 1, migration handler stub). Recent files list. File format round-trip: save, close, reopen, history intact.

**Exit:** Export to STEP, open in FreeCAD — geometry matches. Reopen `.makercad` file — full history restored, all operations functional.

---

### Phase 6 — Performance, Robustness & Selection Hardening
- Background tessellation thread, double-buffered mesh swap
- Async history rebuild for 50+ op models
- Low-fidelity interaction mode (bounding box / decimated during any drag)
- Comprehensive selection testing across operation types (edge, face, vertex after all operation combinations)
- Rebuild profiling: hit all Phase 4 targets on HD 530
- Memory profiling: confirm < 600 MB at runtime for a typical 30-op model
- planegcs silent failure detection

**Exit:** All runtime performance targets met on reference hardware. No UI freeze during tessellation of a 50-op model. Selection reliable across a test suite of 20 operation-type combinations.

---

### Phase 7 — AI / MCP Integration
*(System is now stable enough for AI to work reliably)*

- GDScript HTTP client for Anthropic API
- Model state serialization: `serialize_history()` → AI context window
- Structured JSON command parsing → `HistoryEngine` calls
- godot-mcp server integration (dev tooling)
- Custom MakerCAD MCP server (tool schema from Phase 1)
- Natural language input panel in Godot UI
- AI for constraint suggestions (when sketch is over-constrained)
- AI for operation suggestions based on current geometry

**Exit:** "Make the cylinder 50mm taller" → history updates, model rebuilds. "Add a fillet to all top edges" → AI uses geometric heuristics to select edges and adds fillet.

---

### Phase 8 — UX Polish & Beginner Experience
- Beginner mode: guided workflow from first launch
- Tutorial project: a bracket or hook, completable in < 10 minutes unassisted
- Tooltip system with visual examples for every operation
- High-contrast UI theme, large touch targets (minimum 44×44px), clear iconography
- Touch gesture refinement
- Accessibility review

**Exit:** Person with no CAD experience completes the tutorial in < 10 minutes unassisted. App cold-starts in < 3 seconds on reference hardware.

---

## 11. UX Strategy

### The Opportunity
CAD UX is notoriously hostile to beginners even in billion-dollar tools. MakerCAD can win here. The UX is not a skin on top of the engineering — it is a design goal equal in importance to geometry correctness.

### Principles
1. **Words before icons.** Every tool has a visible text label. Icons are secondary.
2. **Errors that teach.** "Cannot fillet this edge — the radius is larger than the edge length. Try a smaller radius." Never "OCCT error -1."
3. **Progressive disclosure.** Show only what the user needs right now. Advanced options are hidden until the user has context to use them.
4. **Guided first use.** First launch offers a tutorial path. The tutorial produces a real object, not a demo sphere.
5. **Visual constraint state.** Sketch state is shown on the canvas with color. No separate status panel.
6. **Undo is prominent.** Large, obviously labeled. Two-finger tap anywhere in viewport = undo.
7. **Failure recovery is a first-class flow.** A broken operation is not a dead end — it has a clear "Fix" path that preserves all parameters.

### Touch & Pen UX
- Minimum 44×44px touch targets on all toolbar elements
- Long-press on history operation: shows "edit parameters" and "delete" inline (no right-click)
- Two-finger tap = undo (anywhere in viewport)
- Three-finger tap = redo
- Pen in sketch mode: draws geometry directly on workplane, auto-constrain lite suggests constraints after each stroke
- Pinch = zoom, 2-finger drag = orbit, 3-finger drag = pan

### UX References — Touch CAD Precedents
Study these before finalizing Phase 4 and Phase 8 UX:
- **Shapr3D** — the gold standard for touch CAD. Study their sketch mode, constraint UX, and object selection.
- **uMake** — conceptual modeling on iPad. Study their gesture system and approachability.
- **Nomad Sculpt** — not CAD, but has exceptional touch UX. The user base confirms this is a benchmark for how a complex 3D tool can feel natural on a touchscreen. Study their tool panel placement, gesture sensitivity, and undo behavior.

### Beginner Mode vs. Expert Mode
v1 ships in guided mode only. Expert mode (all operations exposed) is v2. The guided mode:
- Curates the visible operation set to the most useful 80%
- Provides contextual hints at each step
- Prevents access to operations that require knowledge the user hasn't built yet
- Surfaces AI suggestions proactively when the user appears stuck

### AI as UX Bridge
AI serves a specific UX function: it lets beginners describe intent in plain language and translates it into CAD operations. "I want a box with a rounded hole in it" → the AI proposes a sketch, extrude, and boolean cut with fillet. This removes the vocabulary barrier — the biggest obstacle preventing non-professionals from starting. AI is not magic; it is a translation layer between human intent and parametric operations.

---

## 12. Open Decisions

| Decision | Options | Current Recommendation | Resolve By |
|---|---|---|---|
| planegcs integration | Static lib, copy source, git submodule | Git submodule (maintainable) | Phase 4 |
| Tessellation thread model | Single background thread, thread pool | Single thread for v1 | Phase 6 |
| Native project compression | Uncompressed JSON+BRep, zip archive | Zip (smaller files, still debuggable) | Phase 5 |
| Undo stack depth | Unlimited (memory risk), fixed N, time-based | Fixed 100 steps with user preference | Phase 3 |
| AI context window size | Full history vs. summary vs. last N ops | Full history for v1 (most correct) | Phase 7 |
| Freeze history granularity | Single freeze point, multiple named checkpoints | Single freeze for v1 | Phase 3.5 |

---

## 13. Expert Review Q&A

*Answers to questions posed in v2.0, answered by the peer reviewer and incorporated here for completeness.*

**Q1: Topological naming — is the heuristic strategy adequate for v1?**
Yes, acceptable for v1. The addition of structured `SelectionIntent` (rich geometric criteria, not just position) makes it viable rather than merely hopeful. The explicit confidence threshold and fail-gracefully behavior are correct.

**Q2: planegcs known failure modes?**
Over-constrained configurations can cause silent failure (solver returns without signaling error). Oscillation possible with simultaneous conflicting distance + angle constraints. Mitigation: poll solver return state, compare vertex positions before/after, limit v1 constraint set to reduce conflict surface area. The v1 UX plan (highlight + suggest removal) is solid.

**Q3: OpenGL 3.3 Compatibility mode limitations?**
Biggest real issues for CAD: no compute shaders, unreliable `glLineWidth`, limited depth tricks. All manageable — CAD rendering does not need compute, and geometry-based line rendering solves the line problem. None of these are blockers.

**Q4: At what scale does linear history break down?**
Approximately 100–200 operations without dirty flags. With the dirty flag partial rebuild system, this limit extends significantly — in practice, a user editing operation [80] in a 120-op model rebuilds 40 ops, not 120. Freeze history provides an additional escape valve when rebuild time is still unacceptable.

**Q5: OCCT 7.9 vs. 8.x?**
Stay on 7.9. Stability, documentation, and binary availability are the priorities. 8.x offers no improvements compelling enough to justify the reduced availability and increased risk.

**Q6: IPC migration trigger?**
Switch if: frequent OCCT-caused crashes in user hands, OR operations regularly exceeding 500ms and blocking the UI despite async tessellation and rebuild mitigations.

**Q7: Tessellation threading — single thread vs. thread pool?**
Single background thread is sufficient for v1. Thread pool is premature complexity. Revisit in v2 if tessellation becomes a bottleneck on complex assemblies.

**Q8: Touch CAD UX references?**
Shapr3D, uMake, and Nomad Sculpt. Nomad Sculpt is specifically noted as a UX benchmark by the development team — its approach to making complex 3D interaction feel natural on touchscreen is directly applicable.

---

## 14. Questions for Further Review

1. **Dirty flag rebuild:** Is the proposed `HistoryNode.dirty` propagation strategy (mark changed node + all downstream) correct for all operation types, including operations with multiple inputs (boolean cut takes two shape inputs)? How should dirty propagation work when one input is clean and one is dirty?

2. **planegcs integration depth:** Should planegcs be embedded as a git submodule with its full source, or should only the solver core be extracted? What are the LGPL compliance implications of each approach?

3. **SelectionIntent heuristic quality:** Are there known operation sequences where geometric heuristics are fundamentally unreliable — cases where two topologically different edges have nearly identical geometric descriptions after a rebuild — and what is the correct behavior when the matcher cannot distinguish between candidates?

4. **Freeze history and STEP interop:** When a user imports a STEP file, the result is a single "frozen" shape with no parametric history. Is there a reasonable approach to inferring partial history from STEP geometry (face recognition, feature detection) that is tractable for v1, or should STEP imports always be frozen shapes?

5. **planegcs LGPL compliance:** The planegcs library is LGPL. What are the specific distribution obligations when embedding it statically in a GDExtension `.dll`, and does this affect the project's license strategy?

6. **Nomad Sculpt UX patterns:** Specifically, which of Nomad Sculpt's interaction patterns (tool wheel, gesture sensitivity, undo depth) are most transferable to a constraint-based CAD sketcher, and which are sculpting-specific and should be avoided?

7. **Async rebuild and UI consistency:** During an async rebuild, the viewport shows the old geometry while the new geometry is computing. What is the correct UX for this state — spinner overlay, progress bar in history panel, or optimistic display of partial results?

---

*MakerCAD Project Plan v3.0 — April 2026*
*Prepared by Claude (Anthropic). Two rounds of senior developer peer review incorporated.*
*Status: Final pre-build design review. History model spec in Section 5 is the binding engineering contract before Phase 3 begins.*
*Primary references: FreeCAD source (LGPL v2), OCCT documentation (LGPL v2.1), Godot GDExtension docs (MIT), planegcs (LGPL), Anthropic MCP specification*
