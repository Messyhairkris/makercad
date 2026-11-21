# MakerCAD v5 — status

Live status. Nothing enters this file without a cold run on the reference machine
(i5-6600 / HD 530 / 8 GB) and the exact command beside it. That rule is the one
lesson of v4 that cost the most to learn.

## 2026-11-12 — M0 skeleton

| Claim | Command | Result |
|---|---|---|
| Workspace builds: 16 crates + xtask | `cargo test --workspace` | PASS, all crates, 3 xtask tests |
| The layer gate reads the workspace | `cargo run -q -p mc-cli -- layers` | `layers ok: 49 dependency edges checked` |
| The layer gate bites | plant `mc-view` under `mc-kernel` | `FAIL mc-kernel (L1) depends on mc-view (L5): up a layer`, exit 1 |
| The gate found a real omission on its first run | `cargo test -p xtask-layers` | failed: `mc-geom and mc-id are both L0 with no ordering rule` — rule added |
| CLI front end | `cargo run -q -p mc-cli -- version` | prints 14 crates with their layers |

## 2026-11-12 — M1 part 1: OpenCASCADE answers Rust

| Claim | Command | Result |
|---|---|---|
| OCCT is reachable from Rust through a plain-C shim | `cargo run -q -p mc-kernel --example kernel_report` | `linked: true`, `version: 7.9.3` |
| A real B-rep is built and measured by the kernel | same | box 60x40x20: `volume 48000.000000 mm^3`, `area 8800.000000 mm^2`, both exact |
| Bad input is refused before the kernel is touched | `cargo test -p mc-kernel` | 3 tests pass; zero and negative dimensions return `BadDimension` |
| A clean build finds the runtime DLLs | `cargo test -p mc-kernel` on a fresh `target/` | build.rs copies the OCCT and 3rd-party DLLs beside the binaries |
| The workspace still builds without OCCT | unset `MC_OCCT_DIR`, point it at a bad path | stub compiles; the kernel test fails with a sentence naming the path (never passes silently) |

OCCT 7.9.3 at `C:\OpenCASCADE\opencascade-7.9.3-vc14-64`, 83 OCCT DLLs and 332
third-party DLLs beside the test binary. The wall holds: `mc-kernel` is the only crate
that names OCCT, and the shim exposes numbers only.

Not yet proven: a window, a pixel, a measured startup time, a measured frame. Those
are M0 part 2, and they are the next thing I do.

## 2026-11-12 — M0 part 2: the window

| Claim | Command | Result |
|---|---|---|
| The window opens, draws the grid and exits on its own | `target/debug/mc-desktop.exe --frames 180` | 181 frames, exit 0, **first frame 49.0 ms**, mean update **0.94 ms**, worst 43.5 ms |
| The camera and the ground barrier are right | `cargo test -p mc-view` | 7 tests pass: the target projects to the screen centre, pitch clamps at 4 deg and 89 deg, a point behind the eye is dropped, a horizontal pan does not drift in Z, zoom is multiplicative and bounded |
| The whole workspace is green | `cargo test --workspace` | 13 tests: mc-kernel 3, mc-view 7, xtask 3 |
| **Vulkan is unusable on the reference machine, and wgpu does not fall back** | `WGPU_BACKEND=vulkan mc-desktop.exe --frames 1` | **segfault, exit 139, no output at all.** With `WGPU_BACKEND=dx12` first frame 79.8 ms; with `gl`, 48.5 ms |
| The app therefore names its backends | in `mc_ui::run` | defaults to `dx12,gl` unless the user sets `WGPU_BACKEND`; this is a stand-in for the capability probe |
| Release build | `cargo build --release -p mc-desktop` | **NOT MEASURED** — exceeded a 15-minute ceiling in this session. Debug numbers above only |

The Vulkan segfault is the most useful result of the day: it is the exact failure the
capability probe exists to prevent, on the exact machine it was designed for. A program
that trusts the OS to pick a graphics backend does not run here.

Still unmeasured: idle RSS, orbital drag frame rate under real input, and the release
build. All three are in the task board, not in a claim.


## 2026-11-12 — M1 part 2: real solids, and the window shows them

| Claim | Command | Result |
|---|---|---|
| A body is built, measured, cut and extruded through the shim | `cargo test -p mc-kernel` | 7 pass: box volume 48000; cut removes exactly 2000 (46,000 left); a 10x10 square extruded 5 is 500 mm^3; a box meshes to at least 12 triangles |
| The whole workspace | `cargo test --workspace` | **17 passed, 0 failed** |
| The window draws a solid and self-closes | `target\debug\mc-desktop.exe --frames 240` | 241 frames, **first frame 65.3 ms**, mean update **1.69 ms**, wall 7.0 s |
| A dimension change is a real rebuild | `run.ps1`, drag W | shape and mm^3 update live; 60x40x20 reads 48000 |

Two bugs found and fixed on the way, both mine, both invisible without a gate:

1. **Handle table had no sentinel.** Index 0 meant "no shape", but the first shape
   ever built also got index 0, so it was reported as a failure. Only the first
   allocating test failed — which is why it looked like an OCCT problem.
2. **Shared handle table had no lock.** Rust runs tests in parallel, so two tests
   corrupted each other's shapes. Every shim entry point now takes a mutex; OCCT is
   not safe to call from two threads anyway.

The shim now reports **why** a call failed (`mc_last_error`), so a kernel refusal reads
"cut threw: ..." instead of "0". That is what turned both bugs from guesswork into facts.

Not measured: release-build timings (debug only), idle RSS, frame time under a real drag.

## What is decided (2026-11-12)

- Helper character is **Sancho** (Sancho Panza). Persona is data, not code.
- The project log starts at creation with "what are you making?", is editable for the
  life of the project, prompts for notes at close, and offers an optional AI rewrite in
  the user's own style at conclusion. A project with skipped prompts is **Incomplete**.
- **Lamina** is part of the design as a source of *concepts only* — it is AGPL-3.0, so no
  code is copied. Its five construction techniques and its assembly checks are adopted.
- Install-time **capability probe** chooses the performance profile, with "check again".


## 2026-11-13 - pass 2: sketch, solver, extrude modes, picking, and the MakerCAD shell

Full record: `docs/INCREMENT_2.md` (what changed, how it was checked, known limits).

| Claim | Command | Result |
|---|---|---|
| The workspace is green | `cargo test --workspace` | 144 passed, 0 failed (was 45) |
| The layer law holds | `cargo run -q -p mc-cli -- layers` | `layers ok: 57 dependency edges checked` |
| The increment gate | `python gates/increment.py` | OK |
| The window runs the example | `target\debug\mc-desktop.exe --template --frames 120` | exit 0, mean update 13.93 ms, debug build |
| Draw, dimension, round, extrude, resize | `cargo test -p mc-ui` | 24 pass, including the making path driven through the tool functions |

Not proven: the real window has been captured but never operated by pointer or keyboard;
the questions and end-of-session screens have not been seen on screen; release timings.
