# MakerCAD Universal Geometry & Format Platform
## Comprehensive Program Plan and Reference Architecture

**Document status:** Architecture proposal / implementation foundation  
**Purpose:** Provide a detailed technical and product blueprint that another AI, engineer, or project lead can extend with direct MakerCAD repository context.

---

## 1. Executive Summary

MakerCAD should not implement a collection of unrelated file importers. It should implement a **universal geometry ingestion, representation, caching, streaming, and visualization platform**.

The central product promise is:

> **Open as much as possible, show something useful quickly, preserve meaningful source structure, and scale automatically to the hardware available.**

The low-end baseline is not an afterthought. MakerCAD is being designed around systems such as:

- Intel Core i5-6600
- Intel HD Graphics 530
- 8 GB RAM
- Windows 10
- Godot 4.6.2 Standard
- Compatibility renderer / OpenGL-class capabilities

The architecture must therefore avoid assuming abundant RAM, dedicated VRAM, modern GPU features, or large CPU headroom.

At the same time, low-end support must not create a crippled application. The same project should be able to scale upward on better hardware.

The recommended architecture is:

```text
                    SOURCE FILE
                         |
                  Format Detection
                         |
              +----------+----------+
              |                     |
        CAD / B-Rep            Polygon / Asset
              |                     |
            OCCT             Assimp / Readers
              |                     |
              +----------+----------+
                         |
                  Vector / BIM /
                  Point Cloud
                         |
                         v
              MakerCAD Intermediate
                 Representation
                         |
               Geometry Processor
        +----------------+----------------+
        |                |                |
     Validate        Tessellate        Optimize
        |                |                |
        +----------------+----------------+
                         |
                Spatial Partitioning
                         |
                    LOD Pyramid
                         |
                Persistent Cache
                         |
                Streaming Manager
                         |
             Adaptive Resource Budget
                         |
                    GDExtension
                         |
                 Godot Viewport
```

The most important architectural distinction is:

**Source representation, canonical representation, and display representation are separate things.**

A STEP B-Rep must not become "just triangles" merely because triangles are convenient to render.

---

# 2. Product Goals

## 2.1 Primary Goals

1. Open a very broad range of 3D, CAD, vector, BIM, and point-cloud formats.
2. Provide a fast, approachable viewing experience for non-CAD users.
3. Preserve source semantics whenever practical.
4. Provide a path from viewing to editing.
5. Support very large files without requiring the entire file to reside in RAM.
6. Provide progressive visual refinement.
7. Maintain a bounded working set on low-end hardware.
8. Scale automatically when better CPU/GPU/RAM resources are available.
9. Avoid unnecessary process boundaries and data copies.
10. Keep the format layer extensible so commercial SDKs or additional open-source readers can be introduced later.
11. Keep rendering and geometry processing cleanly separated.
12. Make failures graceful, diagnosable, and recoverable.

## 2.2 Non-Goals for the First Implementation

Do not attempt to make every supported format fully editable on day one.

Viewing coverage can be much broader than native editing coverage.

Example:

| Format family | View | Preserve structure | Edit |
|---|---:|---:|---:|
| STEP | Yes | Yes | Strong |
| IGES | Yes | Yes | Partial/strong |
| BREP | Yes | Yes | Strong |
| OBJ | Yes | Yes | Mesh |
| STL | Yes | Limited | Mesh |
| FBX | Yes | Yes where available | Limited |
| GLTF/GLB | Yes | Yes | Limited |
| DXF | Yes | Yes | Strong for supported entities |
| DWG | Yes, subject to adapter | Yes where available | Adapter-dependent |
| SVG | Yes | Yes | Strong |
| IFC | Yes | Yes | Selective |
| Point clouds | Yes | Yes | Specialized |

The architecture should permit increasing editing capability over time without redesigning the viewer.

---

# 3. Core Architectural Principles

## Principle 1 — Low-End First

The baseline machine is a first-class target.

Do not:

> build for a powerful workstation and optimize later.

Instead:

> establish a bounded resource model first, then allow higher-end hardware to increase budgets.

## Principle 2 — Never Load What You Do Not Need

The application should maintain a **working set**, not a full in-memory copy of the source.

## Principle 3 — Progressive Results Beat a Long Loading Screen

The user should receive:

1. file recognition,
2. metadata,
3. hierarchy,
4. bounds,
5. coarse preview,
6. usable navigation,
7. progressive refinement.

The model does not have to be perfect before it becomes useful.

## Principle 4 — Preserve Meaning

A B-Rep face, a CAD edge, an SVG Bézier path, and a mesh triangle are not interchangeable concepts.

## Principle 5 — Rendering Is a Consumer

Godot should consume a normalized representation rather than becoming responsible for file-format knowledge.

## Principle 6 — Cache Expensive Work

A conversion performed once should not necessarily be performed again.

## Principle 7 — Optimize for the User's Current Intent

If the user is looking at one small area, resources should concentrate there.

## Principle 8 — No Arbitrary Global Triangle Ceiling

A universal 50,000-triangle limit is too crude.

Budgets should consider:

- visible triangles
- draw calls
- objects
- materials
- memory
- camera distance
- screen-space error
- selection state
- hardware capability

## Principle 9 — Graceful Degradation

When resources become constrained:

1. reduce distant LOD,
2. unload invisible geometry,
3. reduce texture quality,
4. reduce shadow quality,
5. reduce effects,
6. preserve interaction and selection.

The application should not abruptly fail because a model is large.

---

# 4. The Universal Ingestion Pipeline

## 4.1 High-Level Pipeline

```text
User selects file
      |
      v
Format detection
      |
      v
Adapter selection
      |
      v
Source inspection
      |
      +---- metadata
      +---- units
      +---- hierarchy
      +---- bounds
      +---- object count
      +---- geometry type
      |
      v
Canonicalization
      |
      v
Validation / repair
      |
      v
Geometry processing
      |
      +---- tessellation where necessary
      +---- mesh optimization
      +---- LOD generation
      +---- spatial partitioning
      |
      v
Cache creation / lookup
      |
      v
Streaming
      |
      v
Godot display
```

## 4.2 Import Must Be Staged

The importer should not behave as one monolithic operation.

Recommended stages:

```text
DISCOVER
  -> IDENTIFY
  -> INSPECT
  -> STRUCTURE
  -> PREVIEW
  -> PROCESS
  -> CACHE
  -> REFINE
```

Each stage should expose progress and cancellation.

---

# 5. Format Adapter Architecture

Use a plugin/adapter boundary.

Conceptual interface:

```cpp
class IFormatAdapter {
public:
    virtual bool can_open(const FileInfo& file) = 0;

    virtual DocumentInspection inspect(
        const std::filesystem::path& path) = 0;

    virtual ImportSession begin_import(
        const ImportOptions& options) = 0;

    virtual bool read_metadata(ImportSession&) = 0;
    virtual bool read_hierarchy(ImportSession&) = 0;
    virtual bool read_geometry(ImportSession&, GeometrySink&) = 0;
    virtual bool read_materials(ImportSession&, MaterialSink&) = 0;
    virtual bool read_attributes(ImportSession&, AttributeSink&) = 0;

    virtual void cancel(ImportSession&) = 0;
};
```

The exact API should be refined after repository inspection.

The critical architectural property is that **Godot never needs to know which adapter produced the data.**

---

# 6. Recommended Format Families

## 6.1 CAD / B-Rep

Primary technology:

**Open CASCADE Technology (OCCT)**

Use for:

- STEP
- IGES
- BREP
- supported CAD geometry

The canonical object should remain an OCCT `TopoDS_Shape` or an appropriate higher-level structure.

Pipeline:

```text
STEP
  |
  v
OCCT
  |
  v
TopoDS_Shape
  |
  +--> topology
  +--> faces
  +--> edges
  +--> solids
  +--> assemblies
  +--> metadata
  |
  v
adaptive tessellation
  |
  v
display LODs
```

The mesh is a rendering artifact.

## 6.2 Polygon / Asset Formats

Potential foundation:

- Assimp
- specialized readers where Assimp is insufficient

Potential formats include:

- OBJ
- FBX
- GLTF
- GLB
- STL
- PLY
- 3MF
- Collada
- other asset formats supported by the selected adapters

Assimp should be treated as an **asset ingestion component**, not the universal MakerCAD data model.

## 6.3 Vector Formats

Treat vector information as native vector data whenever possible.

Examples:

- SVG
- DXF
- DWG
- HPGL/PLT
- PDF-derived vector content where appropriate

Do not immediately rasterize or triangulate everything.

Conceptual representation:

```text
VectorDocument
  |
  +-- Layers
  +-- Paths
  +-- Lines
  +-- Arcs
  +-- Circles
  +-- Splines
  +-- Text
  +-- Dimensions
  +-- Attributes
```

This enables future conversion to sketches and editable geometry.

## 6.4 BIM

IFC should retain:

- object identity
- hierarchy
- semantic type
- properties
- spatial relationships
- geometry

Do not reduce IFC to a visually correct triangle scene and discard its semantic layer.

## 6.5 Point Clouds

Point clouds should remain point-cloud data.

Potential families:

- LAS
- LAZ
- E57
- PTS
- PTX
- XYZ
- other appropriate formats

Use spatial indexing, density management, and LOD rather than converting the entire dataset to triangles.

---

# 7. MakerCAD Intermediate Representation (MIR)

The MIR is the most important abstraction in the system.

It should sit between importers and the renderer.

Conceptual structure:

```text
MakerDocument
|
+-- document_id
+-- source_info
+-- units
+-- coordinate_system
+-- metadata
|
+-- nodes
|    |
|    +-- Node
|         +-- name
|         +-- source_id
|         +-- parent
|         +-- transform
|         +-- visibility
|         +-- bounds
|         +-- geometry_ref
|         +-- material_ref
|         +-- attributes
|
+-- geometry
|    +-- BRep references
|    +-- mesh references
|    +-- vector references
|    +-- point-cloud references
|
+-- materials
+-- layers
+-- assemblies
+-- properties
```

The MIR should support stable source IDs so selections can map back to source entities.

For example:

```text
Godot selection
      |
      v
MIR entity ID
      |
      v
source adapter entity
      |
      v
original CAD/vector/mesh object
```

This becomes critical for future editing.

---

# 8. Source / Canonical / Display Separation

Use three explicit conceptual layers.

```text
SOURCE
  |
  | immutable source
  v
CANONICAL
  |
  | accurate application representation
  v
DISPLAY
  |
  | optimized approximation
  v
GODOT
```

Examples:

### STEP

```text
STEP file
  -> OCCT B-Rep
  -> canonical CAD representation
  -> tessellated LOD meshes
  -> Godot
```

### OBJ

```text
OBJ
  -> canonical indexed mesh
  -> optimized mesh
  -> LODs
  -> Godot
```

### SVG

```text
SVG
  -> canonical paths
  -> screen/render representation
  -> optional tessellation
```

The display layer must never be mistaken for the source of truth.

---

# 9. True Out-of-Core Architecture

The phrase "out-of-core" should have a strict definition in MakerCAD.

## Incorrect

```text
Huge file
   |
   v
Load entire file into RAM
   |
   v
Simplify
   |
   v
Display
```

## Correct

```text
Huge file
   |
   v
Bounded reader
   |
   v
Disk-backed intermediate/cache
   |
   v
Spatial partition
   |
   v
Camera query
   |
   v
Only needed chunks loaded
   |
   v
Godot
```

The system should establish explicit memory budgets.

Example conceptual budget:

```text
Total RAM
  |
  +-- OS / other applications
  |
  +-- MakerCAD process budget
       |
       +-- UI
       +-- MIR
       +-- worker buffers
       +-- geometry cache buffers
       +-- active render geometry
       +-- safety margin
```

Do not hard-code these values until benchmarked.

---

# 10. Persistent Geometry Cache

Create a MakerCAD cache format.

Conceptual layout:

```text
Model.mcache/
|
+-- manifest
+-- source_info
+-- metadata
+-- hierarchy
+-- materials
+-- spatial_index
+-- geometry/
|    +-- lod0/
|    +-- lod1/
|    +-- lod2/
|    +-- lod3/
|
+-- thumbnails
+-- diagnostics
```

Manifest should include:

