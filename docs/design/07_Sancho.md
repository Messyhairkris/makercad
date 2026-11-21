# 07 — Sancho

> Sancho Panza followed Don Quixote and, with common sense and loyalty, helped turn a fantasy into something that happened on real roads. In MakerCAD, Sancho is the companion who turns the user's idea into something that can be built with what they actually have.

## 7.1 Role

Sancho does four jobs:

1. **Scale**: he is the human-sized (or giant) reference in every scene.
2. **Conscience**: he notices problems (physics, materials, tools, safety, budget, deadline) and says so, plainly and once.
3. **Guide**: he explains tools, answers "why?", and offers the next useful step.
4. **Chronicler**: he runs the session-end interview and keeps the log honest.

What he is **not**: a chatbot bolted to the side, a gate that blocks work, or the place where design happens. Geometry is made by commands. Sancho can *propose* commands.

## 7.2 Three layers, each one optional

This is kept from v5 C2 because it was right.

| Layer | What | Cost | Runs offline | Failure mode |
|---|---|---|---|---|
| **Rules** (`mk-assist::rules`) | Pure function: project state in, findings out. Deterministic, tested against golden projects | ~0 | Yes | None; it is the floor |
| **Voice** (`mk-assist::persona`) | Words, tone, timing, mood, animation choice, relationship memory | ~0 | Yes | Can annoy, so it has a mute |
| **Mind** (`mk-assist::mind`) | Optional AI over MCP: answers questions, proposes changes, writes drafts | Money or compute, latency | With a local model | Degrades to Rules + Voice, says so once |

**Target:** Rules alone produce at least 60% of all findings on the golden projects. The Mind is a bonus, never a crutch.

## 7.3 Presence and scale

| Scale class | Sancho appears as | Behaviour |
|---|---|---|
| room / furniture | Full figure at true height (default 168 cm; user-settable, e.g. to their own height) | Stands beside the work, walks to what he's talking about, opens drawers, sits on the bed, reaches for the top shelf |
| body | Full figure; can wear or carry the design | Backpack on his back with straps adjusted to torso length; tote on shoulder |
| bench | **Giant**: hands and face at the bench edge, leaning over | Points with a huge finger; holds the part between finger and thumb to show its real size |
| macro | A giant eye behind a magnifying glass | — |

- **True scale is a measurement tool.** Hover him and you get his height. Drag him to a spot and he shows reach envelopes (seated and standing) and eye height.

### 7.3.1 The rigged model (PM direction, 2026-11-19)

Sancho is a **rigged 3D humanoid with Mixamo- or Rigify-style bones**, delivered as **glTF 2.0 (.glb)** with skin and animation clips.

**Runtime**
- **Skinning runs on the GPU** in `viewport-gpu` (linear blend skinning, at most 4 influences per vertex, up to 64 bones). A **CPU-skinning path** feeds the same posed mesh to the CPU rasteriser, so the CPU floor still shows him.
- **The root bone fixes his height**: 1.68 m by default, or the user's own height. Scaling is uniform from the root, so he is always a true dimension line. **Bench-scale "giant" Sancho is the same rig**, scaled up uniformly, with the camera framing his hands and face at the bench edge. One asset serves every scale class.
- **Light IK only:**
  - *look-at* on the head and neck chain (clamped yaw and pitch) aimed at the selection, the cursor or the part a finding refers to
  - *point-at*: a two-bone IK on one arm, so he can point to a problem part
  
  It is cheap and very expressive.
- **Clip set:** `idle`, `wave`, `point`, `think`, `worried`, `celebrate`, `walk_to`, `sit`, `carry_board`. Optional later: `reach_high`, `crouch_look`, `measure` (tape measure out), `shrug`.
- **Blending:** simple **crossfades** (0.2–0.3 s) between clips, plus additive IK on top. **No state-machine engine.** A small table maps Sancho events (finding raised, milestone, asked, walking to a target) to a clip.
- `walk_to` uses straight-line travel with a turn-in-place, avoiding the room shell's floor obstacles via a coarse 2D grid. No navmesh library.

**By hardware profile** (the probe decides; doc 12)

| Profile | Mesh | Animation | IK |
|---|---|---|---|
| **capable** | Full mesh (≈ 20–30k tris), full-res textures | GPU skinning, all clips, crossfade blending | look-at + point-at |
| **modest** | LOD ≈ 8k tris | Fixed clip set, crossfades, **no IK blending** (pre-authored point and look variants) | off |
| **potato** | LOD ≈ 3k tris **or** 2D portrait | **Baked vertex animation** at a low frame rate (≈ 12 fps, CPU-friendly), or a portrait with sprite expressions | off |
| **any, muted** | — | **Zero animation cost**: no skinning, no clip update, no draw if Off | — |

The potato choice between baked VAT and the portrait comes from the probe's measured frame time with the baked Sancho in view. If he costs more than 2 ms per frame, use the portrait plus a static **cardboard cut-out** standee for scale (built, fittingly, with the Fold environment).

