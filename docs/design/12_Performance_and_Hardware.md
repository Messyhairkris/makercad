# 12 — Performance and Hardware

## 12.1 The reference machines

| Name | Spec | Role |
|---|---|---|
| **Potato (floor)** | i5-6600 (4C/4T), Intel HD 530, 8 GB RAM, SATA SSD, Windows 10, **no usable Vulkan** (segfaults; v5 STATUS 2026-11-12) | Every release must pass here |
| **Modest** | 4–6 core CPU from about 2018 or later, iGPU (Iris Xe / Vega) or entry GPU, 16 GB | Typical laptop |
| **Capable** | 8+ cores, discrete GPU with ≥ 6 GB VRAM, 32 GB | Splats, physics, path tracing |

**Day-one task:** build stock SolidCraft and run it on the potato before writing any MakerCAD code (doc 14 P0). It is wgpu-based, so it is likely exposed to the same Vulkan crash. If it is, set the backend from the probe, and report it upstream.

## 12.2 The capability probe (`mk-probe`)

Kept from v5 (B3) and extended. It runs at first launch, from "Check again" in settings, and headless via `mk-cli probe`. It **never blocks startup**: a failed probe means `potato`.

Steps:
1. **Backends:** try DX12, then GL (and Metal/Vulkan on other OSes where safe). Each attempt runs **in a child process**, so a driver segfault kills the child, not the app. Record what initialises and the adapter info.
2. **Viewport benchmark:** 2 s, 200k triangles, storybook shading, on the real viewport path. Record frame-time percentiles.
3. **Skinning benchmark:** Sancho's LOD set (doc 07 §7.3.1). Record the ms per frame per tier.
4. **CPU:** core count, a short single- and multi-thread kernel benchmark (a boolean and tessellation of a reference part).
5. **Memory:** total and available RAM; VRAM if reported.
6. **Disk:** read speed of the app directory and cache directory (scans are disk-bound on potato).
7. **Displays:** count, resolutions. A second display may be a projector (offer projector setup if a projector is in inventory).

The probe writes a **hardware profile**:

```toml
[profile]
class = "potato"           # potato | modest | capable
backend = "dx12"           # or "gl" | "cpu"
tri_budget = 200_000
draw_call_budget = 60
lod_aggressiveness = "high"
sancho = "baked_vat"       # full | lod8k | baked_vat | portrait
splat_render = false
physics = "kinematic"      # kinematic | dynamic
mem_budget_mb = 2_500      # environment host budget (doc 06 §6.22)
background_jobs = 2
path_tracer = false
```

**Profiles never change model meaning** (v4 proposal §36): only display, speed and which optional features are offered. A design made on a capable machine is exact on a potato.

## 12.3 The computer as a tool

The profile becomes the `computer` item in the inventory (doc 04 §4.3.5), with **capabilities**:

| Capability | Requires (indicative) |
|---|---|
| `render.storybook` | any (CPU floor) |
| `render.splat` | modest+ with a working GPU backend |
| `render.pathtrace` | capable |
| `sim.kinematic` | any |
| `sim.dynamic.realtime` | modest+ |
| `scan.process.large` (> 20M pts in memory) | capable, or "chunked overnight" on less |
| `slice.local` | modest+ (sidecar) |
| `generative` | capable (sidecar), or remote sidecar |

Sancho uses these exactly as he uses tools: "Rendering the splat needs more GPU than this PC has, so I'll show the room as points. Want to render it on your other computer later?" Projects record which machine did what, which helps shared and multi-machine projects.

**Remote sidecars** (later): a capable machine on the LAN runs `mk-sidecar-host`, and a potato can send scan processing, slicing or generative jobs to it. This is opt-in and local network only.

## 12.4 Budgets and targets (to be measured; nothing is claimed until STATUS has a row)

| Target (potato) | Budget |
|---|---|
| Cold start to opening screen | < 2 s |
| Open a typical project (dresser, 60 bodies) to interactive | < 4 s |
| Idle RSS on the opening screen | < 150 MB |
| RSS: dresser project, Sketch + Solid + Stock loaded | < 600 MB |
| Orbit frame rate, dresser + room shell | ≥ 30 fps (p95 frame < 33 ms) |
| Commit-to-exact (feature edit to rebuilt solid), p95 | < 400 ms |
| 20-feature timeline full rebuild | < 1 s |
| Material meter update after commit (≤ 200 parts) | < 300 ms |
| Never freeze the UI | No main-thread stall > 100 ms (enforced by a watchdog in debug builds, which logs offenders) |
| Sancho muted | 0 ms per frame, 0 animation memory |
| 10M-point scan preview | < 10 s to first ghost |

## 12.5 How we stay light

1. **Initialise on demand** (doc 03 §3.7): dormant environments cost nothing but disk.
2. **No OCCT payload.** solidcore is linked in, with no 400-DLL load at startup and no global C++ static initialisation.
3. **Threading:** kernel operations run on a worker pool. The UI thread only draws and handles input. Long jobs (layout, scan processing, export) are cancellable background jobs with progress chips.
4. **LOD by screen-space error** for display meshes. Tessellation tolerance adapts to profile and zoom (precision on demand, v4 §37).
5. **Caches on disk**, keyed by content hash (tessellations, scan products, thumbnails, step images). Disk is part of the runtime (v4 §24). A "Clear caches" button shows sizes.
6. **Frame pacing:** redraw only on change (egui reactive mode). An idle app uses ~0% CPU.
7. **Potato defaults:** no MSAA (the ink outlines hide most aliasing), no shadows, no animated UI, Sancho as baked VAT or portrait, 2 background jobs.
8. **"Keep everything loaded"** is offered only on capable machines, and is off by default.

## 12.6 Measuring (keep the v4/v5 discipline)

- `mk-cli bench` runs a scripted session (open the golden project, orbit 10 s, edit 5 parameters, re-layout, export a PDF) and prints the budget table with pass/fail.
- A **STATUS.md** row is required for every target, with the machine, command and result. Nothing is "done" without one.
- CI runs the bench on a software-rendered runner for regressions. The potato run is manual per release until a potato CI runner exists. (An old office PC is cheap. Make one.)
