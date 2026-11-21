# Review: Crew campaign mk-core-2 (wave 2, units version 2)

Reviewed by Claude on 2026-11-19. Wave 2 was written because the review of wave 1
([mk-core-1-review.md](mk-core-1-review.md)) found two gaps in Claude's units contract: a
24-digit number read as 0 mm, and `5'-6"` was rejected.

## What the Crew was asked for

A rework of the accepted `units.rs` against contract **version 2**: a 15-digit limit on whole
numbers and each side of a fraction (`NotANumber`, never 0 and never `DivideByZero`); one `-`
allowed between a feet quantity and the inches quantity; and the one clippy warning fixed. The
member got version 1 and its tests as reference and was told to change only what version 2
needs. Five new hidden tests, checked first against Claude's reference (22/22) and against
version 1 (fails exactly the two new-behaviour tests).

## What happened

| Task | Member | Result | Requests | Time |
|---|---|---|---|---|
| units (rework) | Gordon | accepted, first try | 1 | 4 min |
| units tests | Deepak | accepted, first try (his first MakerCAD task) | 1 | 2.5 min |

**2 of 2 accepted, 2 requests, 59,992 tokens, 6.6 member-minutes.** The keeper queued it at
20:33 and it finished at 20:40.

## Checks run in this review

| Check | Result |
|---|---|
| The diff (`git diff --no-index` v1 → v2) | +42 / −11. A `digit_run_too_long` scan before parsing; the dash is consumed only after a feet quantity; `checked_div` in `format_inches`. Also some behaviour-neutral tidying (inlined format strings, `strip_suffix(..).unwrap_or(..)`), slightly beyond "only what v2 needs" but harmless |
| `chars[pos]` in the new dash check cannot go out of range | the line before returns when `pos >= chars.len()` |
| Hidden tests (22) | 22 passed |
| Deepak's tests (kept all of version 1's, added version 2's) | 24 passed |
| Probe of the gaps | the 24-digit number, `24-digit/2` and `1/24-digit` are all `NotANumber`; `5'-6"` and `5 ft-6 in` are 1676.4 mm |
| `cargo clippy --all-targets` | clean |
| Do Deepak's tests test anything? `rs_mutants.py --max 40` | **37/40** planted defects caught (version 1 with Qwen's tests: 30/40) |
| Landed unchanged | `Get-FileHash` outbox vs `crates/mk-units/src/lib.rs`: identical |

## What landed

`crates/mk-units/src/lib.rs` is Gordon's version 2; `tests/crew.rs` is Deepak's file;
`tests/contract.rs` is the 22 contract tests, public now that version 2 is final.