**Bring your own Sancho**
- Any **humanoid glTF** with **Mixamo-style bone naming** (`mixamorig:Hips`, `…Spine`, `…Head`, `…RightArm`…) is accepted. Rigify exports are mapped through a built-in Rigify-to-Mixamo name table.
- The persona TOML maps the model's clip names to Sancho's events and sets height:

```toml
[persona]
name = "Sancho"
model = "sancho.glb"
height_m = 1.68            # or "user"
portrait = "sancho_portrait.png"
lod = { modest = "sancho_lod8k.glb", potato = "sancho_vat.glb" }

[bones]                    # only needed if names aren't Mixamo-standard
head = "mixamorig:Head"
point_arm = ["mixamorig:RightArm", "mixamorig:RightForeArm", "mixamorig:RightHand"]

[clips]                    # event → clip name in the glTF
idle = "Idle"
wave = "Waving"
point = "Pointing"
think = "Thinking"
worried = "Nervously Look Around"
celebrate = "Victory"
walk_to = "Walking"
sit = "Sitting Idle"
carry_board = "Carry Heavy Object"
```

- Import validation checks the bone count, the influences per vertex, that the triangle count is within the profile budget (a LOD is auto-generated with QEM decimation if not), that the clips exist, and that the rest-pose height is sane. Problems come back as honest messages.

**Licensing note.** Mixamo characters and animations can be used in projects and products, but Adobe's terms restrict redistributing the raw assets as standalone files. A user's own Mixamo-sourced Sancho is fine. For the **default Sancho that ships with MakerCAD**, author our own model and clips (Blender + Rigify, exported with Mixamo-compatible naming), so the bundled asset is unambiguously ours to distribute under our licence. Get a quick legal read before shipping any Mixamo-derived clip in the installer.
- **Persona is data** (`personas/sancho.toml`): name, models, portraits, voice lines, tone, thresholds. The user can rename, re-skin or replace him (a robot, a cat, their grandad). Core code never contains the name.
- **Look:** storybook-cartoony to match the theme (doc 11). He's a sturdy, cheerful workshop companion with a flat cap, apron, tape measure on the belt and a pencil behind the ear, not a knight's squire in armour. The Quixote nod is in the name and in a few lines of dialogue.

## 7.4 Findings

Every finding has a **kind**, a **severity**, a **what**, a **why**, a **fix** (a link to a command, preferably one click), a **source** (rule id or ⚡ Mind) and a **location** (an entity, a step or a field).

| Kind | Meaning | Example |
|---|---|---|
| `forbidden` | Physically won't work | "A 40 mm screw through 18 mm stock into nothing." |
| `unsafe` | Physically possible but dangerous | "This is PVC. Laser-cutting it releases chlorine gas, which harms you and the laser." **Needs an explicit confirm** |
| `denied` | Breaks a rule the user set | "You said hand tools only; this rip cut is 1.8 m long. A circular saw and guide would make it 10 minutes instead of an hour." |
| `not_suitable` | Wrong for this material or process | "Particleboard won't hold screws in its edge well. Use cam locks or confirmat screws." |
| `inventory` | Exceeds or misuses what you have | "You're 2 boards short (need 17 × 2x4 @ 3 ft; you have 15)." |
| `fit` | Doesn't fit a Space or a clearance | "The left side clips the alcove's skirting board by 6 mm." |
| `schedule` | Deadline or plan risk | "Glue-up needs 24 h cure; that pushes finishing past Dec 12." |
| `budget` | Money | "These drawer slides take you to 92% of the budget." |
| `tip` | Useful, not urgent | "Cut all four drawer sides in one setup so they match." |
| `praise` | A milestone reached | "All parts fit your boards with 9% waste. That's really good." |

**Speaking rules** (v5 rule 5, kept):
- He **speaks aloud or pops a bubble** only for forbidden, unsafe, denied, inventory and fit, or when asked, or at a milestone, or in **coach mode**.
- Everything else waits in the **Findings panel** (a small badge count on his portrait).
- One finding is said **once**. If it is dismissed, it stays quiet until the condition changes materially.
- **Never blocks** except `unsafe`, which needs one explicit confirm. Overrides are recorded in the journal with an optional reason.
- **Silence** (voice off, panel still updates), **Mute** (nothing, no animation, no frame cost) and **Off** (removed) are each one click. Remembered per project and globally.

## 7.5 The rule packs

Rules are grouped into packs that load with their environment. That way Sancho gets smarter as environments load and costs nothing for unloaded ones.

