//! Contract tests for mk_project::stage, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_stage.md. They were the hidden tests of campaign mk-core-27.

//! Hidden tests for `mk_project::stage` (doc 05 section 5.5).
//!
//! One test per contract rule (A1-A15) and one per row of the Worked examples table.
//! Floats are compared with an absolute tolerance of 1e-9.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_project::stage::{
    class_for_size, grid_for, major_for, next_larger, next_smaller, row, snap_for, SanchoStance,
    ScaleClass, Stage, Units,
};

const ALL: [ScaleClass; 5] = [
    ScaleClass::Room,
    ScaleClass::Furniture,
    ScaleClass::Body,
    ScaleClass::Bench,
    ScaleClass::Macro,
];

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-9
}

#[test]
fn a1_row_returns_the_class() {
    for &class in &ALL {
        assert!(row(class).class == class);
    }
    assert!(row(ScaleClass::Room).class == ScaleClass::Room);
    assert!(row(ScaleClass::Macro).class == ScaleClass::Macro);
}

#[test]
fn a2_typical_size_and_text() {
    let room = row(ScaleClass::Room);
    assert!(close(room.size_min_mm, 500.0));
    assert!(close(room.size_max_mm, 10_000.0));
    assert!(room.size_text == "0.5-10 m");

    let furniture = row(ScaleClass::Furniture);
    assert!(close(furniture.size_min_mm, 300.0));
    assert!(close(furniture.size_max_mm, 2500.0));
    assert!(furniture.size_text == "0.3-2.5 m");

    let body = row(ScaleClass::Body);
    assert!(close(body.size_min_mm, 100.0));
    assert!(close(body.size_max_mm, 1000.0));
    assert!(body.size_text == "0.1-1 m, worn or carried");

    let bench = row(ScaleClass::Bench);
    assert!(close(bench.size_min_mm, 10.0));
    assert!(close(bench.size_max_mm, 500.0));
    assert!(bench.size_text == "10-500 mm");

    let macro_row = row(ScaleClass::Macro);
    assert!(close(macro_row.size_min_mm, 0.0));
    assert!(close(macro_row.size_max_mm, 10.0));
    assert!(macro_row.size_text == "< 10 mm");
}

#[test]
fn a3_stage_per_class() {
    assert!(row(ScaleClass::Room).stage == Stage::Space);
    assert!(
        row(ScaleClass::Room).stage_text
            == "The Space (room shell or a default room box 3.6 x 3 x 2.5 m)"
    );
    assert!(row(ScaleClass::Furniture).stage == Stage::FloorPlane);
    assert!(row(ScaleClass::Furniture).stage_text == "A floor plane with a rug-sized patch");
    assert!(row(ScaleClass::Body).stage == Stage::Mannequin);
    assert!(row(ScaleClass::Body).stage_text == "A mannequin or an accessory layout mat");
    assert!(row(ScaleClass::Bench).stage == Stage::Workbench);
    assert!(
        row(ScaleClass::Bench).stage_text == "A workbench top (cutting mat texture, 600 x 450 mm)"
    );
    assert!(row(ScaleClass::Macro).stage == Stage::CuttingMat);
    assert!(row(ScaleClass::Macro).stage_text == "A cutting mat with mm grid");
}

#[test]
fn a4_sancho_per_class() {
    assert!(row(ScaleClass::Room).sancho == SanchoStance::Standing);
    assert!(
        row(ScaleClass::Room).sancho_text == "True scale, 1.68 m (5'6\"), standing in the room"
    );
    assert!(row(ScaleClass::Furniture).sancho == SanchoStance::Beside);
    assert!(row(ScaleClass::Furniture).sancho_text == "True scale beside the piece");
    assert!(row(ScaleClass::Body).sancho == SanchoStance::Wearing);
    assert!(row(ScaleClass::Body).sancho_text == "True scale; can wear the design");
    assert!(row(ScaleClass::Bench).sancho == SanchoStance::FloorBeside);
    assert!(
        row(ScaleClass::Bench).sancho_text
            == "True size on the floor beside a small part on the workbench"
    );
    assert!(row(ScaleClass::Macro).sancho == SanchoStance::GiantEye);
    assert!(row(ScaleClass::Macro).sancho_text == "A giant eye with a magnifying glass");
}

