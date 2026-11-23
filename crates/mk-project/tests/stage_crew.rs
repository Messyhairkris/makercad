//! Tests for mk_project::stage written by Hoss (the Crew, campaign mk-core-27) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_project::stage` based on the contract.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_project::stage::{
    class_for_size, grid_for, major_for, next_larger, next_smaller, row, snap_for, SanchoStance,
    ScaleClass, Stage, Units, MACRO_GRID_IN, MM_PER_INCH,
};

const TOL: f64 = 1e-9;

/// A1: `row(class)` returns a row whose `class` field matches the argument.
#[test]
fn a1_row_returns_correct_class() {
    for &c in &[
        ScaleClass::Room,
        ScaleClass::Furniture,
        ScaleClass::Body,
        ScaleClass::Bench,
        ScaleClass::Macro,
    ] {
        let r = row(c);
        assert_eq!(r.class, c);
    }
}

/// A2: Typical size fields match the contract table.
#[test]
fn a2_typical_size_fields() {
    let r = row(ScaleClass::Room);
    assert_eq!(r.size_min_mm, 500.0);
    assert_eq!(r.size_max_mm, 10000.0);
    assert_eq!(r.size_text, "0.5-10 m");

    let r = row(ScaleClass::Furniture);
    assert_eq!(r.size_min_mm, 300.0);
    assert_eq!(r.size_max_mm, 2500.0);
    assert_eq!(r.size_text, "0.3-2.5 m");

    let r = row(ScaleClass::Body);
    assert_eq!(r.size_min_mm, 100.0);
    assert_eq!(r.size_max_mm, 1000.0);
    assert_eq!(r.size_text, "0.1-1 m, worn or carried");

    let r = row(ScaleClass::Bench);
    assert_eq!(r.size_min_mm, 10.0);
    assert_eq!(r.size_max_mm, 500.0);
    assert_eq!(r.size_text, "10-500 mm");

    let r = row(ScaleClass::Macro);
    assert_eq!(r.size_min_mm, 0.0);
    assert_eq!(r.size_max_mm, 10.0);
    assert_eq!(r.size_text, "< 10 mm");
}

/// A3: Stage and stage_text fields match the contract table.
#[test]
fn a3_stage_fields() {
    let r = row(ScaleClass::Room);
    assert_eq!(r.stage, Stage::Space);
    assert_eq!(
        r.stage_text,
        "The Space (room shell or a default room box 3.6 x 3 x 2.5 m)"
    );

    let r = row(ScaleClass::Furniture);
    assert_eq!(r.stage, Stage::FloorPlane);
    assert_eq!(r.stage_text, "A floor plane with a rug-sized patch");

    let r = row(ScaleClass::Body);
    assert_eq!(r.stage, Stage::Mannequin);
    assert_eq!(r.stage_text, "A mannequin or an accessory layout mat");

    let r = row(ScaleClass::Bench);
    assert_eq!(r.stage, Stage::Workbench);
    assert_eq!(
        r.stage_text,
        "A workbench top (cutting mat texture, 600 x 450 mm)"
    );

    let r = row(ScaleClass::Macro);
    assert_eq!(r.stage, Stage::CuttingMat);
    assert_eq!(r.stage_text, "A cutting mat with mm grid");
}

/// A4: Sancho and sancho_text fields match the contract table.
#[test]
fn a4_sancho_fields() {
    let r = row(ScaleClass::Room);
    assert_eq!(r.sancho, SanchoStance::Standing);
    assert_eq!(
        r.sancho_text,
        "True scale, 1.68 m (5'6\"), standing in the room"
    );

    let r = row(ScaleClass::Furniture);
    assert_eq!(r.sancho, SanchoStance::Beside);
    assert_eq!(r.sancho_text, "True scale beside the piece");

    let r = row(ScaleClass::Body);
    assert_eq!(r.sancho, SanchoStance::Wearing);
    assert_eq!(r.sancho_text, "True scale; can wear the design");

    let r = row(ScaleClass::Bench);
    assert_eq!(r.sancho, SanchoStance::FloorBeside);
    assert_eq!(
        r.sancho_text,
        "True size on the floor beside a small part on the workbench"
    );

    let r = row(ScaleClass::Macro);
    assert_eq!(r.sancho, SanchoStance::GiantEye);
    assert_eq!(r.sancho_text, "A giant eye with a magnifying glass");
}