- source path or source identity
- source file size
- source modification timestamp
- content hash where appropriate
- importer version
- adapter version
- cache schema version
- tessellation settings
- optimization settings
- coordinate system
- unit system

Cache invalidation must be deterministic.

## Cache goals

First open:

```text
parse -> process -> cache -> display
```

Subsequent open:

```text
validate cache -> display -> refine
```

---

# 11. Spatial Partitioning

Large models should be divided into spatial chunks.

Possible structures:

- BVH
- octree
- spatial grid
- hierarchical bounding volumes

The implementation should be benchmarked against MakerCAD's actual workloads.

Each chunk should contain metadata such as:

```text
Chunk
  +-- ID
  +-- bounds
  +-- object IDs
  +-- triangle count
  +-- LOD levels
  +-- material references
  +-- disk offset
  +-- compressed size
  +-- priority
```

The renderer should be able to ask:

> Which chunks are relevant to this camera?

---

# 12. Camera-Driven Streaming

The streaming manager should prioritize data based on:

- frustum visibility
- distance
- screen-space projected size
- current LOD
- camera velocity
- selection
- active measurement
- user interaction
- object importance
- memory pressure

A useful priority concept:

```text
Priority =
    visibility_score
  + screen_size_score
  + selection_score
  + interaction_score
  + preload_score
  - distance_penalty
  - memory_cost_penalty
```

Exact scoring should be tuned empirically.

---

# 13. Progressive LOD

Recommended conceptual progression:

```text
LOD0 = bounds / proxy
LOD1 = very coarse
LOD2 = coarse
LOD3 = medium
LOD4 = high
LOD5 = source-quality display
```

Not every object needs every level.

LOD should be generated according to geometry complexity.

The key principle:

> **Use screen-space error rather than a universal triangle target whenever practical.**

A tiny distant object can be represented with very little geometry.

A close curved fillet may require significantly more.

---

# 14. Adaptive Tessellation for OCCT

For B-Rep geometry, tessellation should respond to:

- surface curvature
- edge length
- camera distance
- projected screen size
- requested accuracy
- selection/measurement state
- hardware budget

Example:

```text
Large flat face
    -> low tessellation

Small curved fillet
    -> higher tessellation

Far-away object
    -> low tessellation

Selected object
    -> temporarily increase fidelity

Measurement operation
    -> request precision from canonical B-Rep
```

This avoids wasting triangles where they have little visual value.

---

# 15. Mesh Processing Pipeline

Do not reduce the process to "decimate 70-90%."

Use staged processing:

```text
Import
  |
  v
Validate
  |
  v
Repair where safe
  |
  v
Normalize
  |
  v
Deduplicate
  |
  v
Index
  |
  v
Cache optimization
  |
  v
Overdraw optimization
  |
  v
Simplification
  |
  v
Vertex-fetch optimization
  |
  v
LOD generation
```

Meshoptimizer is appropriate for GPU-oriented optimization and topology-aware simplification.

Optimization must preserve important attributes and boundaries.

---

# 16. Adaptive Resource Budget

MakerCAD should detect or estimate hardware capability.

Potential inputs:

- CPU family / logical cores
- system RAM
- graphics adapter
- shared vs dedicated graphics memory
- renderer capabilities
- available disk space
- current application memory pressure
- benchmarked performance

Then establish budgets.

```text
Hardware
   |
   v
Capability Profile
   |
   +-- RAM budget
   +-- active geometry budget
   +-- draw-call budget
   +-- texture budget
   +-- shadow budget
   +-- cache budget
   +-- streaming concurrency
   |
   v
Runtime Resource Manager
```

The user should normally see:

> Performance: Automatic

rather than a complicated technical configuration panel.

Advanced users can override settings.

---

# 17. Low-End Performance Strategy

For the Intel HD 530 baseline:

Prioritize:

1. low draw-call count
2. bounded geometry memory
3. simple materials
4. limited texture memory
5. low shadow cost
6. aggressive distant LOD
7. minimal overdraw
8. predictable frame time
9. background processing
10. avoiding unnecessary data copies

Do not assume a blanket "disable texture compression" or "disable all shadows" policy.

Those choices should be benchmark-driven.

---

# 18. Godot Boundary

Godot should be primarily responsible for:

- viewport
- camera
- navigation
- selection presentation
- gizmos
- overlays
- UI
- visual feedback
- scene presentation

C++ should own:

- file parsing
- OCCT
- asset import
- vector import
- MIR
- validation
- tessellation
- optimization
- LOD generation
- spatial index
- cache
- streaming decisions
- worker scheduling

This boundary should be documented and defended.

---

# 19. GDExtension Worker Architecture

Conceptual C++ structure:

```text
MakerCADExtension
|
+-- ImportManager
|
+-- AdapterRegistry
|
+-- WorkerPool
|
+-- GeometryProcessor
|
+-- TessellationService
|
+-- MeshOptimizer
|
+-- LODGenerator
|
+-- SpatialIndex
|
+-- CacheManager
|
+-- StreamingManager
|
+-- ResourceBudget
|
+-- Diagnostics
```

Worker jobs should use a queue.

Example:

```cpp
struct GeometryJob {
    JobId id;
    DocumentId document;
    ChunkId chunk;
    LodLevel lod;
    Priority priority;
};
```

Workers process jobs independently.

Godot receives completed results through a controlled synchronization boundary.

---

# 20. Threading Rules

Worker threads may perform:

- parsing
- geometry calculations
- OCCT operations where the relevant operation is safe in the chosen usage model
- mesh processing
- serialization
- cache I/O
- spatial index construction

Godot resource and scene operations must obey the engine's thread-safety rules.

Use a pattern like:

```text
Worker
   |
   v
CPU-side result
   |
   v
thread-safe result queue
   |
   v
Godot/main-thread commit
   |
   v
render resource
```

Never solve thread safety by simply moving everything back to the main thread.

---

# 21. Example Job System

Conceptual API:

```cpp
using JobId = uint64_t;

enum class JobType {
    Inspect,
    Import,
    Tessellate,
    Optimize,
    BuildLOD,
    BuildSpatialIndex,
    LoadChunk,
    UnloadChunk,
    BuildCache
};

struct JobRequest {
    JobId id;
    JobType type;
    int priority;
    DocumentId document;
    ChunkId chunk;
    int lod;
};

class JobSystem {
public:
    JobId submit(JobRequest request);
    void cancel(JobId id);
    void cancel_document(DocumentId document);
    JobStatus status(JobId id);
};
```

The exact implementation can later use a lock-free or conventional concurrent queue depending on profiling.

---

# 22. Example Streaming State Machine

Each geometry chunk can have a state:

```text
UNSEEN
  |
REQUESTED
  |
LOADING
  |
CPU_READY
  |
UPLOAD_PENDING
  |
GPU_RESIDENT
  |
VISIBLE
  |
DEPRIORITIZED
  |
EVICTION_PENDING
  |
EVICTED
```

Error path:

```text
LOADING
   |
   v
FAILED
   |
   +--> retry
   +--> fallback LOD
   +--> diagnostic
```

This gives the system explicit lifecycle behavior.

---

# 23. Memory Management

Memory must be treated as a first-class resource.

Track:

```text
CPU RAM
  |
  +-- source buffers
  +-- MIR
  +-- worker buffers
  +-- cache buffers
  +-- active geometry
  +-- pending jobs

GPU/shared graphics memory
  |
  +-- mesh resources
  +-- textures
  +-- frame buffers
```

A resource manager should know approximately how much memory every resident chunk consumes.

When pressure rises:

```text
HIGH PRESSURE
    |
    +-- reduce distant LOD
    +-- unload invisible chunks
    +-- cancel low-priority prefetch
    +-- reduce texture residency
    +-- preserve selected/nearby geometry
```

---

# 24. Disk Cache Is Part of the Runtime

The cache should not merely be an import artifact.

It can become a persistent backing store for streaming.

That means:

```text
Disk
 |
 +-- source
 |
 +-- canonical/cache data
 |
 +-- LOD chunks
 |
 +-- spatial index
```

RAM becomes a working set.

This is the fundamental mechanism that allows huge projects to remain usable on small machines.

---

# 25. Binary Data Format

Avoid JSON for large geometry.

Use compact binary records.

Conceptual chunk:

```text
ChunkHeader
{
    magic;
    version;
    chunk_id;
    lod;
    vertex_count;
    index_count;
    material_count;
    bounds;
    compressed_size;
    uncompressed_size;
}
```

Then:

```text
Header
Vertices
Indices
Normals
UVs
Attributes
Material references
```

Use indexed geometry rather than duplicating vertices whenever practical.

The actual binary format should be versioned from day one.

---

# 26. Zero-Copy / Low-Copy Strategy

Avoid:

```text
disk
 -> parser buffer
 -> intermediate copy
 -> serialized buffer
 -> TCP buffer
 -> Godot buffer
 -> GPU buffer
```

Prefer:

```text
disk/cache
    |
    v
bounded CPU buffer
    |
    v
Godot upload
    |
    v
GPU/shared memory
```

Where feasible, memory mapping or carefully managed shared buffers can reduce copies.

Do not implement sophisticated zero-copy mechanisms prematurely; first measure where copies actually cost time or memory.

---

# 27. Python's Role

Python should not be the core runtime dependency.

It remains useful for:

- importer experiments
- test scripts
- format conversion
- automated regression testing
- benchmark orchestration
- developer utilities
- generating sample datasets

The production architecture should remain:

**C++ + GDExtension + OCCT + native processing + Godot.**

---

# 28. IPC

Avoid TCP/UDP between the main Godot application and the geometry processor in the first architecture.

Preferred:

```text
Godot
  |
  v
C++ GDExtension
  |
  v
Worker Pool
```

If a separate process later becomes necessary for crash isolation, sandboxing, or proprietary SDK requirements, define a versioned binary IPC protocol.

That should be an extension point rather than a foundational requirement.

---

# 29. Error Handling

Import errors must be categorized.

Example:

```text
UNSUPPORTED_FORMAT
CORRUPT_FILE
PARTIAL_FILE
UNSUPPORTED_FEATURE
GEOMETRY_ERROR
TESSELLATION_ERROR
OUT_OF_MEMORY
CACHE_ERROR
PERMISSION_ERROR
CANCELLED
UNKNOWN_ERROR
```

The user-facing message should be human-readable.

The diagnostic layer should contain technical detail.

Example:

User:

> "MakerCAD couldn't display one portion of this model. The rest of the model is available."

Developer diagnostic:

```text
Import warning:
adapter=STEP/OCCT
entity=FACE
source_id=84291
operation=BRepMesh
error=...
fallback=coarse_proxy
```

Never expose raw exception dumps as the primary UX.

---

# 30. Partial Success Is a Feature

A large model may contain a few malformed objects.

Do not make the entire document fail if the majority can be shown.

Use:

```text
Document
  |
  +-- 98.7% successfully imported
  |
  +-- 1.3% fallback/proxy/failed
```

Give the user a diagnostic panel.

Potential UI:

> **Model opened with 4 warnings**

[View warnings]

This is much better than:

> Import failed.

---

# 31. Cancellation

Every long-running operation should be cancellable.

For example:

```text
Opening Factory.step

██████████████░░░░ 78%

[Cancel]
```

Cancellation should propagate:

```text
UI
 |
 v
ImportManager
 |
 v
JobSystem
 |
 v
Adapter
 |
 v
Geometry processing
```

Workers must check cancellation points.

---

# 32. User Experience: "Useful Immediately"

A user should not have to understand:

- tessellation
- LOD
- RAM budgets
- cache invalidation
- meshoptimizer
- B-Reps

The default experience should be:

```text
Open File
     |
     v
MakerCAD recognizes it
     |
     v
"Preparing model..."
     |
     v
Model appears
     |
     v
MakerCAD quietly improves it
```

This is a core part of lowering the barrier to CAD.

---

# 33. Universal Viewer UX

A simple opening experience:

```text
+---------------------------------------------------------+
| Open Model                                               |
+---------------------------------------------------------+
|                                                         |
|  Drop a 3D, CAD, vector, or BIM file here               |
|                                                         |
|                    [ Browse ]                           |
|                                                         |
|  Supported content                                      |
|  3D • CAD • Vector • BIM • Point Cloud                  |
|                                                         |
+---------------------------------------------------------+
```

After opening:

