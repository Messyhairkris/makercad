# Review: Crew campaign mk-core-3 (wave 3, quick-add)

Reviewed by Claude on 2026-11-19. Contract: `crew/contracts/mk_quickadd.md` (ATLAS-api-harness).

## What the Crew was asked for

`quickadd.rs`, the parser behind the Quest's "What materials do you have?" step (doc 05): one
line the way a maker says it ("15 pine 2x4s, 3 foot", "half sheet 3/4 ply", "an old
bookshelf") read into a board, a sheet or an other item, with notes the UI asks the maker to
confirm. Built on the accepted `units.rs` v2. 21 worked examples; the hidden tests (14) were
checked first against Claude's reference.

## What happened

| Task | Member | Result | Requests |
|---|---|---|---|
| quickadd | Deepak | accepted, first try | 6 |
| quickadd tests | Gordon | accepted, first try | 9 |

**2 of 2 first try, 15 requests, 365,153 tokens, 31 member-minutes.**

## Checks

| Check | Result |
|---|---|
| Read of the module | follows the rules in order; sheet before board before other; notes as exact strings; two `words[i]` index the position just found, so never out of range |
| Lands unchanged | `crates/mk-inventory/src/quickadd.rs` is byte-for-byte the outbox file; the crate re-exports `mk_units` as `crate::units`, which the module was written against |
| Contract tests | 14 → **17**: three added after the mutation run (every number word; four-digit quantities; sheet sizes 1–12 ft). Deepak's module and Claude's reference both pass 17/17 |
| Gordon's tests | 21 passed |
| Do Gordon's tests test anything? `rs_mutants.py --max 40` | 28/40 caught. The survivors were number words past "two", the four-digit limit, the 1–12 sheet-size bounds, and one equivalent change (an initial `qty` always overwritten); the three new contract tests cover the first three |
| `cargo clippy --all-targets` | clean |