#[test]
fn a5_grid_snap_major_table() {
    let room = row(ScaleClass::Room);
    assert!(close(room.grid_mm, 100.0) && close(room.snap_mm, 100.0));
    assert!(close(room.major_mm, 1000.0) && close(room.major_in, 12.0));
    assert!(close(room.grid_in, 1.0) && close(room.snap_in, 1.0));

    let furniture = row(ScaleClass::Furniture);
    assert!(close(furniture.grid_mm, 10.0) && close(furniture.major_mm, 100.0));
    assert!(close(furniture.grid_in, 0.25) && close(furniture.major_in, 2.5));

    let body = row(ScaleClass::Body);
    assert!(close(body.grid_mm, 5.0) && close(body.major_mm, 50.0));
    assert!(close(body.grid_in, 0.125) && close(body.major_in, 1.25));

    let bench = row(ScaleClass::Bench);
    assert!(close(bench.grid_mm, 1.0) && close(bench.major_mm, 10.0));
    assert!(close(bench.grid_in, 0.03125) && close(bench.major_in, 0.3125));

    let macro_row = row(ScaleClass::Macro);
    assert!(close(macro_row.grid_mm, 0.1) && close(macro_row.major_mm, 1.0));
    assert!(close(macro_row.grid_in, 0.00390625));
    assert!(close(macro_row.snap_in, 0.00390625));
    assert!(close(macro_row.major_in, 0.0390625));
}

#[test]
fn a6_accessors_return_millimetres() {
    assert!(close(grid_for(ScaleClass::Room, Units::Metric), 100.0));
    assert!(close(grid_for(ScaleClass::Room, Units::Imperial), 25.4));
    assert!(close(grid_for(ScaleClass::Body, Units::Imperial), 3.175));
    assert!(close(major_for(ScaleClass::Bench, Units::Metric), 10.0));
    assert!(close(
        major_for(ScaleClass::Furniture, Units::Imperial),
        63.5
    ));
    assert!(close(major_for(ScaleClass::Room, Units::Imperial), 304.8));
}

#[test]
fn a7_snap_rounds_to_the_increment() {
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, 149.0),
        100.0
    ));
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, 151.0),
        200.0
    ));
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, 150.0),
        200.0
    ));
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, -150.0),
        -200.0
    ));
    assert!(close(snap_for(ScaleClass::Room, Units::Metric, 0.0), 0.0));
    assert!(close(
        snap_for(ScaleClass::Bench, Units::Imperial, 1.0),
        0.79375
    ));
    assert!(close(
        snap_for(ScaleClass::Body, Units::Imperial, 2.0),
        3.175
    ));
}

#[test]
fn a8_nan_and_infinity() {
    assert!(snap_for(ScaleClass::Furniture, Units::Imperial, f64::NAN).is_nan());
    let up = snap_for(ScaleClass::Room, Units::Metric, f64::INFINITY);
    assert!(up.is_infinite() && up.is_sign_positive());
    let down = snap_for(ScaleClass::Room, Units::Metric, f64::NEG_INFINITY);
    assert!(down.is_infinite() && down.is_sign_negative());
}

#[test]
fn a9_class_for_size_rejects_bad_lengths() {
    assert!(class_for_size(f64::NAN).is_none());
    assert!(class_for_size(f64::INFINITY).is_none());
    assert!(class_for_size(f64::NEG_INFINITY).is_none());
    assert!(class_for_size(0.0).is_none());
    assert!(class_for_size(-0.0).is_none());
    assert!(class_for_size(-1.0).is_none());
    assert!(class_for_size(-1.0e9).is_none());
}

#[test]
fn a10_class_for_size_ladder() {
    assert!(class_for_size(0.1) == Some(ScaleClass::Macro));
    assert!(class_for_size(9.999) == Some(ScaleClass::Macro));
    assert!(class_for_size(10.0) == Some(ScaleClass::Bench));
    assert!(class_for_size(499.0) == Some(ScaleClass::Bench));
    assert!(class_for_size(500.0) == Some(ScaleClass::Bench));
    assert!(class_for_size(500.1) == Some(ScaleClass::Furniture));
    assert!(class_for_size(2499.0) == Some(ScaleClass::Furniture));
    assert!(class_for_size(2500.0) == Some(ScaleClass::Furniture));
    assert!(class_for_size(2500.1) == Some(ScaleClass::Room));
    assert!(class_for_size(1.0e7) == Some(ScaleClass::Room));
}

#[test]
fn a11_body_is_never_suggested() {
    let sizes = [
        0.001, 1.0, 9.9, 10.0, 100.0, 499.0, 500.0, 1000.0, 2500.0, 2500.1, 1.0e9,
    ];
    for &size in &sizes {
        assert!(class_for_size(size) != Some(ScaleClass::Body));
    }
}

#[test]
fn a12_next_larger() {
    assert!(next_larger(ScaleClass::Macro) == Some(ScaleClass::Bench));
    assert!(next_larger(ScaleClass::Bench) == Some(ScaleClass::Furniture));
    assert!(next_larger(ScaleClass::Furniture) == Some(ScaleClass::Room));
    assert!(next_larger(ScaleClass::Room).is_none());
}

