//! One more test for mk_assist::golden, written by Claude after mutation testing showed a gap:
//! no test gave a seed whose rule matches but whose place does not, or the other way round,
//! so a check that took "rule OR place" for "rule AND place" passed everything.

use mk_assist::golden::{check, with_faults, Seed};

fn seed(rule: &str, entity: &str) -> Seed {
    Seed {
        rule: rule.to_string(),
        entity: entity.to_string(),
    }
}

#[test]
fn a_seed_needs_its_rule_and_its_place_to_match() {
    // `with_faults(&[0])` has exactly one finding: structure.sag at carcase_top.
    let golden = with_faults(&[0]);
    let found = vec![("structure.sag".to_string(), "carcase_top".to_string())];

    for wrong in [
        seed("structure.sag", "side_left"), // the right rule, the wrong place
        seed("structure.thin_wall", "carcase_top"), // the right place, the wrong rule
    ] {
        let report = check(&golden, std::slice::from_ref(&wrong));
        assert!(report.matched.is_empty());
        assert!(report.missing == vec![wrong]);
        assert!(report.false_flags == found);
        assert!(!report.is_perfect());
    }
}
