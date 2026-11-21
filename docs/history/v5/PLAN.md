# MakerCAD v5 — the plan

The plan of record is the chat artifact:

    C:/Users/<you>/ATLAS-api-harness/runs/crew/pa/chats/ses_e2eeb572cffe1dXl1dwoPYkJy0/plan.md

This file exists so the code tree explains itself without the chat. When the plan
changes, change the artifact and bump the date here.

- **2026-11-12** — v5 plan written: five reference programs reviewed (solidcraft,
  cadcraft, popupcad, packbox, oriedita), eight TRIZ contradictions resolved with
  measurable proofs, 14 crates plus four load-on-demand modules, milestones M0–M5.
- **2026-11-12** — PM answers recorded in plan §11: the helper is **Sancho**; the log
  starts at project creation and ends with an optional AI rewrite in the user's own
  style; **lamina** is adopted as concepts only (AGPL-3.0); the install-time capability
  probe chooses the performance profile.

## Reading order for a new contributor

1. `docs/STATUS.md` — what is proven, and the command that proves it.
2. `docs/DECISIONS.md` — what was decided and what would reverse it.
3. plan §3 — the layers, the OCCT wall, the renderer.
4. plan §4 and §11 — the log and Sancho, which are the product.
5. `docs/ATTRIBUTION.md` — whose ideas these are, and under what licence.

## The rules this tree is built to keep

- A crate depends only on lower layers; `makercad layers` fails the build otherwise.
- OpenCASCADE is named in exactly one crate. No OCCT type crosses into Rust.
- Every user action is a command with a schema, so the UI, the CLI, the MCP server and
  the project log cannot drift apart.
- A claim without a command and a measured result is not a claim.

## The unified design

`docs/DESIGN.md` is the single document to read. Fable (claude-fable-5-2) unified the plan of
record and these docs on 2026-11-12; Gordon reviewed the result and filed it. Where DESIGN.md and
an older document disagree, DESIGN.md wins and the older one is corrected.

Open inconsistencies Fable found are listed in
`C:/Users/<you>/ATLAS-api-harness/runs/crew/special_teams/UNIFY_NOTES.md`.