#[test]
fn a13_next_smaller() {
    assert!(next_smaller(ScaleClass::Room) == Some(ScaleClass::Furniture));
    assert!(next_smaller(ScaleClass::Furniture) == Some(ScaleClass::Bench));
    assert!(next_smaller(ScaleClass::Bench) == Some(ScaleClass::Macro));
    assert!(next_smaller(ScaleClass::Macro).is_none());
}

#[test]
fn a14_body_is_outside_the_stage_order() {
    assert!(next_larger(ScaleClass::Body).is_none());
    assert!(next_smaller(ScaleClass::Body).is_none());
}

#[test]
fn a15_no_input_panics() {
    let values = [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.0,
        -0.0,
        1.0e-300,
        1.0e300,
        -1.0,
    ];
    for &class in &ALL {
        let _ = row(class);
        let _ = next_larger(class);
        let _ = next_smaller(class);
        assert!(!row(class).size_text.is_empty());
        for &units in &[Units::Metric, Units::Imperial] {
            let _ = grid_for(class, units);
            let _ = major_for(class, units);
            for &value in &values {
                let _ = snap_for(class, units, value);
            }
        }
    }
    for &value in &values {
        let _ = class_for_size(value);
    }
}

#[test]
fn worked_w1_room_stage() {
    assert!(row(ScaleClass::Room).stage == Stage::Space);
}

#[test]
fn worked_w2_bench_stage() {
    assert!(row(ScaleClass::Bench).stage == Stage::Workbench);
}

#[test]
fn worked_w3_bench_sancho() {
    assert!(row(ScaleClass::Bench).sancho == SanchoStance::FloorBeside);
}

#[test]
fn worked_w4_room_size_text() {
    assert!(row(ScaleClass::Room).size_text == "0.5-10 m");
}

#[test]
fn worked_w5_furniture_size_max() {
    assert!(close(row(ScaleClass::Furniture).size_max_mm, 2500.0));
}

#[test]
fn worked_w6_furniture_metric_grid() {
    assert!(close(grid_for(ScaleClass::Furniture, Units::Metric), 10.0));
}

#[test]
fn worked_w7_body_imperial_grid() {
    assert!(close(grid_for(ScaleClass::Body, Units::Imperial), 3.175));
}

#[test]
fn worked_w8_room_imperial_grid() {
    assert!(close(grid_for(ScaleClass::Room, Units::Imperial), 25.4));
}

#[test]
fn worked_w9_room_metric_major() {
    assert!(close(major_for(ScaleClass::Room, Units::Metric), 1000.0));
}

#[test]
fn worked_w10_bench_metric_major() {
    assert!(close(major_for(ScaleClass::Bench, Units::Metric), 10.0));
}

#[test]
fn worked_w11_snap_149() {
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, 149.0),
        100.0
    ));
}

#[test]
fn worked_w12_snap_150_ties_away() {
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, 150.0),
        200.0
    ));
}

#[test]
fn worked_w13_snap_negative_150() {
    assert!(close(
        snap_for(ScaleClass::Room, Units::Metric, -150.0),
        -200.0
    ));
}

#[test]
fn worked_w14_bench_imperial_snap() {
    assert!(close(
        snap_for(ScaleClass::Bench, Units::Imperial, 1.0),
        0.79375
    ));
}

#[test]
fn worked_w15_nan_snap() {
    assert!(snap_for(ScaleClass::Furniture, Units::Imperial, f64::NAN).is_nan());
}

#[test]
fn worked_w16_9_9_is_macro() {
    assert!(class_for_size(9.9) == Some(ScaleClass::Macro));
}

#[test]
fn worked_w17_10_is_bench() {
    assert!(class_for_size(10.0) == Some(ScaleClass::Bench));
}

#[test]
fn worked_w18_500_1_is_furniture() {
    assert!(class_for_size(500.1) == Some(ScaleClass::Furniture));
}

#[test]
fn worked_w19_2500_1_is_room() {
    assert!(class_for_size(2500.1) == Some(ScaleClass::Room));
}

#[test]
fn worked_w20_zero_is_none() {
    assert!(class_for_size(0.0).is_none());
}

#[test]
fn worked_w21_next_larger_bench() {
    assert!(next_larger(ScaleClass::Bench) == Some(ScaleClass::Furniture));
}

#[test]
fn worked_w22_next_smaller_bench() {
    assert!(next_smaller(ScaleClass::Bench) == Some(ScaleClass::Macro));
}

#[test]
fn worked_w23_next_larger_body() {
    assert!(next_larger(ScaleClass::Body).is_none());
}

#[test]
fn worked_w24_next_smaller_body() {
    assert!(next_smaller(ScaleClass::Body).is_none());
}