/// A5: Grid, snap and major numeric fields match the contract table.
#[test]
fn a5_grid_snap_major_fields() {
    let r = row(ScaleClass::Room);
    assert_eq!(r.grid_mm, 100.0);
    assert_eq!(r.snap_mm, 100.0);
    assert_eq!(r.major_mm, 1000.0);
    assert_eq!(r.grid_in, 1.0);
    assert_eq!(r.snap_in, 1.0);
    assert_eq!(r.major_in, 12.0);

    let r = row(ScaleClass::Furniture);
    assert_eq!(r.grid_mm, 10.0);
    assert_eq!(r.snap_mm, 10.0);
    assert_eq!(r.major_mm, 100.0);
    assert_eq!(r.grid_in, 0.25);
    assert_eq!(r.snap_in, 0.25);
    assert_eq!(r.major_in, 2.5);

    let r = row(ScaleClass::Body);
    assert_eq!(r.grid_mm, 5.0);
    assert_eq!(r.snap_mm, 5.0);
    assert_eq!(r.major_mm, 50.0);
    assert_eq!(r.grid_in, 0.125);
    assert_eq!(r.snap_in, 0.125);
    assert_eq!(r.major_in, 1.25);

    let r = row(ScaleClass::Bench);
    assert_eq!(r.grid_mm, 1.0);
    assert_eq!(r.snap_mm, 1.0);
    assert_eq!(r.major_mm, 10.0);
    assert_eq!(r.grid_in, 0.03125);
    assert_eq!(r.snap_in, 0.03125);
    assert_eq!(r.major_in, 0.3125);

    let r = row(ScaleClass::Macro);
    assert_eq!(r.grid_mm, 0.1);
    assert_eq!(r.snap_mm, 0.1);
    assert_eq!(r.major_mm, 1.0);
    assert_eq!(r.grid_in, MACRO_GRID_IN);
    assert_eq!(r.snap_in, MACRO_GRID_IN);
    assert_eq!(r.major_in, MACRO_GRID_IN * 10.0);
}

/// A6: `grid_for` returns millimetres for both metric and imperial units.
#[test]
fn a6_grid_for_returns_mm() {
    // Metric cases - direct values.
    assert_eq!(grid_for(ScaleClass::Furniture, Units::Metric), 10.0);
    assert_eq!(grid_for(ScaleClass::Body, Units::Metric), 5.0);

    // Imperial cases - convert via MM_PER_INCH.
    let expected = row(ScaleClass::Body).grid_in * MM_PER_INCH;
    assert!((grid_for(ScaleClass::Body, Units::Imperial) - expected).abs() < TOL);
    let expected = row(ScaleClass::Room).grid_in * MM_PER_INCH;
    assert!((grid_for(ScaleClass::Room, Units::Imperial) - expected).abs() < TOL);
}

/// A7: `snap_for` rounds to the nearest step, ties away from zero.
#[test]
fn a7_snap_for_rounds() {
    // Room, metric step = 100 mm.
    assert!((snap_for(ScaleClass::Room, Units::Metric, 149.0) - 100.0).abs() < TOL);
    assert!((snap_for(ScaleClass::Room, Units::Metric, 150.0) - 200.0).abs() < TOL);
    assert!((snap_for(ScaleClass::Room, Units::Metric, -150.0) + 200.0).abs() < TOL);
    // Bench, imperial step = 0.03125 in * 25.4 = 0.79375 mm.
    let step = row(ScaleClass::Bench).snap_in * MM_PER_INCH;
    let val = 1.0; // mm
    let expected = (val / step).round() * step;
    assert!((snap_for(ScaleClass::Bench, Units::Imperial, val) - expected).abs() < TOL);
}