```text
+---------------------------------------------------------+
| Factory.step                                  78%       |
+---------------------------------------------------------+
|                                                         |
|                    3D VIEWPORT                          |
|                                                         |
|                                                         |
|                                                         |
+---------------------------------------------------------+
| Preparing detailed geometry...                          |
+---------------------------------------------------------+
```

The user can interact while refinement continues.

---

# 34. Format Inspection Before Full Import

The system should quickly answer:

- What is this file?
- What units does it use?
- How many objects?
- What is its bounding box?
- What geometry types are present?
- Are there layers?
- Is it an assembly?
- Is it likely to be large?
- Is a cache available?

This can be done before full geometry processing where the format permits.

---

# 35. Automatic Performance Profiles

Suggested internal profiles:

```text
ULTRA_LOW
LOW
BALANCED
HIGH
ULTRA
CUSTOM
```

But expose primarily:

```text
Performance
[ Automatic ]
```

Advanced settings can reveal:

- Geometry quality
- Texture quality
- Shadows
- Antialiasing
- Streaming cache
- memory target
- maximum background workers

The defaults should be hardware-aware.

---

# 36. Performance Profiles Should Not Change Model Meaning

A low-end profile should change:

- display resolution
- LOD
- effects
- texture residency
- cache working set

It should not change:

- dimensions
- source units
- topology meaning
- object identity
- CAD precision

A low-performance machine must still be able to measure the same canonical geometry.

---

# 37. Precision-on-Demand

The application can display a low-resolution representation while using the canonical model for precise operations.

Example:

```text
Viewport
  |
  | user clicks edge
  v
MIR entity ID
  |
  v
Canonical geometry
  |
  v
exact edge/face/point
```

This is superior to relying on a decimated display mesh for CAD measurement.

---

# 38. Selection Architecture

Selection should be semantic rather than purely visual.

Potential selection layers:

```text
Document
  -> Assembly
      -> Component
          -> Body
              -> Face
              -> Edge
              -> Vertex
```

For mesh data:

```text
Object
  -> Submesh
      -> Face/triangle mapping where supported
```

For vector:

```text
Layer
  -> Path
      -> Segment
```

This lays groundwork for future editing.

---

# 39. Measurement

Measurements should prefer canonical data.

Examples:

- point-to-point
- edge length
- radius
- diameter
- angle
- face area
- bounding dimensions
- distance between faces

The display mesh can provide approximate visual picking, but the final measurement should resolve against the accurate representation when possible.

---

# 40. Vector-to-CAD Bridge

One major future advantage:

```text
DXF/SVG
   |
   v
Vector Representation
   |
   v
Sketch Conversion
   |
   v
MakerCAD Sketch
```

This could become a major differentiator.

A user can import a logo, floor plan, DXF, or SVG and convert useful paths into sketch geometry rather than manually redrawing them.

---

# 41. CAD-to-Mesh Bridge

The reverse should also be possible:

```text
BRep
  |
  v
Controlled Tessellation
  |
  v
Mesh
  |
  v
STL / OBJ / GLB / etc.
```

Export quality should be governed by explicit user intent:

- Draft
- Standard
- High
- Custom tolerance

---

# 42. Cache and Export Are Separate

Do not confuse:

**internal cache representation**

with:

**user export format**

The cache is optimized for MakerCAD's own runtime.

The user-facing export should remain standards-based.

---

# 43. Security / Robustness

Because MakerCAD will eventually accept arbitrary files from users, treat importers as untrusted input boundaries.

Important safeguards:

- bounds checking
- allocation limits
- recursion limits
- malformed file handling
- path validation
- cache path isolation
- cancellation
- timeouts where appropriate
- robust parser testing
- fuzz testing

If a proprietary or external SDK is introduced later, it should be isolated behind an adapter.

---

# 44. Benchmark Program

Build a permanent benchmark corpus.

Categories:

### CAD

- tiny STEP
- medium STEP
- large STEP
- assembly STEP
- complex curved STEP

### Mesh

- OBJ
- STL
- FBX
- GLB
- high-poly scan

### Vector

- small SVG
- large SVG
- simple DXF
- complex DXF

### BIM

- small IFC
- medium IFC
- large IFC

### Point Cloud

- small
- medium
- very large

Each benchmark should record:

```text
file size
entity count
triangle count
object count
materials
import time
first-visible time
usable-navigation time
peak RAM
cache size
FPS
1% low FPS
frame time
draw calls
stream latency
LOD transitions
```

---

# 45. Critical KPI: Time to Useful Model

The most important user-centric performance metric should not be "time until all geometry is finished."

Measure:

```text
T0 = file selection
T1 = format recognized
T2 = metadata visible
T3 = first geometry visible
T4 = camera/navigation usable
T5 = primary model area usable
T6 = detailed refinement complete
```

A model that takes 60 seconds to fully refine but becomes usable in 1 second may feel much better than a model that takes 10 seconds before showing anything.

---

# 46. Performance Targets

Do not permanently hard-code targets before benchmark data exists.

However, establish initial goals to validate.

For the low-end baseline:

```text
Cold application start       < 3 sec target
First useful geometry        < 1-2 sec target where feasible
Viewport                     30 FPS target at 1080p
Memory                       bounded, target < 600 MB for ordinary runtime
```

These targets should be treated as engineering goals, not promises until measured.

Large-file import targets should be expressed relative to file complexity rather than raw file size.

---

# 47. Godot Rendering Strategy

For Compatibility renderer:

Prefer:

- simple materials
- minimal shader complexity
- indexed meshes
- reasonable batching
- limited material variants
- conservative shadow use
- efficient culling
- minimal transparent surfaces
- low overdraw

Avoid designing the viewer around features that require Vulkan-class hardware.

The architecture should be able to add a more advanced renderer later without replacing the importer/streaming architecture.

---

# 48. Rendering Abstraction

Consider a renderer-facing interface:

```text
GeometryBackend
  |
  +-- upload_chunk()
  +-- remove_chunk()
  +-- update_lod()
  +-- set_visibility()
  +-- update_material()
  +-- release_resources()
```

Godot is one implementation.

This keeps the geometry system independent of the exact rendering technology.

---

# 49. Future High-End Scaling

On a stronger computer, the same pipeline can increase:

- active chunk count
- LOD quality
- texture resolution
- shadows
- worker concurrency
- cache size
- preload radius
- geometry precision
- background processing

No separate "high-end architecture" is required.

---

# 50. Resource Manager Example

Conceptual pseudocode:

```cpp
struct ResourceBudget {
    uint64_t cpu_memory_limit;
    uint64_t gpu_memory_limit;

    uint32_t max_active_chunks;
    uint32_t max_draw_calls;
    uint32_t max_worker_jobs;

    float target_frame_ms;
};

class ResourceManager {
public:
    void update(const HardwareProfile& hardware);
    void report_chunk_resident(ChunkId id, MemoryCost cost);
    void report_chunk_evicted(ChunkId id);

    bool can_resident(const ChunkCost& cost) const;

    void handle_pressure();
};
```

The actual implementation should be data-driven.

---

# 51. Streaming Manager Example

Conceptual pseudocode:

```cpp
void StreamingManager::update(const CameraState& camera)
{
    auto candidates = spatial_index.query(camera.frustum);

    for (auto& candidate : candidates) {
        float distance = candidate.distance(camera.position);
        float projected_size = candidate.projected_size(camera);

        LodLevel desired =
            choose_lod(projected_size, resource_budget);

        int priority =
            score(candidate, distance, projected_size);

        request_chunk(candidate.id, desired, priority);
    }

    evict_if_over_budget();
}
```

This is illustrative rather than production-ready.

---

# 52. Import Manager Example

```cpp
ImportResult ImportManager::open(const std::filesystem::path& path)
{
    auto adapter = registry.find_adapter(path);

    if (!adapter) {
        return ImportResult::unsupported();
    }

    auto inspection = adapter->inspect(path);

    Document document;
    document.source = path;
    document.metadata = inspection.metadata;

    schedule_structure_import(document, adapter);
    schedule_preview_generation(document, adapter);

    return ImportResult::started(document.id);
}
```

The key is that `open()` should not block until every triangle is ready.

---

# 53. Example User Flow

### User drops `Machine.step`

MakerCAD:

```text
0.0s
File recognized: STEP

0.1s
Units: millimeters

0.2s
Assembly structure detected

0.5s
Bounding structure ready

0.8s
Preview geometry available

1.0s
User can orbit/zoom/select

1.4s
Visible region refined

2.5s
Additional regions streamed

Background:
cache generation continues
```

The user does not need to know which worker did what.

---

# 54. Example Large-Model Flow

Suppose a model contains 20 million display triangles.

Do not do:

```text
20M -> RAM -> simplify -> GPU
```

Instead:

```text
20M source geometry
        |
        v
spatial chunks
        |
        +-- chunk 001
        +-- chunk 002
        +-- ...
        +-- chunk N
        |
        v
LOD hierarchy
        |
        v
camera query
        |
        v
maybe 100k-500k active triangles
        |
        v
GPU
```

The actual active budget should come from benchmarks.

---

# 55. Example Program Structure

A possible repository layout:

```text
MakerCAD/
|
+-- godot/
|   +-- project.godot
|   +-- scenes/
|   +-- scripts/
|   +-- ui/
|   +-- viewport/
|
+-- native/
|   +-- extension/
|   +-- core/
|   +-- import/
|   |   +-- adapters/
|   |   +-- occt/
|   |   +-- assimp/
|   |   +-- vector/
|   |   +-- bim/
|   |   +-- pointcloud/
|   |
|   +-- mir/
|   +-- geometry/
|   +-- tessellation/
|   +-- optimization/
|   +-- lod/
|   +-- spatial/
|   +-- cache/
|   +-- streaming/
|   +-- jobs/
|   +-- memory/
|   +-- diagnostics/
|
+-- tests/
|   +-- import/
|   +-- geometry/
|   +-- cache/
|   +-- streaming/
|   +-- performance/
|   +-- fuzz/
|
+-- benchmarks/
|   +-- corpus/
|   +-- results/
|
+-- tools/
|   +-- Python/
|   +-- converters/
|   +-- diagnostics/
|
+-- docs/
```

This is an architectural example, not a mandate for the existing repository.

---

# 56. Suggested Core C++ Modules

```text
mc::core
mc::formats
mc::mir
mc::geometry
mc::occt
mc::mesh
mc::vector
mc::bim
mc::pointcloud
mc::spatial
mc::lod
mc::cache
mc::streaming
mc::jobs
mc::memory
mc::hardware
mc::diagnostics
mc::godot
```

Use clear ownership boundaries.

---

# 57. API Boundary Example

```cpp
namespace mc {

class DocumentService {
public:
    DocumentId open(const std::filesystem::path& path);
    void close(DocumentId id);
    void cancel(DocumentId id);
};

class GeometryService {
public:
    void request_lod(
        DocumentId document,
        ChunkId chunk,
        LodLevel lod,
        int priority);
};

class CacheService {
public:
    bool has_valid_cache(DocumentId document);
    void invalidate(DocumentId document);
};

class StreamingService {
public:
    void update(const CameraState& camera);
};

}
```

The eventual implementation can evolve without changing the conceptual responsibilities.

---

# 58. Diagnostics Architecture

Every major subsystem should emit structured diagnostic events.

Example:

```text
ImportStarted
FormatDetected
MetadataRead
HierarchyRead
PreviewReady
ChunkRequested
ChunkLoaded
ChunkEvicted
LODChanged
CacheCreated
CacheInvalidated
ImportWarning
ImportError
```

These events can feed:

- developer logging
- performance profiling
- user progress
- future telemetry if ever desired
- automated tests

Do not require telemetry to operate.

---

# 59. Testing Strategy

## Unit Tests

Test:

- format detection
- cache hashing
- MIR serialization
- chunk indexing
- LOD selection
- budget calculations
- priority scoring

## Integration Tests

Test:

```text
file
 -> adapter
 -> MIR
 -> geometry
 -> cache
 -> streaming
 -> Godot
```

## Regression Tests

Every fixed import failure should become a permanent fixture.

## Stress Tests

Test:

- huge file
- thousands of objects
- many materials
- malformed geometry
- low disk space
- limited RAM
- rapid camera movement
- repeated LOD transitions

## Cancellation Tests

Cancel during:

- parsing
- tessellation
- simplification
- cache creation
- chunk loading

The application should recover cleanly.

---

# 60. Fuzzing

Because file importers consume arbitrary external data, fuzz:

- headers
- lengths
- nesting
- malformed topology
- corrupt indices
- invalid floating-point values
- enormous declared counts
- truncated files

The goal is:

> A malformed file should fail safely, not crash MakerCAD.

---

# 61. Security Boundary for External SDKs

If proprietary or third-party SDKs are introduced:

```text
MakerCAD
   |
   v
Adapter Interface
   |
   v
External SDK
```

Never spread SDK-specific types throughout the entire application.

Translate them into MIR.

This protects the rest of the application from future library changes.

---

# 62. Licensing Review

Before committing to a library, record:

- license
- commercial-use requirements
- static/dynamic linking implications
- attribution requirements
- source obligations
- patent concerns where applicable
- supported versions
- maintenance activity
- platform support

Create a dependency ledger.

Example:

```text
Dependency
Purpose
Version
License
Linking model
Commercial constraints
Replacement difficulty
Owner/module
```

This should become a release requirement.

---

# 63. Commercial DWG/DGN Strategy

DWG/DGN interoperability should remain an adapter boundary.

Potential routes:

1. open-source implementation where licensing and feature coverage are acceptable
2. commercial SDK
3. ODA-based integration
4. user-provided conversion workflow for unsupported formats

The architecture should not depend on any single route.

The important design requirement is:

```text
DWGAdapter : IFormatAdapter
```

rather than DWG assumptions embedded throughout MakerCAD.

---

# 64. Why This Architecture Supports Non-CAD Users

The technical architecture directly enables UX improvements.

### Non-CAD user problem:
"Why is this file taking forever?"

Solution:

Progressive preview.

### Problem:
"My computer can't handle CAD."

Solution:

Adaptive resource budgets.

### Problem:
"I don't know what this file type is."

Solution:

Automatic format detection.

### Problem:
"I don't understand tessellation."

Solution:

Automatic defaults.

### Problem:
"I just want to look at it."

Solution:

Viewer-first workflow.

### Problem:
"I want to measure it."

Solution:

Canonical geometry queries.

### Problem:
"I want to edit this later."

Solution:

MIR + semantic source IDs.

---

# 65. The Application Should Teach Without Getting in the Way

The viewer should not overwhelm the first-time user with CAD terminology.

Instead of:

> "B-Rep tessellation tolerance"

use:

> **Model Detail**

Instead of:

> "GPU resident geometry budget"

use:

> **Performance**

Advanced technical information can exist in diagnostics/advanced settings.

This is an important part of the product mission.

---

# 66. Recommended First Implementation Scope

Do not implement every format immediately.

### Phase A — Foundation

Build:

- Format detection
- MIR skeleton
- OCCT adapter
- Assimp adapter
- C++ worker pool
- basic cache
- Godot GDExtension bridge
- simple chunk representation
- diagnostics

### Phase B — Streaming

Add:

- spatial index
- chunk loading
- LOD
- eviction
- adaptive budgets
- camera-driven streaming

### Phase C — Vector

Add:

- SVG
- DXF
- vector MIR
- sketch conversion pathway

### Phase D — Extended Interoperability

Add:

- FBX
- GLB/GLTF
- 3MF
- IFC
- point clouds
- DWG strategy

### Phase E — Advanced

Add:

- persistent cache refinement
- precision-on-demand
- richer semantic selection
- advanced export
- additional SDK adapters

---

# 67. Recommended Phase Gates

Do not advance merely because code exists.

## Gate A

Can MakerCAD open representative STEP and mesh files without freezing the UI?

## Gate B

Can a large model display a useful preview before full processing?

## Gate C

Can the system remain inside a predefined RAM envelope?

## Gate D

Can the system unload invisible geometry and recover memory?

## Gate E

Can camera movement dynamically change LOD without severe stutter?

## Gate F

Can the cache make second-open significantly faster?

## Gate G

Can the same model use higher fidelity on a stronger system without changing project data?

---

# 68. Example Acceptance Criteria

### Import

- UI remains responsive during import.
- Cancellation works.
- Partial failure does not necessarily destroy the whole document.
- Units are detected and preserved.
- Object identity remains stable.

### Rendering

- Low-end baseline achieves the agreed frame-time target for benchmark scenes.
- Geometry is bounded.
- Distant objects use lower LOD.
- Invisible chunks can be evicted.

### Memory

- Peak working set is measurable.
- Budget violations trigger controlled degradation.
- No uncontrolled memory growth during repeated camera movement.

### Cache

- Cache is versioned.
- Invalid cache is detected.
- Source modifications invalidate appropriately.
- Corrupt cache can be rebuilt.

### UX

- First useful geometry appears quickly.
- Progress is understandable.
- Users can navigate during refinement.
- Technical failures are explained in human terms.

---

# 69. Example End-to-End Program

The following is intentionally conceptual. It illustrates how the pieces should interact rather than providing production-ready code.

```cpp
class MakerCADApplication {
    FormatRegistry formats;
    DocumentService documents;
    ImportManager importer;
    JobSystem jobs;
    CacheManager cache;
    SpatialIndex spatial;
    StreamingManager streaming;
    ResourceManager resources;
};

void MakerCADApplication::open_file(Path path)
{
    auto adapter = formats.find(path);

    if (!adapter) {
        ui.show_error("MakerCAD does not recognize this file type.");
        return;
    }

    auto inspection = adapter->inspect(path);

    auto document = documents.create(path, inspection);

    ui.show_document_info(document);

    jobs.submit({
        .type = JobType::BuildPreview,
        .document = document.id
    });

    jobs.submit({
        .type = JobType::BuildStructure,
        .document = document.id
    });

    if (cache.has_valid_cache(document)) {
        streaming.attach_cache(document);
    } else {
        jobs.submit({
            .type = JobType::BuildCache,
            .document = document.id
        });
    }
}
```

Then:

```cpp
void StreamingManager::update(CameraState camera)
{
    auto visible = spatial.query(camera);

    for (auto& item : visible) {
        auto desired_lod =
            lod_policy.choose(item, camera, resources);

        request(
            item.chunk_id,
            desired_lod,
            priority(item, camera)
        );
    }

    enforce_budget();
}
```

And:

```cpp
void ResourceManager::enforce_budget()
{
    if (!over_budget())
        return;

    evict_invisible_chunks();

    if (over_budget())
        downgrade_distant_chunks();

    if (over_budget())
        cancel_low_priority_prefetch();

    if (over_budget())
        reduce_optional_render_resources();
}
```

This is the central runtime behavior.

---

# 70. The Key Architectural Contract

The entire program can be summarized as five contracts.

## Contract 1 — Importer Contract

> Convert source formats into MakerCAD concepts without requiring Godot to understand the source format.

## Contract 2 — Representation Contract

> Preserve meaningful canonical geometry and semantics independently from display meshes.

## Contract 3 — Resource Contract

> Never allow a large model to consume unbounded memory merely because the source file is large.

## Contract 4 — Streaming Contract

> Load the highest useful quality for the user's current view and intent.

## Contract 5 — Renderer Contract

> Godot receives optimized display data and is not responsible for understanding the source file.

---

# 71. What I Would Explicitly Reject

The project should reject these architectural shortcuts unless benchmark evidence later proves them appropriate:

### Reject

```text
Load entire giant file into RAM
```

### Reject

```text
Convert all CAD to triangles permanently
```

### Reject

```text
One global 50k triangle limit
```

### Reject

```text
Python as a mandatory production runtime
```

### Reject

```text
TCP between Godot and its own native geometry worker
```

### Reject

```text
JSON for large geometry payloads
```

### Reject

```text
Godot main thread doing heavy geometry processing
```

### Reject

```text
Low-end mode as a fundamentally different project representation
```

### Reject

```text
One library being responsible for every format category
```

---

# 72. What I Would Preserve From Googie

Googie's proposal should not be discarded. Its strongest ideas form part of the foundation:

- background processing
- C++
- GDExtension
- Assimp as a polygon-format helper
- meshoptimizer
- binary geometry
- simple Compatibility rendering
- asynchronous work
- low-end hardware as a primary design concern

The major change is moving from a **single importer-to-mesh pipeline** to a **universal, semantic, streaming geometry platform**.

---

# 73. Final Recommended Architecture

```text
                           USER
                            |
                            v
                     MakerCAD UI
                            |
                            v
                   Document Controller
                            |
                            v
                   Import Coordinator
                            |
                  +---------+---------+
                  |                   |
                  v                   v
             Cache Lookup       Format Registry
                                      |
             +------------------------+----------------------+
             |                        |                      |
             v                        v                      v
        OCCT Adapter            Asset Adapter          Vector/BIM
             |                   Assimp/etc.              Adapters
             |                        |                      |
             +------------------------+----------------------+
                                      |
                                      v
                         MakerCAD Intermediate
                            Representation
                                      |
                     +----------------+----------------+
                     |                |                |
                     v                v                v
                  Geometry         Semantics         Metadata
                     |
                     v
                Geometry Jobs
                     |
       +-------------+-------------+
       |             |             |
       v             v             v
    Tessellate    Optimize       LOD
       |             |             |
       +-------------+-------------+
                     |
                     v
                Spatial Index
                     |
                     v
              Persistent Cache
                     |
                     v
              Streaming Manager
                     |
            +--------+---------+
            |                  |
            v                  v
        Camera State       Resource Budget
            |                  |
            +--------+---------+
                     |
                     v
               Chunk Requests
                     |
                     v
              Worker Pool
                     |
                     v
             CPU-side geometry
                     |
                     v
              Godot GDExtension
                     |
                     v
          Godot Compatibility Renderer
                     |
                     v
                   USER
```

---

# 74. Final Program Philosophy

MakerCAD should be built around one fundamental idea:

> **The computer should adapt to the model and the user's intent, not force the user to adapt to the computer.**

A modern workstation can keep more geometry resident.

An Intel HD 530 machine can keep less.

Neither user should need to understand why.

The user should simply open the model and work.

The system should decide:

- what must remain accurate,
- what can be approximated,
- what should be loaded,
- what should be unloaded,
- what should be cached,
- what needs higher precision,
- what the GPU can afford,
- and what the user is actually looking at.

That is the foundation for a genuinely accessible CAD platform rather than merely a CAD application with a low-end mode.

---

# 75. Recommended Next Engineering Work

Before writing large amounts of implementation code, the next project-level work should be:

1. Inspect the current MakerCAD repository and identify existing OCCT/Godot boundaries.
2. Establish the current C++/GDExtension architecture.
3. Define the first MIR schema.
4. Define the `IFormatAdapter` contract.
5. Implement STEP through OCCT first.
6. Implement OBJ/GLB or another simple mesh format through the asset adapter.
7. Build a minimal worker pool.
8. Build a minimal disk-backed cache.
9. Build one spatial chunk format.
10. Build one camera-driven streaming prototype.
11. Establish the Intel HD 530 benchmark harness.
12. Measure before selecting final memory/triangle/draw-call budgets.
13. Add progressive preview.
14. Add LOD.
15. Add eviction.
16. Add vector architecture before committing to vector-to-mesh conversion.
17. Add additional format adapters only after the architecture is stable.
18. Maintain a dependency/license ledger from the first external library integration.
19. Turn every significant bug into a regression fixture.
20. Keep the architecture capable of adding stronger hardware and additional format SDKs without redesign.

---

## Closing Recommendation

The correct next step is **not** to immediately choose between Googie’s C++ and Python implementations.

The C++ path is the stronger production architecture for MakerCAD, but the more important decision is to establish the architectural boundaries first.

The project should build a **native C++ Universal Geometry Platform** around:

**OCCT + format adapters + MakerCAD MIR + worker jobs + persistent geometry cache + spatial partitioning + adaptive LOD + camera-driven streaming + resource budgets + Godot GDExtension.**

Once those boundaries exist, individual format libraries become replaceable implementation details.

That is the architecture most likely to let MakerCAD achieve both sides of its mission:

**accessible to someone using a technically obsolete computer, while still being capable of scaling to serious modern hardware and serious engineering workloads.**

---

# 76. Architecture Revision — Authoritative Model vs Presentation Proxy

**Revision date:** 2026-08-13

