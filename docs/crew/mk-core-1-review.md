# Review: Crew campaign mk-core-1 (wave 1)

Reviewed by Claude on 2026-11-19, before the code landed in `crates/`. Rule 8 of
[AGENTS.md](../../AGENTS.md): Crew output never lands directly; it lands through a review like
this one.

## What the Crew was asked for

Two pure-`std` Rust modules, each judged by hidden tests the members could not read:

| Module | Contract (ATLAS-api-harness) | Lands as |
|---|---|---|
| `units.rs`: lengths the way makers type them (`5' 3 1/2"`, `2x4`), millimetres inside | `crew/contracts/mk_units.md` (version 1) | `crates/mk-units/src/lib.rs` |
| `cutstock.rs`: lay parts out on the boards a maker has, with kerf, waste and offcuts | `crew/contracts/mk_cutstock.md` | `crates/mk-nest/src/lib.rs` |

## What happened

| Task | Member | Result | Requests |
|---|---|---|---|
| units | Gordon | accepted, first try (32.7 min) | 5 |
| units tests | Qwen | accepted, first try | 2 |
| cutstock | Qwen → **Gordon** | Qwen's provider returned 504 three times (not counted against him); the campaign **relayed** the task to Gordon, who was accepted first try | 2 |
| cutstock tests | Gordon | attempt 1 had a wrong expectation for the more-than-12-pieces rule; the gate said so and attempt 2 was accepted | 8 |

**4 of 4 accepted, 17 requests, 406,430 tokens, 209 member-minutes. First-try rate 3 of 4.**
The relay is the "failed work is passed to the next capable agent" rule working unattended:
nobody was watching at 19:32 when Qwen's provider gave up and Gordon picked the task up.

## Checks run in this review

| Check | Command | Result |
|---|---|---|
| Members never read the hidden tests | search every job record of the campaign for `hidden_tests` / `reference/mk_` | only the gate's own job spec mentions them; no reads |
| Files land unchanged | `Get-FileHash` outbox file vs `crates/*/src/lib.rs` | identical |
| Hidden tests pass on the landed crates | scratch crate re-exporting `mk_units`/`mk_nest`, `cargo test` | 17/17 units, 13/13 cutstock |
| The Crew's own tests pass in the repo | `cargo test --workspace` | 15 (mk-units) + 13 (mk-nest) |
| Lints | `cargo clippy --workspace --all-targets` | 1 warning: `manual_checked_ops` at `mk-units/src/lib.rs:122` |
| Do the tests test anything? | `rs_mutants.py --max 40` (one-token defects) | Crew tests: units 30/40 caught, cutstock 20/40. Hidden tests on cutstock: 24/40 |

## Findings

1. **Gaps in Claude's contract, not in Gordon's work** (`units`), confirmed by a probe:
   - `parse_length("123456789012345678901234")` returns `Ok(0.0)`: a 24-digit number silently
     becomes 0 mm (the `u64` parse fails into `unwrap_or(0)`).
   - `parse_length("1/123456789012345678901234")` says `DivideByZero`, which is misleading.
   - `5'-6"` and `5 ft-6 in`, the way plans write feet and inches, are rejected.

   The contract never mentioned either case. **Fixed by contract version 2** (15-digit limit,
   a dash between feet and inches, clippy clean) with 5 new hidden tests. My updated
   reference passes all 22; version 1 fails exactly the two new-behaviour tests. The rework
   is campaign **mk-core-2** (Gordon writes, Deepak tests), started 20:34 under the keeper.
2. **My cutstock hidden tests were weak on the exact search (rule 4).** Surviving defects
   showed that no test had the search beat first fit *with kerf*, so wrong kerf totals inside
   the search went unseen, and nothing required the search to consider the first board. Two
   tests added (to the hidden set and to `crates/mk-nest/tests/contract.rs`):
   `the_exact_search_counts_kerf_like_first_fit_does` (with a piece as long as a board) and
   `the_exact_search_may_need_the_first_board`. Both pass on Gordon's module and on my
   reference. The same 40 planted defects: **24 → 28 caught**; the 12 left are the kinds
   in findings 3 and 4.
3. Most other survivors are equivalent defects: `<=` vs `<` next to a `1e-9` tolerance,
   fallbacks like `unwrap_or(0)` that never run, and changes to the search's pruning that make
   it slower but not wrong.
4. One survivor is real but hard to reach: making the search's "same board state" check
   looser (`&&` → `||`) could skip a needed branch when two boards of the same length are open
   with different space left. No small test found yet; noted for whoever touches rule 4 next.

## What landed

- `crates/mk-units`: Gordon's `units.rs` unchanged, plus Qwen's tests as `tests/crew.rs`.
  To be replaced by the version 2 module after mk-core-2 is reviewed.
- `crates/mk-nest`: Gordon's `cutstock.rs` unchanged, plus Gordon's tests as `tests/crew.rs`
  and Claude's contract tests (the former hidden tests plus the two new ones) as
  `tests/contract.rs`. The cutstock contract is final, so its tests are public now; new rules
  will get new hidden tests first.