/// A8: `snap_for` handles NaN, infinities and zero specially.
#[test]
fn a8_snap_for_special_values() {
    let nan = snap_for(ScaleClass::Room, Units::Metric, f64::NAN);
    assert!(nan.is_nan());

    let pos_inf = snap_for(ScaleClass::Room, Units::Metric, f64::INFINITY);
    assert!(pos_inf.is_infinite() && pos_inf.is_sign_positive());

    let neg_inf = snap_for(ScaleClass::Room, Units::Metric, f64::NEG_INFINITY);
    assert!(neg_inf.is_infinite() && neg_inf.is_sign_negative());

    assert_eq!(snap_for(ScaleClass::Room, Units::Metric, 0.0), 0.0);
    assert_eq!(snap_for(ScaleClass::Room, Units::Metric, -0.0), -0.0);
}

/// A9: `class_for_size` rejects NaN, infinities, zero and negative values.
#[test]
fn a9_class_for_size_rejects_invalid() {
    assert_eq!(class_for_size(f64::NAN), None);
    assert_eq!(class_for_size(f64::INFINITY), None);
    assert_eq!(class_for_size(f64::NEG_INFINITY), None);
    assert_eq!(class_for_size(0.0), None);
    assert_eq!(class_for_size(-5.0), None);
}

/// A10: `class_for_size` ladder edges.
#[test]
fn a10_class_for_size_ladder_edges() {
    assert_eq!(class_for_size(9.9), Some(ScaleClass::Macro));
    assert_eq!(class_for_size(10.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.1), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.0), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.1), Some(ScaleClass::Room));
}

/// A11: `class_for_size` never returns Body.
#[test]
fn a11_body_never_suggested() {
    // Values that fall inside Body's typical range but should map elsewhere.
    assert_ne!(class_for_size(200.0), Some(ScaleClass::Body));
    assert_ne!(class_for_size(100.0), Some(ScaleClass::Body));
}

/// A12: `next_larger` follows the defined order.
#[test]
fn a12_next_larger_order() {
    assert_eq!(next_larger(ScaleClass::Macro), Some(ScaleClass::Bench));
    assert_eq!(next_larger(ScaleClass::Bench), Some(ScaleClass::Furniture));
    assert_eq!(next_larger(ScaleClass::Furniture), Some(ScaleClass::Room));
    assert_eq!(next_larger(ScaleClass::Room), None);
    assert_eq!(next_larger(ScaleClass::Body), None);
}

/// A13: `next_smaller` follows the reverse order.
#[test]
fn a13_next_smaller_order() {
    assert_eq!(next_smaller(ScaleClass::Room), Some(ScaleClass::Furniture));
    assert_eq!(next_smaller(ScaleClass::Furniture), Some(ScaleClass::Bench));
    assert_eq!(next_smaller(ScaleClass::Bench), Some(ScaleClass::Macro));
    assert_eq!(next_smaller(ScaleClass::Macro), None);
    assert_eq!(next_smaller(ScaleClass::Body), None);
}

/// A14: Body has no neighbours.
#[test]
fn a14_body_outside_order() {
    assert_eq!(next_larger(ScaleClass::Body), None);
    assert_eq!(next_smaller(ScaleClass::Body), None);
}

/// A15: No function panics on any input (NaN, infinities, zero, negative, huge, tiny).
#[test]
fn a15_no_panics_on_extreme_inputs() {
    // Snap_for with a variety of inputs.
    let _ = snap_for(ScaleClass::Room, Units::Metric, f64::NAN);
    let _ = snap_for(ScaleClass::Room, Units::Metric, f64::INFINITY);
    let _ = snap_for(ScaleClass::Room, Units::Metric, f64::NEG_INFINITY);
    let _ = snap_for(ScaleClass::Room, Units::Metric, 0.0);
    let _ = snap_for(ScaleClass::Room, Units::Metric, -0.0);
    let _ = snap_for(ScaleClass::Room, Units::Metric, -1e-300);
    let _ = snap_for(ScaleClass::Room, Units::Metric, 1e300);

    // grid_for and major_for with extreme inputs (should just return the step values).
    let _ = grid_for(ScaleClass::Bench, Units::Metric);
    let _ = major_for(ScaleClass::Bench, Units::Metric);
    let _ = grid_for(ScaleClass::Bench, Units::Imperial);
    let _ = major_for(ScaleClass::Bench, Units::Imperial);

    // class_for_size with extreme inputs already covered above.
    let _ = class_for_size(f64::NAN);
    let _ = class_for_size(f64::INFINITY);
    let _ = class_for_size(-1e-300);
    let _ = class_for_size(1e300);
}