This section is a major architectural clarification following review of the original plan and additional research into Open CASCADE, FreeCAD, Cesium 3D Tiles, Assimp, ODA, libigl, and geometry-kernel architectures.

The earlier document correctly separated canonical geometry from display meshes, but the distinction must now be made explicit and elevated to a foundational MakerCAD requirement:

> **The authoritative model must exist independently of the renderer. Godot displays and manipulates a resource-constrained presentation proxy of that model; it is not the source of truth.**

This is not merely an optimization technique. It is the architectural boundary that permits MakerCAD to support exact CAD geometry, mathematical vectors, semantic BIM objects, meshes, point clouds, and other representations while using a raster/triangle-oriented renderer on low-end hardware.

OCCT strongly validates this separation. Its visualization documentation explicitly describes separation between CAD data and graphical presentation, and its AIS framework separates presentation from selection. `AIS_Shape` can prepare presentations of a `TopoDS_Shape` while also supporting selection of constituent vertices, edges, wires, faces, shells, and solids. This is very close to the architecture MakerCAD needs, even though MakerCAD will implement its own Godot-facing presentation system. [Research: OCCT visualization and AIS architecture.]

The practical consequence is:

```text
                         AUTHORITATIVE MODEL
                                  |
             +--------------------+--------------------+
             |                    |                    |
             v                    v                    v
         CAD / B-Rep          Vector / Path        Mesh / Point
         exact geometry       mathematical data      source data
             |                    |                    |
             +--------------------+--------------------+
                                  |
                                  v
                       REPRESENTATION MANAGER
                                  |
               +----------------+----------------+
               |                |                |
               v                v                v
           Mesh Proxy       Raster Proxy      Line/Curve Proxy
               |                |                |
               +----------------+----------------+
                                |
                                v
                         GODOT VIEWPORT
                                |
                                v
                         USER INTERACTION
                                |
                                v
                      REPRESENTATION ID MAP
                                |
                                v
                       AUTHORITATIVE ENTITY
                                |
                                v
                         MODEL OPERATION
```

The renderer is therefore a **consumer** of geometry, not the owner of geometry.

---

# 77. The Representation Layer Is Now a First-Class Subsystem

The previous concept of `display geometry` is too narrow. The correct term is:

> **Presentation Representation**

A Presentation Representation is a computationally cheaper or renderer-compatible representation of an authoritative entity, selected for a particular view, task, hardware budget, and level of detail.

A proxy is not necessarily a triangle mesh.

It may be:

- a triangle mesh,
- a line/edge representation,
- a tessellated curve,
- a raster texture,
- a point sample,
- a bounding box,
- a simplified solid,
- an impostor,
- a semantic placeholder,
- a coarse spatial tile,
- or another renderer-specific representation.

This distinction prevents a major architectural mistake: designing the Universal Geometry Platform around triangles merely because Godot's Compatibility renderer ultimately needs GPU-renderable primitives.

The canonical representation remains mathematically meaningful even when the current presentation is aggressively simplified.

---

# 78. The Authoritative Model Must Survive Rasterization

Rasterization is not the enemy.

Rasterization is useful and often desirable in the presentation layer. The architectural failure would be to rasterize or triangulate and then discard the source representation.

Correct:

```text
SVG / vector paths
        |
        +----------------------> authoritative vector model
        |
        v
  raster/mesh proxy
        |
        v
      Godot
```

Incorrect:

```text
SVG
 |
v
rasterize
 |
v
PNG
 |
v
DELETE VECTOR INFORMATION
```

The same rule applies to CAD:

```text
STEP
  |
  v
OCCT B-Rep
  |
  +--------------------> authoritative CAD model
  |
  v
adaptive tessellation
  |
  v
Godot mesh proxy
```

The mesh is a **presentation artifact**, not the CAD truth.

FreeCAD provides a useful adjacent example: its Part objects can be tessellated for display, and its documentation describes tessellation as something needed to display geometric shapes in the 3D view. The source Part shape remains a higher-level object rather than becoming permanently replaced by the resulting mesh.

---

# 79. Multiple Proxies May Exist for One Authoritative Entity

A single model entity may have multiple representations simultaneously.

For example:

```text
                 FACE-008472
                      |
          +-----------+-----------+
          |           |           |
          v           v           v
        LOD 0       LOD 1       LOD 2
       preview      working     detailed
          |           |           |
          +-----------+-----------+
                      |
                      v
                same true face
```

A vector path could similarly have:

```text
                    PATH-00312
                        |
        +---------------+---------------+
        |               |               |
        v               v               v
     Raster          Tessellated     Outline
      proxy             mesh          proxy
```

The representation can change without changing entity identity.

This is critical for:

- LOD,
- camera movement,
- selection,
- highlighting,
- measurement,
- snapping,
- editing,
- low-end resource pressure,
- high-end refinement,
- and persistent caching.

---

# 80. Representation Identity Mapping

MakerCAD must maintain a stable mapping between presentation objects and authoritative entities.

Conceptually:

```text
Authoritative Entity ID
        |
        +-- Representation ID
        |      +-- LOD 0
        |      +-- LOD 1
        |      +-- LOD 2
        |      +-- LOD 3
        |
        +-- Source ID
        |
        +-- Semantic ID
        |
        +-- Topology ID
```

Example:

```text
Entity ID: FACE-008472

Source:
  STEP entity / source topology reference

Canonical:
  OCCT TopoDS_Face

Presentation:
  Proxy A = 32 triangles
  Proxy B = 180 triangles
  Proxy C = 1,200 triangles

Godot:
  MeshInstance / surface / face ranges
```

All representations must resolve back to the same authoritative entity.

This means the user can click a triangle without MakerCAD treating that triangle as the object being edited.

Instead:

```text
Godot triangle
    -> proxy primitive
    -> representation ID
    -> authoritative entity ID
    -> source/canonical object
```

This is the foundation for accurate selection and interaction.

---

# 81. Selection and Presentation Must Be Separate Systems

This idea is strongly reinforced by OCCT's AIS architecture. OCCT explicitly treats presentation and selection as separate mechanisms, and its shape objects can select constituent topology even when the displayed representation is graphical. This is an important precedent for MakerCAD.

MakerCAD should therefore have two related but distinct pathways:

```text
                 AUTHORITATIVE ENTITY
                         |
              +----------+----------+
              |                     |
              v                     v
        Presentation             Selection
              |                     |
              v                     v
        Godot rendering       precise entity query
```

Selection should not require the renderer to expose the entire exact model.

For example, if a coarse mesh face is clicked, the system should be able to resolve:

```text
screen coordinate
    -> Godot proxy triangle
    -> proxy ID
    -> true CAD face
```

The exact CAD face can then answer:

- area,
- surface type,
- radius,
- center,
- normal,
- topology neighbors,
- parameter-space location,
- source identifier,
- semantic object association,
- or other exact queries.

The user does not need the high-resolution display mesh for these operations.

---

# 82. Exact-on-Demand Geometry Queries

This becomes a major performance strategy.

MakerCAD should not automatically increase display tessellation simply because a user asks for accurate information.

Instead:

```text
User asks:
"What is the radius of this fillet?"

Godot proxy
    |
    v
identity mapping
    |
    v
canonical B-Rep
    |
    v
exact geometric query
    |
    v
result
```

The renderer can remain coarse while the geometry engine supplies exact information.

This should apply to:

- measurements,
- distances,
- angles,
- radii,
- areas,
- volumes,
- snapping,
- constraint inference,
- topology inspection,
- and future AI-assisted modeling.

The distinction is:

> **Visual precision and geometric precision are separate resources.**

---

# 83. Hardware Must Affect Presentation, Never Design Truth

This becomes an explicit MakerCAD engineering rule:

> **Hardware determines presentation quality and working-set size, never the authoritative model's correctness.**

On an Intel HD 530 machine:

```text
Authoritative model: unchanged
Presentation: coarse / bounded / selectively resident
```

On a modern workstation:

```text
Authoritative model: unchanged
Presentation: higher LOD / more resident / higher texture quality
```

The project file must not fork into a "low-end model" and a "high-end model."

Only the runtime representation changes.

This also means a model saved on a weak machine should open on a powerful machine with the same underlying design information available for refinement.

---

# 84. Representation Quality Should Be Dynamic

Introduce an internal concept such as:

```text
Representation Quality
    |
    +-- Preview
    +-- Draft
    +-- Working
    +-- Detailed
    +-- Exact-on-demand
```

The user should not be forced to understand these technical levels.

The Representation Manager can select them automatically based on:

- camera distance,
- projected screen size,
- object importance,
- current tool,
- selection state,
- measurement state,
- camera velocity,
- memory pressure,
- GPU/renderer capability,
- CPU availability,
- and user preference.

The system may display a coarse object while navigating and refine it when the camera stops.

---

# 85. Screen-Space Error Is a Better General Principle Than Triangle Counts

The original proposal's fixed "50,000 triangles" concept should remain rejected as a universal rule.

Cesium 3D Tiles provides a particularly relevant adjacent precedent. Its tileset hierarchy combines spatial organization with hierarchical LOD and uses geometric error and screen-space error to determine whether a representation is sufficiently detailed for the current view. It can also skip intermediate LODs to reduce loading and memory costs.

MakerCAD should adopt the same general principle, adapted to CAD rather than geospatial rendering:

> **Spend geometry where the user can perceive or interact with its effect.**

A tiny distant object may need only a handful of triangles.

A large close fillet may need many more.

A selected face may require an accurate interaction proxy even if its visual mesh is coarse.

Therefore the LOD decision should consider:

```text
projected screen error
+ object importance
+ interaction state
+ geometric complexity
+ hardware budget
+ memory pressure
```

not simply:

```text
triangle_count < 50,000
```

---

# 86. Spatial Streaming Should Borrow the Strongest Idea from 3D Tiles

Cesium's 3D Tiles architecture is an excellent adjacent example for the streaming problem. It organizes massive heterogeneous datasets into a spatial hierarchy, associates content with bounding volumes, and progressively refines the representation according to view-dependent error.

MakerCAD should not copy 3D Tiles wholesale, because CAD has different semantics and precision requirements. However, the following principles are directly useful:

- hierarchical spatial partitioning,
- bounding volumes,
- hierarchical LOD,
- screen-space error,
- selective loading,
- refinement instead of all-at-once loading,
- spatial coherence,
- and independent tile/chunk content.

A MakerCAD chunk can therefore be conceptualized as:

```text
Chunk
  +-- stable chunk ID
  +-- authoritative entity IDs
  +-- bounding volume
  +-- spatial parent
  +-- children
  +-- LOD availability
  +-- geometric error estimate
  +-- display representation offsets
  +-- compressed disk location
  +-- memory cost
  +-- priority metadata
```

The important difference is that MakerCAD's spatial tree is a **presentation/streaming structure**, not the authoritative CAD topology.

---

# 87. Representation Graph vs Model Graph

The system should explicitly distinguish two graphs.

## Model Graph

Describes what the object *is*:

```text
Assembly
  -> Component
      -> Body
          -> Feature
              -> Face
                  -> Surface
```

It may also describe:

- constraints,
- parameters,
- semantic relationships,
- source IDs,
- BIM relationships,
- vector layers,
- sketch entities.

## Representation Graph

Describes how the model is currently *shown*:

```text
Spatial Root
  -> Chunk
      -> LOD
          -> Proxy
              -> Render resource
```

The two graphs are connected through stable IDs.

They must not be collapsed into one graph.

---

# 88. Vector Data Requires Its Own Canonical Representation

The vector portion of the original plan should now be strengthened.

For SVG, DXF, DWG, HPGL/PLT, and similar formats, MakerCAD should preserve vector semantics before any rasterization or tessellation.

Conceptually:

```text
VectorDocument
  |
  +-- Document metadata
  +-- Coordinate system
  +-- Units
  +-- Layers
  +-- Groups
  +-- Paths
  +-- Lines
  +-- Polylines
  +-- Arcs
  +-- Circles
  +-- Ellipses
  +-- Bézier curves
  +-- Splines
  +-- Text
  +-- Dimensions
  +-- Styles
  +-- Transforms
  +-- Source IDs
```

A path should remain a path even if Godot sees a tessellated mesh or raster proxy.

This preserves the possibility of:

- converting SVG paths into sketches,
- snapping to curve mathematics,
- editing Bézier control points,
- measuring exact vector distances,
- converting drawings into CAD profiles,
- exporting back to SVG/DXF/DWG,
- and using vector geometry for downstream modeling.

---

# 89. Vector Presentation May Have Multiple Forms

A vector object could choose among:

```text
True Vector Path
       |
       +--> raster proxy
       |
       +--> tessellated fill mesh
       |
       +--> stroke mesh
       |
       +--> polyline approximation
       |
       +--> simplified preview
```

The choice can depend on the task.

For example:

### Far away

Use a raster or very cheap proxy.

### Normal viewing

Use tessellated fills/strokes.

### Selected

Show vector control points and exact path overlays.

### Editing

Use canonical path mathematics for snapping and manipulation.

This is substantially more powerful than simply importing an SVG as a texture.

---

# 90. The Universal Geometry Platform Is Really a Representation Platform

This revision suggests a naming refinement.

The system's core responsibility is not merely:

> import geometry

It is:

> **preserve, transform, cache, stream, query, and present multiple representations of geometric truth.**

The full pipeline becomes:

```text
SOURCE
  |
  v
FORMAT ADAPTER
  |
  v
AUTHORITATIVE REPRESENTATION
  |
  +-----------------------------+
  |                             |
  v                             v
SEMANTIC MODEL             MODEL GRAPH
  |                             |
  +-------------+---------------+
                |
                v
       REPRESENTATION MANAGER
                |
       +--------+--------+--------+
       |        |        |        |
       v        v        v        v
     Mesh    Raster    Lines    Points
       |        |        |        |
       +--------+--------+--------+
                |
                v
             Godot
                |
                v
         Interaction Map
                |
                v
       Authoritative Query
                |
                v
       Model Operation
```

This should be considered the revised target architecture.

---

# 91. Research Validation — Open CASCADE

OCCT is not merely compatible with this architecture; its own visualization model provides strong evidence for it.

OCCT describes visualization as a separation of CAD data from graphical presentation. Its AIS framework supplies presentation objects and selection mechanisms. `AIS_Shape` supports presentation and selection of constituent topology, while `StdPrs_ShadedShape` prepares a triangulated presentation of a shape. OCCT also provides `AIS_Triangulation` for directly displaying a `Poly_Triangulation`.

This establishes an important precedent:

```text
CAD topology / B-Rep
        |
        +--> exact model
        |
        +--> presentation generation
        |
        +--> selection of semantic/topological components
```

MakerCAD should preserve this conceptual separation while adapting the rendering side to Godot.

**Engineering conclusion:** keep OCCT as a source of authoritative CAD truth and treat tessellation as a presentation-generation step.

---

# 92. Research Validation — FreeCAD

FreeCAD provides another adjacent example. Its Part geometry can be tessellated into a simpler mesh for display, and its documentation treats tessellation as a display concern rather than as the definition of the Part object itself.

This reinforces the MakerCAD rule:

> **Tessellation is a representation operation, not a model-destructive conversion.**

It also suggests that MakerCAD should expose tessellation controls internally in terms of visual accuracy, screen-space error, and interaction requirements rather than forcing users to reason about polygon counts.

---

# 93. Research Validation — Cesium 3D Tiles / Massive Dataset Streaming

3D Tiles is especially relevant to the out-of-core portion of this design. Its stated purpose is streaming massive heterogeneous 3D datasets, including photogrammetry, buildings, BIM/CAD, instanced features, and point clouds. It organizes content into spatial trees and uses hierarchical LOD and geometric error to determine refinement.

The strongest transferable lesson is not the file format itself. It is the **runtime strategy**:

```text
Do not ask:
"How can we load the whole model?"

Ask:
"What portion of the model and what fidelity are necessary for this frame and this user action?"
```

This is directly aligned with MakerCAD's low-end-first philosophy.

The additional concept of skipping intermediate LODs is also valuable. MakerCAD should not blindly stream LOD 0 -> 1 -> 2 -> 3 -> 4 if it can safely jump from a coarse representation to a much more useful representation.

---

# 94. Research Validation — Assimp

Assimp remains useful, but the research reinforces the existing recommendation that it should be treated as an **asset-format ingestion component**, not the universal semantic model.

Its supported importer list covers a broad set of polygon and asset formats, including OBJ, FBX, glTF/GLB, STL, PLY, 3MF, Collada, IFC-STEP, STEP, USD, and others.

That makes Assimp valuable for the breadth of the viewer, especially where the application needs broad mesh interoperability.

However, the architecture should avoid treating an `aiScene` as MakerCAD's canonical model. Format-specific meaning must remain available through specialized adapters where necessary.

Recommended rule:

```text
Assimp
  = broad asset ingestion

OCCT
  = exact CAD geometry foundation

Vector adapter
  = mathematical vector foundation

BIM adapter
  = semantic building foundation

Point-cloud adapter
  = spatial point foundation
```

MakerCAD's MIR and Representation Graph sit above them.

---

# 95. Research Validation — ODA and the Reality of Universal CAD Viewing

Open Design Alliance provides a useful reality check on the breadth of the requested universal viewer. Its platform spans DWG/DGN, BIM/Revit/Navisworks/IFC, STEP, SolidWorks, Inventor, CATIA, ACIS, Parasolid, JT, IGES, and other CAD/BIM technologies, while its Drawings SDK targets professional access and visualization of DWG/DGN data.

This demonstrates two things.

First, universal CAD viewing is not realistically solved by one open-source importer.

Second, a robust format architecture is naturally a **family of specialized adapters behind a common application-level representation**.

MakerCAD should therefore plan for three adapter tiers:

### Tier 1 — Open/native libraries

Use where practical and licensing permits.

### Tier 2 — Specialized open-source readers

Use where a format requires dedicated treatment.

### Tier 3 — Commercial SDK adapters

Keep an architectural slot for ODA or other licensed SDKs where format coverage, fidelity, legal interoperability, or production reliability justifies them.

The platform must not be architecturally dependent on any one tier.

---

# 96. Research Validation — libigl and Geometry Processing Separation

libigl provides another useful adjacent pattern: it separates geometry-processing algorithms from the viewer while representing mesh geometry with explicit vertex and face structures.

The lesson for MakerCAD is that geometry processing should remain independently testable from the final renderer.

This supports the proposed modules:

```text
mc::geometry
mc::mesh
mc::tessellation
mc::lod
mc::spatial
mc::optimization
```

rather than putting those responsibilities directly into Godot scripts.

libigl is particularly relevant as an optional geometry-processing toolkit for mesh-specific operations, research prototypes, and validation, but it should not replace OCCT or the MIR.

---

# 97. Revised Universal Architecture

The complete architecture should now be represented as:

```text
                             USER
                              |
                              v
                        MakerCAD UI
                              |
                              v
                     Document Controller
                              |
                              v
                       Import Coordinator
                              |
               +--------------+--------------+
               |              |              |
               v              v              v
          CAD Adapter    Vector Adapter   Mesh Adapter
               |              |              |
              OCCT       SVG/DXF/etc.      Assimp/etc.
               |              |              |
               +--------------+--------------+
                              |
                              v
                   AUTHORITATIVE MODEL LAYER
                              |
               +--------------+--------------+
               |              |              |
               v              v              v
           Model Graph   Semantic Model   Source Mapping
               |              |              |
               +--------------+--------------+
                              |
                              v
                  REPRESENTATION MANAGER
                              |
       +----------------------+----------------------+
       |                      |                      |
       v                      v                      v
  Tessellation          Vector Rasterizer       Mesh Optimizer
       |                      |                      |
       +----------------------+----------------------+
                              |
                              v
                   REPRESENTATION GRAPH
                              |
          +-------------------+-------------------+
          |                   |                   |
          v                   v                   v
      Spatial Chunks       LOD Tree          Proxy Identity
          |                   |                   |
          +-------------------+-------------------+
                              |
                              v
                     Disk-backed Cache
                              |
                              v
                    Streaming Scheduler
                              |
                              v
                    Hardware Budgeter
                              |
                              v
                       Godot Bridge
                              |
                              v
                  Godot Compatibility Renderer
                              |
                              v
                           VIEWPORT
                              |
                              v
                     Picking / Interaction
                              |
                              v
                     Identity Resolution
                              |
                              v
                  Authoritative Model Query
                              |
                              v
                       Model Operation
                              |
                              v
                     Proxy Invalidation
                              |
                              v
                      Targeted Refinement
```

This is the architecture that should supersede the simpler source-to-mesh diagram in the earlier sections.

---

# 98. Revised Core Contracts

The previous five contracts remain valid, but they should be expanded to seven.

## Contract 1 — Importer Contract

> Convert source formats into MakerCAD concepts without requiring Godot to understand the source format.

## Contract 2 — Authority Contract

> Preserve authoritative geometry, semantics, topology, and source identity independently from presentation representations.

## Contract 3 — Representation Contract

> Any renderer representation must be traceable back to an authoritative entity and may be discarded or regenerated without changing model truth.

## Contract 4 — Interaction Contract

> User interaction with a presentation proxy must resolve to an authoritative entity or model operation rather than treating display primitives as design truth.

## Contract 5 — Resource Contract

> Never allow a large model to consume unbounded memory merely because the source file is large.

## Contract 6 — Streaming Contract

> Load and refine the highest useful representation for the user's current view and intent, while preserving exact information for on-demand queries.

## Contract 7 — Renderer Contract

> Godot receives optimized presentation data and is not responsible for understanding the source format or owning authoritative model truth.

---

# 99. Revised C++ Module Architecture

Add the following modules to the previous proposal:

```text
mc::core
mc::formats
mc::mir
mc::model
mc::semantic
mc::geometry
mc::occt
mc::vector
mc::mesh
mc::bim
mc::pointcloud
mc::representation
mc::proxy
mc::identity
mc::selection
mc::tessellation
mc::lod
mc::spatial
mc::cache
mc::streaming
mc::jobs
mc::memory
mc::hardware
mc::diagnostics
mc::renderer
mc::godot
```

### Important ownership boundaries

`mc::model` owns authoritative model concepts.

`mc::representation` owns presentation strategies.

`mc::proxy` owns generated renderer-facing representations.

`mc::identity` maps proxies to authoritative entities.

`mc::selection` resolves user interaction.

`mc::renderer` defines the renderer-independent interface.

`mc::godot` implements the Godot-specific backend.

This prevents Godot from leaking into the model layer.

---

# 100. Representation Manager API — Example

Conceptual C++ interface:

```cpp
namespace mc {

struct RepresentationRequest {
    EntityId entity;
    LodLevel desired_lod;
    float screen_error;
    RepresentationKind preferred_kind;
    bool interactive;
    bool selected;
    bool measurement_relevant;
    int priority;
};

struct RepresentationHandle {
    RepresentationId id;
    EntityId source_entity;
    RepresentationKind kind;
    LodLevel lod;
};

class RepresentationManager {
public:
    RepresentationHandle request(const RepresentationRequest& request);

    void release(RepresentationId id);

    void invalidate(EntityId entity);

    void refine(EntityId entity, RefinementReason reason);

    void update_view(const CameraState& camera,
                     const ResourceBudget& budget);
};

}
```

The important property is that `RepresentationManager` never becomes the owner of the authoritative model.

---

# 101. Identity Map — Example

```cpp
namespace mc {

struct ProxyIdentity {
    RepresentationId representation;
    EntityId entity;
    SourceId source;
    uint32_t primitive_start;
    uint32_t primitive_count;
};

class IdentityMap {
public:
    std::optional<EntityId> resolve(
        RepresentationId representation,
        uint32_t primitive_index) const;

    void bind(const ProxyIdentity& identity);
    void unbind(RepresentationId representation);
};

}
```

The real implementation will likely need ranges, topology mappings, and possibly barycentric or parameter-space mapping for advanced picking.

The conceptual requirement is more important than this exact API:

> **Every interactive display primitive must have a path back to authoritative model identity whenever that interaction is meaningful.**

---

# 102. Interaction Pipeline — Example

```cpp
void InteractionController::on_pick(const PickResult& pick)
{
    auto entity = identity_map.resolve(
        pick.representation_id,
        pick.primitive_index);

    if (!entity) {
        return;
    }

    if (tool.requires_exact_geometry()) {
        geometry_service.ensure_query_ready(*entity);
    }

    selection_service.select(*entity);
}
```

