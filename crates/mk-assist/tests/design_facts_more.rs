//! One more test for mk_assist::design_facts, written by Claude after mutation testing showed a
//! gap: no test put a small gap between two boxes with the first one on the high side, so a
//! tolerance that only worked in one direction passed everything.

use mk_assist::design_facts::{touching, BoxPart, CONTACT_TOLERANCE_MM};

fn cube(id: &str, min: [f64; 3]) -> BoxPart {
    BoxPart {
        id: id.to_string(),
        material: "wood.softwood.pine".to_string(),
        min,
        max: [min[0] + 10.0, min[1] + 10.0, min[2] + 10.0],
    }
}

#[test]
fn a_gap_up_to_the_tolerance_touches_whichever_box_is_on_which_side_and_along_every_axis() {
    for axis in 0..3 {
        let low = cube("low", [0.0, 0.0, 0.0]);
        for (gap, expected) in [
            (0.0, true),
            (CONTACT_TOLERANCE_MM * 0.9, true),
            (CONTACT_TOLERANCE_MM, true),
            (CONTACT_TOLERANCE_MM * 1.1, false),
        ] {
            let mut at = [0.0; 3];
            at[axis] = 10.0 + gap;
            let high = cube("high", at);
            assert!(
                touching(&low, &high) == expected,
                "axis {axis}, gap {gap}: low then high"
            );
            assert!(
                touching(&high, &low) == expected,
                "axis {axis}, gap {gap}: high then low"
            );
        }
    }
}