/// Worked example W1
#[test]
fn worked_w1() {
    assert_eq!(row(ScaleClass::Room).stage, Stage::Space);
}

/// Worked example W2
#[test]
fn worked_w2() {
    assert_eq!(row(ScaleClass::Bench).stage, Stage::Workbench);
}

/// Worked example W3
#[test]
fn worked_w3() {
    assert_eq!(row(ScaleClass::Bench).sancho, SanchoStance::FloorBeside);
}

/// Worked example W4
#[test]
fn worked_w4() {
    assert_eq!(row(ScaleClass::Room).size_text, "0.5-10 m");
}

/// Worked example W5
#[test]
fn worked_w5() {
    assert_eq!(row(ScaleClass::Furniture).size_max_mm, 2500.0);
}

/// Worked example W6
#[test]
fn worked_w6() {
    assert_eq!(grid_for(ScaleClass::Furniture, Units::Metric), 10.0);
}

/// Worked example W7
#[test]
fn worked_w7() {
    let expected = row(ScaleClass::Body).grid_in * MM_PER_INCH;
    assert!((grid_for(ScaleClass::Body, Units::Imperial) - expected).abs() < TOL);
}

/// Worked example W8
#[test]
fn worked_w8() {
    let expected = row(ScaleClass::Room).grid_in * MM_PER_INCH;
    assert!((grid_for(ScaleClass::Room, Units::Imperial) - expected).abs() < TOL);
}

/// Worked example W9
#[test]
fn worked_w9() {
    assert_eq!(major_for(ScaleClass::Room, Units::Metric), 1000.0);
}

/// Worked example W10
#[test]
fn worked_w10() {
    assert_eq!(major_for(ScaleClass::Bench, Units::Metric), 10.0);
}

/// Worked example W11
#[test]
fn worked_w11() {
    assert!((snap_for(ScaleClass::Room, Units::Metric, 149.0) - 100.0).abs() < TOL);
}

/// Worked example W12
#[test]
fn worked_w12() {
    assert!((snap_for(ScaleClass::Room, Units::Metric, 150.0) - 200.0).abs() < TOL);
}

/// Worked example W13
#[test]
fn worked_w13() {
    assert!((snap_for(ScaleClass::Room, Units::Metric, -150.0) + 200.0).abs() < TOL);
}

/// Worked example W14
#[test]
fn worked_w14() {
    let step = row(ScaleClass::Bench).snap_in * MM_PER_INCH;
    let expected = (1.0 / step).round() * step;
    assert!((snap_for(ScaleClass::Bench, Units::Imperial, 1.0) - expected).abs() < TOL);
}

/// Worked example W15
#[test]
fn worked_w15() {
    assert!(snap_for(ScaleClass::Furniture, Units::Imperial, f64::NAN).is_nan());
}

/// Worked example W16
#[test]
fn worked_w16() {
    assert_eq!(class_for_size(9.9), Some(ScaleClass::Macro));
}

/// Worked example W17
#[test]
fn worked_w17() {
    assert_eq!(class_for_size(10.0), Some(ScaleClass::Bench));
}

/// Worked example W18
#[test]
fn worked_w18() {
    assert_eq!(class_for_size(500.1), Some(ScaleClass::Furniture));
}

/// Worked example W19
#[test]
fn worked_w19() {
    assert_eq!(class_for_size(2500.1), Some(ScaleClass::Room));
}

/// Worked example W20
#[test]
fn worked_w20() {
    assert_eq!(class_for_size(0.0), None);
}

/// Worked example W21
#[test]
fn worked_w21() {
    assert_eq!(next_larger(ScaleClass::Bench), Some(ScaleClass::Furniture));
}

/// Worked example W22
#[test]
fn worked_w22() {
    assert_eq!(next_smaller(ScaleClass::Bench), Some(ScaleClass::Macro));
}

/// Worked example W23
#[test]
fn worked_w23() {
    assert_eq!(next_larger(ScaleClass::Body), None);
}

/// Worked example W24
#[test]
fn worked_w24() {
    assert_eq!(next_smaller(ScaleClass::Body), None);
}