The renderer does not need to know whether the entity originated from:

- STEP,
- SVG,
- DXF,
- OBJ,
- IFC,
- point cloud,
- or a native MakerCAD feature.

That is the responsibility of the model/adapter layers.

---

# 103. Proxy Invalidation Rules

Whenever authoritative data changes, dependent representations must be invalidated rather than silently becoming stale.

Example:

```text
True Face changes
     |
     +--> exact query cache invalidated
     |
     +--> LOD 0 invalidated
     +--> LOD 1 invalidated
     +--> LOD 2 invalidated
     +--> raster proxy invalidated
     +--> selection mapping invalidated
     |
     v
Representation Manager
     |
     v
regenerate only required representations
```

Invalidation should be dependency-aware rather than document-wide whenever possible.

This is particularly important for large assemblies where changing one component should not force the entire project to re-tessellate.

---

# 104. Cache Architecture Revision

The persistent cache should now be understood as storing **derived representations**, not merely an imported mesh.

A cache may contain:

```text
/cache/<document-hash>/
    manifest
    model-index
    spatial-tree
    entity-map
    semantic-cache
    vector-cache
    lod/
        0/
        1/
        2/
        3/
    raster/
    mesh/
    pointcloud/
    materials/
    diagnostics
```

The authoritative source remains the source file or native MakerCAD document.

Cache invalidation must consider:

- source modification timestamp where appropriate,
- content hash,
- adapter version,
- OCCT/library version,
- tessellation algorithm version,
- proxy format version,
- renderer compatibility version,
- and relevant user/import settings.

A cache must never be treated as the only source of truth unless MakerCAD explicitly creates a native document format designed for that purpose.

---

# 105. Low-End Memory Strategy — Revised

The low-end machine should be treated as having multiple independent budgets:

```text
SYSTEM RAM
    |
    +-- authoritative working-set buffers
    +-- importer buffers
    +-- geometry query data
    +-- representation cache buffers
    +-- worker memory
    +-- Godot process

GPU / SHARED GRAPHICS MEMORY
    |
    +-- resident mesh proxies
    +-- textures / raster proxies
    +-- frame buffers
    +-- renderer resources
```

On integrated graphics such as Intel HD 530, CPU and graphics memory pressure are closely coupled. Therefore MakerCAD must avoid treating GPU residency as an independent infinite pool.

The system should degrade in this order where safe:

```text
1. Stop speculative prefetch
2. Drop distant high LOD
3. Drop distant medium LOD
4. Evict invisible proxies
5. Reduce raster resolution
6. Reduce material/texture residency
7. Reduce expensive visual effects
8. Preserve selected and actively manipulated geometry
9. Preserve authoritative model/query capability
```

The most important rule is:

> **When memory pressure rises, sacrifice visual richness before interaction or model truth.**

---

# 106. Do Not Force the Renderer to Render Exact Geometry

An exact B-Rep is not required to be directly rendered as exact mathematical surfaces by Godot.

The correct division is:

```text
OCCT / Vector Kernel
    |
    | exact calculations
    | exact topology
    | exact queries
    v
Representation Manager
    |
    | approximate for visual presentation
    v
Godot
```

This allows MakerCAD to exploit the strengths of both systems:

### Geometry engine

- mathematical precision,
- topology,
- booleans,
- curves,
- surfaces,
- exact measurements.

### Godot

- camera,
- UI,
- interaction presentation,
- lighting,
- rasterized display,
- simple real-time mesh rendering.

Neither side needs to become the other.

---

# 107. The "True Model + Proxy" Principle Should Affect UX

This architecture should be visible to the user only through behavior, not terminology.

The user should never need to understand:

- tessellation,
- proxy meshes,
- cache chunks,
- LOD,
- GPU residency,
- spatial tiles,
- representation graphs.

Instead they experience:

```text
Open
  -> see something quickly
  -> navigate immediately
  -> zoom in
  -> detail appears
  -> select
  -> precise object is identified
  -> edit
  -> result updates
```

The application should feel like it is "keeping up" with the user rather than loading a huge file.

---

# 108. Revised Viewer-First Workflow

For a large file:

```text
DROP FILE
   |
   v
IDENTIFY
   |
   v
SHOW FILE INFORMATION
   |
   v
SHOW STRUCTURE / BOUNDS
   |
   v
SHOW COARSE PROXY
   |
   v
ALLOW CAMERA CONTROL
   |
   v
STREAM VISIBLE DETAIL
   |
   v
REFINE BASED ON USER ATTENTION
   |
   v
EXACT-ON-DEMAND QUERY
```

The application should not make the user wait for the final representation before allowing basic navigation.

---

# 109. Universal Viewer Goal — Revised Scope

The phrase "view all known 3D formats" must be treated as a product aspiration rather than a literal claim that every format will be natively supported by one library.

The practical goal is:

> **Provide a universal viewing architecture capable of integrating specialized adapters for the broadest practical set of 2D vector, 3D CAD, polygon, BIM, point-cloud, and exchange formats.**

A useful coverage matrix is:

| Family | Examples | Canonical representation | Primary presentation |
|---|---|---|---|
| Exact CAD | STEP, IGES, BREP | OCCT B-Rep | Adaptive mesh / edge proxy |
| Polygon | OBJ, STL, PLY | Indexed mesh | Optimized mesh LOD |
| Asset | FBX, GLTF, GLB, 3MF | Scene/mesh representation | Mesh LOD |
| Vector | SVG, DXF, DWG, HPGL | Vector path/entity model | Raster/mesh/line proxy |
| BIM | IFC, RVT* | Semantic object graph + geometry | Mesh LOD |
| Point cloud | LAS, LAZ, E57, PTS, PTX | Point-cloud hierarchy | Sampled point proxy |
| Drawing/document | PDF/vector-derived content | Page/vector model where possible | Raster/vector hybrid |
| Future proprietary | ODA / commercial SDKs | Adapter-defined canonical model | Standard proxy system |

`*` Format support may require specialized/commercial SDKs and licensing.

The architecture must allow support to grow without requiring a renderer rewrite.

---

# 110. Recommended Adapter Strategy

A format adapter should ideally expose at least four levels of capability:

```text
Level 0 — Detect
  Can we identify the file?

Level 1 — Inspect
  Can we read metadata, bounds, hierarchy and basic information?

Level 2 — Present
  Can we generate a useful proxy for viewing?

Level 3 — Semantic
  Can we preserve meaningful source entities and support exact queries/editing?
```

This is important because not every format needs full editing support to be useful.

A user may want to open an unfamiliar file simply to inspect it.

MakerCAD should therefore distinguish:

- **View support**
- **Selection support**
- **Measurement support**
- **Semantic preservation**
- **Editing support**
- **Export support**

These are separate capabilities.

---

# 111. Recommended Import Capability Matrix

For each adapter, maintain a machine-readable capability record:

```yaml
format: STEP
capabilities:
  detect: true
  metadata: true
  hierarchy: true
  geometry: true
  exact_queries: true
  selection: true
  measurement: true
  editing: true
  export: true
  streaming: partial
  progressive_preview: true
presentation:
  mesh_proxy: true
  line_proxy: true
  raster_proxy: false
  lod: true
```

This allows the UI to communicate capabilities without hard-coding format-specific behavior.

For example:

> **STEP — View, Select, Measure, Edit**

versus:

> **STL — View, Select, Measure**

The user should not be presented with a misleading expectation that every format is equally editable.

---

# 112. Revised Development Phases

The earlier implementation sequence should be adjusted to establish the representation layer earlier.

## Phase 0 — Architecture and Measurement

- repository audit,
- dependency/license audit,
- renderer capability audit,
- hardware benchmark harness,
- baseline memory measurements,
- MIR draft,
- model graph draft,
- representation graph draft.

## Phase 1 — Authoritative Model + Minimal Proxy

Implement:

- OCCT STEP adapter,
- canonical B-Rep ownership,
- entity IDs,
- basic tessellation,
- one mesh proxy format,
- Godot bridge,
- identity mapping,
- selection.

**Gate:** select a rendered face and resolve it back to its true OCCT face.

## Phase 2 — Progressive Presentation

Implement:

- multiple LODs,
- screen-space error,
- adaptive tessellation,
- proxy replacement,
- invalidation,
- exact-on-demand queries.

**Gate:** changing LOD must not change entity identity or measurement correctness.

## Phase 3 — Out-of-Core Streaming

Implement:

- spatial hierarchy,
- disk-backed chunks,
- camera-driven loading,
- eviction,
- memory budgets,
- prefetch,
- cancellation.

**Gate:** large models remain interactive without loading the entire presentation into RAM.

## Phase 4 — Mesh Ecosystem

Add:

- OBJ,
- STL,
- PLY,
- GLTF/GLB,
- FBX where supported,
- 3MF,
- mesh optimization,
- mesh-specific LOD.

## Phase 5 — Vector Ecosystem

Add:

- SVG canonical paths,
- DXF entities,
- DWG strategy,
- vector proxy generation,
- raster proxy generation,
- vector selection mapping,
- future sketch conversion.

**Gate:** zooming or rasterizing a vector must never destroy its canonical path representation.

## Phase 6 — BIM / Point Clouds

Add:

- IFC semantics,
- point-cloud hierarchy,
- specialized spatial streaming,
- property/metadata queries.

## Phase 7 — Commercial Adapter Slots

Evaluate:

- ODA,
- additional CAD SDKs,
- proprietary BIM readers,
- high-fidelity exchange formats.

The architecture should already be ready for these integrations.

---

# 113. New Acceptance Criteria — Representation Integrity

The previous acceptance criteria should be expanded.

### Representation

- Every display proxy can be traced to an authoritative entity where the format supports meaningful identity.
- Replacing a proxy does not alter the authoritative model.
- LOD changes do not change entity identity.
- Cache deletion does not destroy model truth.
- Rasterization does not destroy canonical vector data.
- Tessellation does not replace canonical CAD geometry.

### Selection

- A user can select a display proxy and resolve the corresponding model entity.
- Selection can remain valid when the proxy is replaced by another LOD.
- Selection can request exact geometry when required.

### Measurement

- Measurement uses authoritative geometry whenever the source supports exact geometry.
- Measurement does not require maximum display tessellation.

### Editing

- Editing operates on authoritative model data.
- Proxy regeneration occurs after successful model changes.
- Failed model operations do not leave the display representation falsely claiming success.

### Hardware scaling

- A low-end machine can use a lower presentation budget without changing model data.
- A high-end machine can increase presentation quality without requiring a different document representation.

---

# 114. New Test Matrix — Truth vs Proxy

Every supported representation family should be tested against these cases:

```text
Import
  -> canonical data exists
  -> proxy exists
  -> identity mapping exists

LOD change
  -> proxy replaced
  -> canonical data unchanged
  -> identity unchanged

Eviction
  -> proxy removed
  -> canonical data remains available
  -> selection can restore/refetch proxy

Cache rebuild
  -> derived data recreated
  -> canonical semantics preserved

Selection
  -> proxy primitive resolves to true entity

Measurement
  -> canonical query returns expected result

Edit
  -> canonical model changes
  -> dependent proxy invalidated
  -> new proxy generated
```

This test family should become a permanent regression suite.

---

# 115. New Benchmark Suite

Benchmark the architecture as a **representation system**, not merely a renderer.

Measure:

### Import

- time to identify,
- time to metadata,
- time to structure,
- time to first preview,
- time to usable navigation,
- total canonical parse time.

### Representation

- proxy generation time,
- tessellation time,
- LOD generation time,
- rasterization time,
- cache write time.

### Streaming

- time to first visible chunk,
- camera movement latency,
- refinement latency,
- eviction latency,
- cache read latency.

### Interaction

- pick latency,
- identity resolution latency,
- exact-query latency,
- measurement latency,
- edit-to-display latency.

### Resources

- peak RAM,
- steady-state RAM,
- worker memory,
- resident proxy memory,
- texture/raster memory,
- Godot resource count,
- draw calls,
- visible triangles,
- cache size.

The benchmark harness should run against the Intel HD 530 baseline and at least one stronger machine.

---

# 116. Revised Example End-to-End Flow — STEP

User drops `Machine.step`.