| Pack | Loaded with | Example rules (≈ count at v1) |
|---|---|---|
| core | always | intent missing; skipped prompt relevant now; unit sanity (a 3 m pencil) (15) |
| stock | Stock & Cut | over-consumption; grain against load; part longer than any board; kerf ignored; defect overlap; offcut reuse opportunity (25) |
| structure | Solid | span vs. thickness sag table (shelves); screw length vs. stock; edge distance for screws and dowels; unsupported part (lamina-style "held by nothing"); too-thin wall; slot severs part (30) |
| tools | always (with inventory) | step requires a capability you lack; accuracy of chosen tool vs. tolerance; noise vs. work windows (20) |
| fold | Fold | panel won't close; flap collision; crease too close to cut; caliper allowance missing; perf tie too weak; glue flap on wrong side (25) |
| soft | Soft Goods | distortion over the fabric's stretch; seam allowance mismatch between paired edges; notch missing; zipper longer than its seam; webbing width vs. buckle (20) |
| motion | Motion | collision along path; pinch point; counterbalance insufficient; tip-over risk (tall, shallow, drawers out) (15) |
| space | Space | fit; clearance; door swing; blocks outlet or vent; walkway < 600 mm (15) |
| plan | Plan | behind schedule; cure time not scheduled; tool unavailable that day; deadline impossible (12) |
| print | 3D Print | overhang > 45° without support; wall < 2 × nozzle; part bigger than the bed (10) |
| safety | always | PVC/vinyl laser; finishes in unventilated space; mains wiring; child-furniture anti-tip; load-bearing wall-mount anchors (15) |

Rules are **Rust functions with data tables** (sag tables, screw charts, material properties). Each has golden tests: v5's acceptance of 5 seeded faults with 0 false flags, extended to every pack.

## 7.6 "Why?" and "Explain this"

From the UX specification §33/§62:
- **Why?** on any number: material meter, waste %, a sag warning, a plan date. It opens the derivation: inputs, formula or rule, and source ("Sag table: American Wood Council span data for pine, 30 kg/m load").
- **Explain this** on any tool or panel: a 2–3 sentence explanation plus an optional 10-second demo clip, played by Sancho in a mini viewport using a tiny sample model. The clips are scripted command sequences, so they cost no video.

## 7.7 The session-end interview

When the user closes the project, or after 2+ hours of activity, or on "Wrap up", Sancho offers a short interview. It takes under a minute, every field is skippable, and skipped fields show as such.

1. **What did you do today?** Pre-filled draft from the session's command log ("You added 4 drawers, changed the top from pine to plywood, and generated cut diagrams"). The user edits it.
2. **What worked?**
3. **What didn't, and why?** Chips: *measurement was off · material behaved unexpectedly · tool limitation · design changed · ran out of time · other*.
4. **Any photos or video?** Drop zone; phone-to-PC transfer with a QR code on the local network (optional).
5. **What's next?** Becomes the "Keep going" hint and a Plan task.
6. Optional: **How are you feeling about it?** (😟 🙂 🤩), which tunes Sancho's tone next session. Purely local.

The interview becomes a `session` record in the journal (doc 10 §10.4).

**Tone rules for the log** (from v5 risk F4: "Incomplete reads as punishment"): incomplete prompts are shown as *"open pages"* in the log, not as failures. Sancho never guilt-trips. "Skip" is a first-class button, and he says "No problem, I'll keep the page open."

## 7.8 The Mind (optional AI)

- **Off by default.** Turning it on asks for the provider (a cloud API key, or a local model endpoint), shows what is sent and what is not, and runs a test.
- **Boundary:** the Mind is an **MCP client** of MakerCAD's own server. It can read project state through query tools and **propose** commands. Proposals show as a ⚡ diff ("Change shelf thickness 12 → 18 mm; add centre support"), and the user applies them with one click. Applied changes are ⚡-marked in history and the log, and undo as one group.
- **Good uses:** answering "how do I…" in context; turning a vague goal into success criteria; proposing design alternatives that fit inventory (doc 08 §8.5); drafting the published build guide from the journal *in the user's own voice* (the style sample is their own entries); vision intake for inventory photos; the experimental photo-to-part modelling.
- **Bad uses (not allowed):** silent geometry edits; overriding rules; making up material properties without marking them as estimates.
- **Honesty:** every Mind-originated text carries ⚡. If the Mind is unreachable, he says once, "I can't reach my thinking cap right now — I'll stick to the rulebook," and carries on.
- **Budget:** a per-project token/money cap with a meter, because the budget is real.

## 7.9 Relationship memory (tamagotchi lite)

Local, small and opt-out: days since last session, streaks, completed projects, favourite materials, and how often the user overrides each rule (rules overridden 3+ times get demoted to the panel for this user, which is how Sancho learns not to nag). He references these lightly: "Welcome back — it's been 9 days. The drawer fronts are waiting."

## 7.10 Acceptance

| Criterion | Measured by |
|---|---|
| Golden projects (dresser, package, backpack, fold-bed, battery cover): all seeded faults flagged, 0 false flags | `cargo xtask corpus --assist` |
| Rules produce ≥ 60% of findings with the Mind on | Finding source counts on the golden set |
| Mute costs 0 fps and 0 MB of animation | Potato bench |
| Session interview median completion time < 60 s | Usability test (n ≥ 5) |
| Testers leave Sancho On (Silence counts as On) | ≥ 70% at the end of a 2-week trial |
