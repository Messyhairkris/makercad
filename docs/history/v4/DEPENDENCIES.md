# MakerCAD v4 — dependency & license ledger

Release gate: this file must be current before any external dependency ships.
Originated from the Universal Geometry Proposal comparison (2026-08-12);
proved necessary by the C4a licence decision.

| Dependency | Purpose | Version | License | Linking model | Constraints/notes | Replacement difficulty |
|---|---|---|---|---|---|---|
| OpenCASCADE | CAD kernel (B-Rep, booleans, fillet/chamfer, STEP/STL IO) | 7.9.3 | LGPL-2.1 with additional exception (OCCT exception) | Dynamic — OCCT ships as a multi-DLL toolkit set alongside `makercad.*.dll` | Ship license text; user-swappable DLLs satisfy replaceability; include exception notice | Very high (kernel) |
| godot-cpp | GDExtension C++ bindings | godot-4.5-stable pin (junction at `extern/godot-cpp`) | MIT | Compiled into extension TU set (static portions); GDExtension boundary is runtime | Attribution in About/licenses | Low |
| planegcs (vendored, FreeCAD) | 2D sketch constraint solver (C4a) | commit faf5fea (2026-08-22), VENDORED 2026-08-23 under `vendor/planegcs/` (Kris approved) | LGPL-2.1+ | **Dynamic — own `makercad_solver.dll`** (decision 2026-08-12, option B) | Ship LGPL-2.1 text (TODO); keep vendored source visible in repo (done); user-swappable DLL = relink path. Coupled to FreeCAD — needs FCConfig.h / Base/Console.h / boost_graph_adjacency_list.hpp shims (see vendor/planegcs/NOTICE.md) | Medium (behind `sketch_solver` PIMPL wall) |
| Eigen (vendored with planegcs) | planegcs' linear algebra (`Core`, `Dense`, `QR`) | pinned at vendoring (TODO fetch into `vendor/eigen/`) | MPL-2.0 | Header-only (source form) | MPL file-level source availability satisfied by repo | Low (standard templates) |
| Boost (Graph + Math, header-only) | planegcs graph partitioning + math constants (NEW — found during C4a vendoring) | pinned at vendoring (TODO) | Boost Software License 1.0 | Header-only | BSL-1.0 attribution in About/licenses; header subset only (graph/connected_components, graph/graph_concepts, math/constants) | Low |
| Godot 4.6.2 | runtime/headless host (asset, not linked) | 4.6.2 stable | MIT | n/a (external engine) | redistributed editor binary only if shipped with app | Medium |
| AHK bridge / tooling, Python | verification + desktop automation | n/a | user-local | n/a | not shipped | Low |

## Adding a dependency — checklist

1. Record row here (license confirmed from upstream repo, not memory).
2. State the linking model explicitly; static-linking an (L)GPL dependency is
   a decision that requires Kris's sign-off.
3. Isolation: the dependency must live behind a containment wall (TU or
   PIMPL) like OCCT/planegcs — no leaked types past it.
4. About/licenses UI surfaces this file's contents.