```text
0.0s
Format identified: STEP

0.1s
Metadata / units available

0.2s
Assembly structure begins appearing

0.4s
Bounds available

0.7s
Coarse representation available

0.8s
User can orbit and zoom

1.0s
Visible region refined

1.3s
User selects a face

1.3s
Proxy face -> FACE-008472

1.4s
Exact OCCT face resolved

1.5s
Selection highlight appears

2.0s
Background refinement continues
```

If the user asks for a measurement:

```text
User -> Measure radius
          |
          v
      Entity ID
          |
          v
      OCCT Face
          |
          v
     exact query
          |
          v
      measurement
```

No 2-million-triangle mesh is required merely to know the radius.

---

# 117. Revised Example End-to-End Flow — SVG

User drops `logo.svg`.

```text
SVG parser
   |
   v
VectorDocument
   |
   +-- path IDs
   +-- Bézier curves
   +-- fills
   +-- strokes
   +-- transforms
   +-- layers
   |
   +----------------------+
                          |
                          v
                  Representation Manager
                          |
             +------------+------------+
             |                         |
             v                         v
       raster proxy              tessellated proxy
             |                         |
             +------------+------------+
                          |
                          v
                        Godot
```

If the user zooms in, MakerCAD can regenerate the representation at a higher resolution.

If the user edits a control point:

```text
Godot handle
   -> vector entity ID
   -> canonical Bézier path
   -> modify control point
   -> regenerate affected proxy
```

The original vector mathematics remains intact.

---

# 118. Revised Example End-to-End Flow — Huge Mesh

For a 20-million-triangle mesh:

```text
Source file
    |
    v
Mesh adapter
    |
    v
Canonical mesh representation / indexed source
    |
    v
Spatial hierarchy
    |
    +-- chunk A
    +-- chunk B
    +-- chunk C
    +-- ...
    |
    v
LOD generation
    |
    v
Disk cache
    |
    v
Camera sees chunk B
    |
    v
Load B / appropriate LOD
    |
    v
Godot
```

The system does not need all 20 million triangles resident in Godot.

If the camera moves, the working set moves with it.

---

# 119. Revised Example End-to-End Flow — IFC / BIM

BIM data should be treated as a semantic model plus geometry.

```text
IFC
 |
 +--> object identity
 +--> hierarchy
 +--> properties
 +--> spatial relationships
 +--> geometry
          |
          v
      BIM Model Graph
          |
          +--> semantic queries
          |
          v
   Representation Manager
          |
          v
      mesh proxies
          |
          v
        Godot
```

If the user clicks a wall, the system should know:

```text
Wall proxy
 -> IFC object ID
 -> Wall semantic object
 -> properties
 -> exact/derived geometry
```

It should not merely know:

```text
Triangle #18392
```

---

# 120. Revised Example End-to-End Flow — Point Cloud

Point clouds should use the same philosophy but a different presentation strategy.

```text
Point cloud source
      |
      v
Canonical point data
      |
      v
Spatial hierarchy
      |
      +--> coarse points
      +--> medium points
      +--> dense points
      |
      v
screen-space / density selection
      |
      v
Godot point/mesh proxy
```

The point cloud remains point-cloud data. It should not be permanently converted into a solid mesh merely because Godot needs a renderable representation.

---

# 121. Architecture Decision — C++ Remains the Production Path

The new findings do not change the earlier conclusion that C++ should be the production implementation.

They make the reason stronger.

The native layer now has to coordinate:

- OCCT,
- format adapters,
- canonical model ownership,
- vector parsing,
- mesh processing,
- spatial indexes,
- cache storage,
- worker jobs,
- representation generation,
- identity mapping,
- selection,
- resource budgets,
- and Godot resource creation.

A standalone Python process remains valuable for experimentation and test tooling, but it should not become the permanent production boundary between Godot and the geometry platform.

The recommended production boundary remains:

```text
Godot
  <->
GDExtension C++ bridge
  <->
MakerCAD Universal Geometry Platform
```

Avoid localhost TCP for normal in-process geometry traffic unless a future architectural requirement makes process isolation worthwhile.

---

# 122. Architecture Decision — Do Not Build Around Godot's SVG Importer

Godot's built-in SVG handling can remain useful for ordinary UI/icon needs, but it should not define MakerCAD's vector architecture.

MakerCAD needs canonical vector entities that can participate in:

- snapping,
- selection,
- constraints,
- sketch creation,
- measurement,
- transformation,
- extrusion/profile generation,
- and future vector editing.

Therefore:

```text
Godot SVG import
      = UI/display convenience

MakerCAD vector adapter
      = design-data ingestion
```

This distinction prevents a renderer-specific asset pipeline from becoming the application's design model.

---

# 123. Architecture Decision — Preserve Multiple Levels of Truth

MakerCAD should distinguish at least four concepts:

```text
1. Source Truth
   What the original file actually contains.

2. Canonical Truth
   What MakerCAD's geometry/semantic system understands and preserves.

3. Presentation Truth
   What representation is currently sufficient to show the model.

4. Interaction Truth
   What authoritative entity the user is currently manipulating or querying.
```

These can be related but must not be conflated.

Example:

```text
STEP source
   |
   v
TopoDS_Face
   |
   +--> 180-triangle proxy
   |
   +--> user-selected face
   |
   +--> exact radius query
```

Four different representations/concerns are involved, but they all refer to the same underlying design entity.

---

# 124. New Principle — Approximation Is Allowed; Ambiguity Is Not

A low-end machine may use an approximate visual representation.

It must not create ambiguity about what that representation represents.

Therefore:

> **Approximate the picture, not the identity.**

And:

> **Approximate display geometry when necessary; never silently approximate authoritative measurements or model semantics when exact information is available.**

If an exact query is unavailable because the source format itself lacks the information, MakerCAD should state that honestly.

---

# 125. New Principle — The Renderer Can Be Replaced

The Representation Platform should not be architecturally dependent on Godot.

Godot 4.6 Compatibility is the current renderer because it fits the project's hardware target and existing development strategy.

But the core should theoretically be able to support:

```text
MakerCAD Geometry Platform
       |
       +--> Godot Compatibility backend
       +--> future Vulkan backend
       +--> future desktop renderer
       +--> future headless renderer
       +--> future export renderer
```

This is why the renderer-facing API should be narrow.

The core should know about:

- geometry representations,
- resource handles,
- identity,
- visibility,
- LOD,
- materials/attributes.

It should not know about Godot scene-tree organization.

---

# 126. Revised Final Architecture Statement

The strongest final statement for the program plan is now:

> **MakerCAD is a model-centric geometry platform with a hardware-adaptive presentation layer.**
>
> **Authoritative geometry and semantics live outside the renderer. Presentation representations are generated, cached, streamed, refined, and discarded according to camera state, user intent, interaction requirements, and available hardware. Godot is the current visualization and interaction shell, not the source of truth.**

The resulting architecture is:

```text
SOURCE FORMATS
     |
     v
FORMAT ADAPTERS
     |
     v
AUTHORITATIVE MODEL
     |
     +-------------------------------+
     |                               |
     v                               v
SEMANTIC / MODEL GRAPH        EXACT GEOMETRY / VECTORS
     |                               |
     +---------------+---------------+
                     |
                     v
            REPRESENTATION MANAGER
                     |
       +-------------+-------------+
       |             |             |
       v             v             v
     MESH          VECTOR        RASTER
     PROXY         PROXY         PROXY
       |             |             |
       +-------------+-------------+
                     |
                     v
             SPATIAL / LOD GRAPH
                     |
                     v
              DISK-BACKED CACHE
                     |
                     v
              STREAMING MANAGER
                     |
                     v
              RESOURCE MANAGER
                     |
                     v
               GODOT BRIDGE
                     |
                     v
             GODOT VIEWPORT
                     |
                     v
             USER INTERACTION
                     |
                     v
              IDENTITY MAP
                     |
                     v
             EXACT QUERY / EDIT
                     |
                     v
             AUTHORITATIVE MODEL
```

This should replace any earlier architecture diagram that implies a one-way conversion from source data to display mesh.

---

# 127. Research and Reference Notes

The following adjacent projects and technologies informed this revision:

1. **Open CASCADE Technology (OCCT)** — especially its explicit separation of CAD data and graphical presentation, AIS presentation/selection model, `AIS_Shape`, `StdPrs_ShadedShape`, and `AIS_Triangulation`.
2. **FreeCAD** — practical separation of higher-level Part geometry from tessellated display meshes.
3. **Cesium 3D Tiles** — hierarchical spatial streaming, HLOD, bounding volumes, geometric error, screen-space error, and selective refinement for massive datasets.
4. **Assimp** — breadth of asset-format ingestion and the value of treating broad format support as an adapter concern.
5. **Open Design Alliance (ODA)** — evidence that professional universal CAD/BIM interoperability requires a family of specialized SDKs/readers rather than one universal parser.
6. **libigl** — separation of geometry processing and visualization, with lightweight mesh representations and independently testable geometry algorithms.
7. **CGAL** — geometry-kernel separation between mathematical representation and algorithms, reinforcing the idea that the computational geometry layer should not be dictated by the renderer.

These projects are not being treated as templates to copy. They are evidence that the proposed separation of **model truth, geometry processing, presentation, selection, and streaming** is consistent with established approaches in adjacent engineering systems.

---

# 128. Sources Consulted for This Revision

- Open CASCADE Technology — Visualization overview and separation of CAD data from graphical presentation.
- Open CASCADE Technology — AIS custom presentation and independent selection mechanisms.
- Open CASCADE Technology — `AIS_Shape` reference and topology-aware selection.
- Open CASCADE Technology — `AIS_Triangulation` reference.
- FreeCAD documentation — Part-to-mesh tessellation.
- FreeCAD documentation — tessellation as a 3D-view display requirement.
- Cesium 3D Tiles specification — massive heterogeneous dataset streaming, spatial hierarchy, HLOD, bounding volumes, and screen-space error.
- Cesium — skipping LOD levels to reduce load time, requests, and memory.
- Assimp — supported importer/exporter formats.
- Open Design Alliance — Drawings SDK and broader CAD/BIM platform coverage.
- libigl — geometry processing and viewer separation.
- CGAL — geometry kernel representation architecture.

See the project research log for the exact URLs and version/date checks used during final implementation planning.

---

# 129. Final Engineering Recommendation After Revision

The project should proceed with the following architectural commitment:

### Build this:

```text
Native C++ Universal Geometry Platform
+
OCCT authoritative CAD representation
+
Specialized format adapters
+
MakerCAD model/semantic graph
+
Vector-native canonical representation
+
Presentation Representation Manager
+
Proxy identity mapping
+
Exact-on-demand geometry queries
+
Spatial hierarchy
+
Hierarchical LOD
+
Screen-space error driven refinement
+
Disk-backed cache
+
Out-of-core working set
+
Camera/intent driven streaming
+
Adaptive resource budgets
+
Godot GDExtension renderer bridge
```

### Do not build this:

```text
file
 -> parse everything
 -> convert everything to triangles
 -> load everything into RAM
 -> send everything to Godot
 -> hope the HD 530 survives
```

The first architecture is a platform.

The second is a viewer with a crash threshold.

MakerCAD's mission requires the first.

---

# 130. Closing Product Philosophy — Updated

The original philosophy remains correct but can now be stated more precisely:

> **The computer should adapt to the model and the user's intent, not force the user to adapt to the computer.**

And the implementation principle beneath it is:

> **The model should remain more capable than the machine's current display budget.**

A technically obsolete computer should not force a technically obsolete model representation.

A powerful computer should not require a different model representation.

The renderer should simply show as much of the model as the current machine can afford, at the fidelity the current user action requires.

The user should be able to:

- open a model,
- see it quickly,
- navigate it,
- select it,
- measure it,
- understand it,
- and eventually edit it,

without knowing whether the current picture is:

- a raster proxy,
- a 500-triangle mesh,
- a 500,000-triangle mesh,
- a vector tessellation,
- a point sample,
- or an exact-on-demand query.

That complexity belongs to MakerCAD.

The user gets the experience.

The model keeps the truth.

The renderer gets only what it can afford.

**That is the architecture that gives MakerCAD the best chance of making serious 3D/CAD work accessible to people whose hardware would normally exclude them, while still scaling upward to much stronger systems without splitting the product into separate low-end and high-end versions.**
