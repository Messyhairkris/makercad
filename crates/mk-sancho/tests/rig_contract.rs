//! Contract tests for mk_sancho::rig, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_rig.md. They were the hidden tests of campaign mk-core-14.

// The worked rows copy the contract's table, angles to six decimals (0.785398 is pi/4 as the
// table prints it); they are compared within 1e-6 on purpose.
#![allow(clippy::approx_constant)]

use mk_sancho::rig::{
    caps, clip_for, crossfade_duration, fade_weight, height_scale, look_at, point_at, walk_to,
    wrap_angle, Arm, Clip, Event, IkError, Limits, Tier, Vec3, CLIPS, CROSSFADE_MAX_S,
    CROSSFADE_MIN_S, CROSSFADE_S, DEFAULT_HEIGHT_M, MAX_HEIGHT_M, MIN_HEIGHT_M,
};

// Integration tests for Sancho's light rig math (contract: rig.rs v1). Every rule is covered on
// its own, then the contract's worked rows, then the no-panic sweeps. Rows are compared within
// 1e-6; exact identities (bone lengths, splits) within 1e-9.

const EYE: Vec3 = [0.0, 0.0, 1.6];
const ROUNDED: f64 = 1e-6;
const TIGHT: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= ROUNDED
}

fn close3(a: Vec3, b: Vec3) -> bool {
    close(a[0], b[0]) && close(a[1], b[1]) && close(a[2], b[2])
}

fn assert_close(a: f64, b: f64) {
    assert!(close(a, b), "expected {b}, got {a}");
}

fn assert_close3(a: Vec3, b: Vec3) {
    assert!(close3(a, b), "expected {b:?}, got {a:?}");
}

fn assert_tight(a: f64, b: f64) {
    assert!((a - b).abs() <= TIGHT, "expected {b} within 1e-9, got {a}");
}

fn sub3(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: Vec3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn default_limits() -> Limits {
    Limits::DEFAULT
}

// ---- L1: the look-at turn ----

#[test]
fn l1_looking_ahead_turns_nobody() {
    let look = look_at(EYE, [0.0, 2.0, 1.6], default_limits(), 0.5).expect("ahead is visible");
    assert_close(look.yaw, 0.0);
    assert_close(look.pitch, 0.0);
    assert_close(look.neck_yaw, 0.0);
    assert_close(look.neck_pitch, 0.0);
    assert_close(look.head_yaw, 0.0);
    assert_close(look.head_pitch, 0.0);
    assert!(!look.clamped);
}

#[test]
fn l1_yaw_is_positive_to_his_left() {
    let left = look_at(EYE, [-1.0, 1.0, 1.6], default_limits(), 0.5).expect("left target");
    let right = look_at(EYE, [1.0, 1.0, 1.6], default_limits(), 0.5).expect("right target");
    assert!(
        left.yaw > 0.0,
        "a target to his left must give a positive yaw"
    );
    assert!(
        right.yaw < 0.0,
        "a target to his right must give a negative yaw"
    );
    assert_close(left.yaw, -right.yaw);
}

#[test]
fn l1_pitch_is_positive_looking_up() {
    let up = look_at(EYE, [0.0, 1.0, 2.6], default_limits(), 0.5).expect("above the eyes");
    let down = look_at(EYE, [0.0, 1.0, 0.6], default_limits(), 0.5).expect("below the eyes");
    assert!(up.pitch > 0.0, "looking up must be a positive pitch");
    assert!(down.pitch < 0.0, "looking down must be a negative pitch");
}

#[test]
fn l1_yaw_is_clamped_to_the_limit() {
    let limits = default_limits();
    let behind = look_at(EYE, [-2.0, -0.5, 1.6], limits, 0.5).expect("behind him, left");
    assert_close(behind.yaw, limits.yaw_max);
    assert!(behind.clamped);
    let other = look_at(EYE, [2.0, -0.5, 1.6], limits, 0.5).expect("behind him, right");
    assert_close(other.yaw, -limits.yaw_max);
    assert!(other.clamped);
}

#[test]
fn l1_pitch_is_clamped_up_and_down_separately() {
    let limits = default_limits();
    let steep_up = look_at(EYE, [1.0, 1.0, 2.6], limits, 0.5).expect("up and right");
    assert_close(steep_up.pitch, limits.pitch_up);
    let steep_down = look_at(EYE, [0.0, 1.0, 0.6], limits, 0.5).expect("below");
    assert_close(steep_down.pitch, -limits.pitch_down);
}

#[test]
fn l1_a_target_at_the_eye_gives_none() {
    assert!(look_at(EYE, EYE, default_limits(), 0.5).is_none());
    let almost = [EYE[0] + 5e-10, EYE[1], EYE[2]];
    assert!(look_at(EYE, almost, default_limits(), 0.5).is_none());
}

#[test]
fn l1_non_finite_components_give_none() {
    let d = default_limits();
    assert!(look_at([f64::NAN, 0.0, 0.0], [0.0, 1.0, 0.0], d, 0.5).is_none());
    assert!(look_at(EYE, [0.0, f64::INFINITY, 0.0], d, 0.5).is_none());
    assert!(look_at(EYE, [0.0, 0.0, f64::NEG_INFINITY], d, 0.5).is_none());
}

#[test]
fn l1_a_negative_or_bad_limit_counts_as_zero() {
    let limits = Limits {
        yaw_max: -1.0,
        pitch_up: f64::NAN,
        pitch_down: f64::INFINITY,
    };
    let look = look_at(EYE, [-1.0, 1.0, 1.0], limits, 0.5).expect("target");
    assert_close(look.yaw, 0.0);
    assert_close(look.pitch, 0.0);
    assert!(look.clamped);
}

// ---- L2: the neck and head split ----

#[test]
fn l2_an_even_share_splits_the_turn_in_half() {
    let look = look_at(EYE, [-1.0, 1.0, 1.6], default_limits(), 0.5).expect("left target");
    assert_close(look.neck_yaw, look.yaw / 2.0);
    assert_close(look.head_yaw, look.yaw / 2.0);
    assert_close(look.neck_pitch, look.pitch / 2.0);
    assert_close(look.head_pitch, look.pitch / 2.0);
}

#[test]
fn l2_the_share_is_clamped_into_zero_and_one() {
    let low = look_at(EYE, [-1.0, 1.0, 1.6], default_limits(), -3.0).expect("left target");
    assert_close(low.neck_yaw, 0.0);
    assert_close(low.head_yaw, low.yaw);
    let high = look_at(EYE, [-1.0, 1.0, 1.6], default_limits(), 2.0).expect("left target");
    assert_close(high.neck_yaw, high.yaw);
    assert_close(high.head_yaw, 0.0);
}

#[test]
fn l2_a_nan_share_means_half_and_half() {
    let nan = look_at(EYE, [-1.0, 1.0, 1.6], default_limits(), f64::NAN).expect("left target");
    let half = look_at(EYE, [-1.0, 1.0, 1.6], default_limits(), 0.5).expect("left target");
    assert_tight(nan.neck_yaw, half.neck_yaw);
    assert_tight(nan.head_yaw, half.head_yaw);
}

#[test]
fn l2_neck_plus_head_is_the_whole_turn() {
    for &share in &[0.0, 0.1, 0.5, 0.9, 1.0] {
        let look = look_at(EYE, [0.7, 1.3, 2.0], default_limits(), share).expect("target");
        assert_tight(look.neck_yaw + look.head_yaw, look.yaw);
        assert_tight(look.neck_pitch + look.head_pitch, look.pitch);
    }
}

// ---- the contract's worked rows, compared as written ----

#[test]
fn worked_look_at_rows() {
    let d = default_limits();
    let ahead = look_at(EYE, [0.0, 2.0, 1.6], d, 0.5).expect("row 1");
    assert_close(ahead.yaw, 0.0);
    assert_close(ahead.pitch, 0.0);
    assert_close(ahead.neck_yaw, 0.0);
    assert_close(ahead.head_yaw, 0.0);
    assert!(!ahead.clamped);

    let left = look_at(EYE, [-1.0, 1.0, 1.6], d, 0.5).expect("row 2");
    assert_close(left.yaw, 0.785398);
    assert_close(left.pitch, 0.0);
    assert_close(left.neck_yaw, 0.392699);
    assert_close(left.head_yaw, 0.392699);
    assert!(!left.clamped);

    let up_right = look_at(EYE, [1.0, 1.0, 2.6], d, 0.5).expect("row 3");
    assert_close(up_right.yaw, -0.785398);
    assert_close(up_right.pitch, 0.523599);
    assert_close(up_right.neck_yaw, -0.392699);
    assert_close(up_right.neck_pitch, 0.261799);
    assert!(up_right.clamped);

    let behind = look_at(EYE, [-2.0, -0.5, 1.6], d, 0.5).expect("row 4");
    assert_close(behind.yaw, 1.221730);
    assert_close(behind.pitch, 0.0);
    assert!(behind.clamped);

    let down = look_at(EYE, [0.0, 1.0, 0.6], d, 0.5).expect("row 5");
    assert_close(down.yaw, 0.0);
    assert_close(down.pitch, -0.698132);
    assert!(down.clamped);
}

// ---- P1: point-at errors, in order ----

#[test]
fn p1_bad_lengths_come_first() {
    let pole = [0.0, 0.0, -1.0];
    let shoulder = [0.0, 0.0, 0.0];
    let target = [0.0, 0.4, 0.0];
    assert_eq!(
        point_at(shoulder, target, 0.0, 0.25, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(shoulder, target, 0.3, 0.0, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(shoulder, target, -0.3, 0.25, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(shoulder, target, f64::NAN, 0.25, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(shoulder, target, 0.3, f64::INFINITY, pole),
        Err(IkError::BadLength)
    );
    // A bad length outranks a degenerate target.
    assert_eq!(
        point_at(shoulder, shoulder, 0.0, 0.25, pole),
        Err(IkError::BadLength)
    );
}

#[test]
fn p1_no_direction_when_the_target_is_the_shoulder() {
    let pole = [0.0, 0.0, -1.0];
    assert_eq!(
        point_at([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    let near = [0.0, 0.0, 5e-10];
    assert_eq!(
        point_at([0.0, 0.0, 0.0], near, 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    // A bad direction outranks a pole along the arm.
    assert_eq!(
        point_at(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.3,
            0.25,
            [0.0, 0.0, -1.0]
        ),
        Err(IkError::NoDirection)
    );
}

#[test]
fn p1_pole_along_the_arm_is_refused() {
    let shoulder = [0.0, 0.0, 0.0];
    let target = [0.0, 0.4, 0.0];
    assert_eq!(
        point_at(shoulder, target, 0.3, 0.25, [0.0, 2.0, 0.0]),
        Err(IkError::PoleAlongArm)
    );
    assert_eq!(
        point_at(shoulder, target, 0.3, 0.25, [0.0, -1.0, 0.0]),
        Err(IkError::PoleAlongArm)
    );
    assert_eq!(
        point_at(shoulder, target, 0.3, 0.25, [f64::NAN, 0.0, -1.0]),
        Err(IkError::PoleAlongArm)
    );
}

fn pole_normal(shoulder: Vec3, target: Vec3, pole: Vec3) -> Vec3 {
    let delta = sub3(target, shoulder);
    let d = len3(delta);
    let u = [delta[0] / d, delta[1] / d, delta[2] / d];
    let along = dot3(pole, u);
    let p = [
        pole[0] - along * u[0],
        pole[1] - along * u[1],
        pole[2] - along * u[2],
    ];
    let n = len3(p);
    [p[0] / n, p[1] / n, p[2] / n]
}

fn assert_arm_lengths(arm: &Arm, shoulder: Vec3, upper: f64, lower: f64) {
    let to_elbow = len3(sub3(arm.elbow, shoulder));
    let elbow_to_hand = len3(sub3(arm.hand, arm.elbow));
    assert!(
        (to_elbow - upper).abs() <= TIGHT,
        "upper bone not kept: {to_elbow} vs {upper}"
    );
    assert!(
        (elbow_to_hand - lower).abs() <= TIGHT,
        "lower bone not kept: {elbow_to_hand} vs {lower}"
    );
}

// ---- P2: the two-bone solve ----

#[test]
fn p2_a_close_target_bends_the_arm() {
    let arm = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    assert_close3(arm.elbow, [0.0, 0.234375, -0.187265]);
    assert_close3(arm.hand, [0.0, 0.4, 0.0]);
    assert_close(arm.elbow_angle, 1.620817);
    assert!(arm.reached);
}

#[test]
fn p2_a_far_target_straightens_the_arm() {
    let arm = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    assert_close3(arm.elbow, [0.0, 0.3, 0.0]);
    assert_close3(arm.hand, [0.0, 0.55, 0.0]);
    assert_close(arm.elbow_angle, std::f64::consts::PI);
    assert!(!arm.reached);
}

#[test]
fn p2_a_target_inside_the_inner_reach_folds_the_arm() {
    let arm = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.02, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    assert_close3(arm.elbow, [0.0, 0.3, 0.0]);
    assert_close3(arm.hand, [0.0, 0.05, 0.0]);
    assert_close(arm.elbow_angle, 0.0);
    assert!(!arm.reached);
}

#[test]
fn p2_a_pole_offset_along_the_arm_changes_nothing() {
    let plain = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    let offset = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        0.3,
        0.25,
        [0.0, 0.5, -1.0],
    )
    .expect("solvable");
    assert_tight(offset.elbow[0], plain.elbow[0]);
    assert_tight(offset.elbow[1], plain.elbow[1]);
    assert_tight(offset.elbow[2], plain.elbow[2]);
    assert_tight(offset.elbow_angle, plain.elbow_angle);
    assert!(offset.reached);
}

#[test]
fn p2_the_elbow_bends_toward_the_pole() {
    let shoulder = [0.0, 0.0, 0.0];
    let target = [0.0, 0.4, 0.0];
    let down = point_at(shoulder, target, 0.3, 0.25, [0.0, 0.0, -1.0]).expect("solvable");
    assert!(
        down.elbow[2] < 0.0,
        "a pole below the arm must put the elbow below it"
    );
    let left = point_at(shoulder, target, 0.3, 0.25, [-1.0, 0.0, 0.0]).expect("solvable");
    assert!(
        left.elbow[0] < 0.0,
        "a pole to his left must put the elbow to his left"
    );
}

/// Targets near, inside and well beyond a 0.3 + 0.25 arm, plus both extremes of reach.
fn grid_targets() -> Vec<Vec3> {
    let mut targets = Vec::new();
    for ix in 0..7i32 {
        for iy in 0..7i32 {
            let x = -1.2 + 0.4 * (ix as f64);
            let y = -0.8 + 0.5 * (iy as f64);
            let z = 0.3 * (ix as f64) - 0.6;
            targets.push([x, y, z]);
        }
    }
    targets.push([0.0, 20.0, 0.0]);
    targets.push([-5.0, 0.0, 3.0]);
    targets.push([0.05, 0.02, 0.0]);
    targets
}

#[test]
fn p2_grid_keeps_both_bone_lengths() {
    let shoulder = [0.0, 0.0, 0.0];
    let (upper, lower) = (0.3, 0.25);
    let pole = [0.0, 0.0, -1.0];
    let targets = grid_targets();
    assert!(targets.len() >= 50);
    for target in targets {
        let arm = point_at(shoulder, target, upper, lower, pole).expect("solvable");
        assert_arm_lengths(&arm, shoulder, upper, lower);
    }
}

#[test]
fn p2_grid_puts_the_hand_on_the_target_when_reached() {
    let shoulder = [0.0, 0.0, 0.0];
    let (upper, lower) = (0.3, 0.25);
    let pole = [0.0, 0.0, -1.0];
    for target in grid_targets() {
        let arm = point_at(shoulder, target, upper, lower, pole).expect("solvable");
        if arm.reached {
            let miss = len3(sub3(arm.hand, target));
            assert!(miss <= TIGHT, "hand off target {target:?} by {miss}");
        } else {
            // Out of reach: the hand sits on the nearer end of the reach, pointing at the target.
            let reach = len3(sub3(arm.hand, shoulder));
            assert!(
                (reach - (upper + lower)).abs() <= TIGHT
                    || (reach - (upper - lower).abs()).abs() <= TIGHT,
                "hand {reach} not on the reach boundary"
            );
        }
    }
}

#[test]
fn p2_grid_keeps_the_elbow_on_the_poles_side() {
    let shoulder = [0.0, 0.0, 0.0];
    let pole = [0.0, 0.0, -1.0];
    for target in grid_targets() {
        let arm = point_at(shoulder, target, 0.3, 0.25, pole).expect("solvable");
        let n = pole_normal(shoulder, target, pole);
        let side = dot3(sub3(arm.elbow, shoulder), n);
        assert!(
            side >= -1e-12,
            "elbow fell behind the pole for {target:?}: {side}"
        );
    }
}

#[test]
fn p2_the_reached_flag_is_the_inclusive_reach_range() {
    // 0.5 and 0.25 are exact in binary, so both ends of the reach are exactly representable.
    let shoulder = [0.0, 0.0, 0.0];
    let (upper, lower) = (0.5, 0.25);
    let pole = [0.0, 0.0, -1.0];
    let reach = |d: f64| point_at(shoulder, [0.0, d, 0.0], upper, lower, pole).expect("solvable");
    assert!(
        reach(0.25).reached,
        "the inner end of reach counts as reached"
    );
    assert!(
        reach(0.75).reached,
        "the outer end of reach counts as reached"
    );
    assert!(!reach(0.24).reached);
    assert!(!reach(0.76).reached);
    assert_arm_lengths(&reach(0.75), shoulder, upper, lower);
    assert_arm_lengths(&reach(0.24), shoulder, upper, lower);
}

// ---- C1: crossfade duration ----

#[test]
fn c1_crossfade_duration_stays_inside_the_band() {
    assert_close(crossfade_duration(0.1), 0.2);
    assert_close(crossfade_duration(0.25), 0.25);
    assert_close(crossfade_duration(0.5), 0.3);
    assert_close(crossfade_duration(CROSSFADE_MIN_S), CROSSFADE_MIN_S);
    assert_close(crossfade_duration(CROSSFADE_MAX_S), CROSSFADE_MAX_S);
    assert_close(crossfade_duration(0.0), CROSSFADE_MIN_S);
    assert_close(CROSSFADE_S, 0.25);
}

#[test]
fn c1_bad_requests_use_the_default() {
    assert_close(crossfade_duration(f64::NAN), CROSSFADE_S);
    assert_close(crossfade_duration(f64::INFINITY), CROSSFADE_S);
    assert_close(crossfade_duration(f64::NEG_INFINITY), CROSSFADE_S);
}

// ---- C2: the fade weight ----

#[test]
fn c2_fade_weight_rises_from_nothing_to_all() {
    assert_close(fade_weight(0.0, 0.25), 0.0);
    assert_close(fade_weight(0.0625, 0.25), 0.15625);
    assert_close(fade_weight(0.125, 0.25), 0.5);
    assert_close(fade_weight(0.25, 0.25), 1.0);
    assert_close(fade_weight(1.0, 0.25), 1.0);
    assert_close(fade_weight(-1.0, 0.25), 0.0);
}

#[test]
fn c2_a_bad_duration_means_the_fade_is_finished() {
    assert_close(fade_weight(0.1, 0.0), 1.0);
    assert_close(fade_weight(0.1, -1.0), 1.0);
    assert_close(fade_weight(0.1, f64::NAN), 1.0);
    assert_close(fade_weight(0.1, f64::INFINITY), 1.0);
}

// Added by Claude after mutation testing: a zero duration with no time (or negative time)
// elapsed is still a finished fade, never NaN or 0 (rs_mutants line 348, `<=` -> `<`).
#[test]
fn c2_a_zero_duration_is_finished_at_any_elapsed_time() {
    assert_close(fade_weight(0.0, 0.0), 1.0);
    assert_close(fade_weight(-1.0, 0.0), 1.0);
}

#[test]
fn c2_a_nan_elapsed_starts_the_fade() {
    assert_close(fade_weight(f64::NAN, 0.25), 0.0);
}

#[test]
fn c2_fade_weight_rises_monotonically() {
    let duration = 0.25;
    let mut previous = -1.0;
    for i in 0..=100 {
        let elapsed = duration * (i as f64) / 100.0;
        let weight = fade_weight(elapsed, duration);
        assert!(
            weight >= previous,
            "weight fell at step {i}: {weight} < {previous}"
        );
        assert!(weight <= 1.0);
        previous = weight;
    }
    assert_close(previous, 1.0);
}

// ---- E1: the event table ----

#[test]
fn e1_every_event_maps_to_its_clip() {
    assert_eq!(clip_for(Event::Idle), Clip::Idle);
    assert_eq!(clip_for(Event::Greet), Clip::Wave);
    assert_eq!(clip_for(Event::FindingRaised), Clip::Point);
    assert_eq!(clip_for(Event::UnsafeRaised), Clip::Worried);
    assert_eq!(clip_for(Event::Milestone), Clip::Celebrate);
    assert_eq!(clip_for(Event::Asked), Clip::Think);
    assert_eq!(clip_for(Event::WalkingToTarget), Clip::WalkTo);
}

// ---- E2: clip keys ----

#[test]
fn e2_every_clip_has_the_persona_name() {
    assert_eq!(Clip::Idle.key(), "idle");
    assert_eq!(Clip::Wave.key(), "wave");
    assert_eq!(Clip::Point.key(), "point");
    assert_eq!(Clip::Think.key(), "think");
    assert_eq!(Clip::Worried.key(), "worried");
    assert_eq!(Clip::Celebrate.key(), "celebrate");
    assert_eq!(Clip::WalkTo.key(), "walk_to");
    assert_eq!(Clip::Sit.key(), "sit");
    assert_eq!(Clip::CarryBoard.key(), "carry_board");
}

#[test]
fn e2_the_clip_table_is_in_the_docs_order() {
    assert_eq!(CLIPS.len(), 9);
    assert_eq!(CLIPS[0], Clip::Idle);
    assert_eq!(CLIPS[1], Clip::Wave);
    assert_eq!(CLIPS[2], Clip::Point);
    assert_eq!(CLIPS[3], Clip::Think);
    assert_eq!(CLIPS[4], Clip::Worried);
    assert_eq!(CLIPS[5], Clip::Celebrate);
    assert_eq!(CLIPS[6], Clip::WalkTo);
    assert_eq!(CLIPS[7], Clip::Sit);
    assert_eq!(CLIPS[8], Clip::CarryBoard);
    let mut keys: Vec<&str> = CLIPS.iter().map(|c| c.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), 9, "clip keys must be unique");
}

// ---- T1: the hardware tiers ----

#[test]
fn t1_capable_runs_the_live_path() {
    let c = caps(Tier::Capable);
    assert!(c.skinning);
    assert!(c.clips);
    assert!(c.crossfade);
    assert!(c.look_at);
    assert!(c.point_at);
    assert!(!c.baked);
    assert!(c.frame_rate.is_none());
}

#[test]
fn t1_modest_keeps_skinning_but_drops_the_ik() {
    let c = caps(Tier::Modest);
    assert!(c.skinning);
    assert!(c.clips);
    assert!(c.crossfade);
    assert!(!c.look_at);
    assert!(!c.point_at);
    assert!(!c.baked);
    assert!(c.frame_rate.is_none());
}

#[test]
fn t1_potato_is_baked_at_twelve_frames() {
    let c = caps(Tier::Potato);
    assert!(!c.skinning);
    assert!(c.clips);
    assert!(!c.crossfade);
    assert!(!c.look_at);
    assert!(!c.point_at);
    assert!(c.baked);
    assert_eq!(c.frame_rate, Some(12.0));
}

#[test]
fn t1_muted_runs_nothing() {
    let c = caps(Tier::Muted);
    assert!(!c.skinning);
    assert!(!c.clips);
    assert!(!c.crossfade);
    assert!(!c.look_at);
    assert!(!c.point_at);
    assert!(!c.baked);
    assert!(c.frame_rate.is_none());
}

// ---- W1: wrapping angles ----

#[test]
fn w1_wrap_angle_worked_examples() {
    let pi = std::f64::consts::PI;
    assert_close(wrap_angle(pi), pi);
    assert_close(wrap_angle(-pi), pi);
    assert_close(wrap_angle(3.0 * pi / 2.0), -pi / 2.0);
    assert_close(wrap_angle(-3.0 * pi / 2.0), pi / 2.0);
    assert_close(wrap_angle(7.0), 0.716815);
    assert_close(wrap_angle(0.0), 0.0);
}

#[test]
fn w1_wrap_angle_always_lands_in_the_turn() {
    let pi = std::f64::consts::PI;
    for i in -400..=400i32 {
        let angle = (i as f64) * 0.05;
        let wrapped = wrap_angle(angle);
        assert!(wrapped > -pi - 1e-12, "{angle} wrapped to {wrapped}");
        assert!(wrapped <= pi + 1e-12, "{angle} wrapped to {wrapped}");
    }
}

// ---- W2: walking ----

#[test]
fn w2_walking_straight_ahead_needs_no_turn() {
    let walk = walk_to([0.0, 0.0], 0.0, [0.0, 3.0], 0.5);
    assert_close(walk.turn, 0.0);
    assert_close(walk.distance, 2.5);
    assert_close(walk.heading, 0.0);
}

#[test]
fn w2_a_target_to_his_left_turns_him_left() {
    let walk = walk_to([0.0, 0.0], 0.0, [-3.0, 0.0], 0.5);
    assert_close(walk.turn, std::f64::consts::PI / 2.0);
    assert_close(walk.distance, 2.5);
    assert_close(walk.heading, std::f64::consts::PI / 2.0);
}

#[test]
fn w2_a_target_to_his_right_turns_him_right() {
    let walk = walk_to([0.0, 0.0], std::f64::consts::PI / 2.0, [3.0, 0.0], 0.5);
    assert_close(walk.turn, std::f64::consts::PI);
    assert_close(walk.distance, 2.5);
    assert_close(walk.heading, -std::f64::consts::PI / 2.0);
}

#[test]
fn w2_standing_still_keeps_his_heading() {
    let walk = walk_to([1.0, 1.0], 4.0, [1.0, 1.0], 0.5);
    assert_close(walk.turn, 0.0);
    assert_close(walk.distance, 0.0);
    assert_close(walk.heading, -2.283185);
}

#[test]
fn w2_close_enough_to_stop_still_turns_to_face_it() {
    let walk = walk_to([0.0, 0.0], std::f64::consts::PI / 2.0, [0.0, 0.3], 0.5);
    assert_close(walk.turn, -std::f64::consts::PI / 2.0);
    assert_close(walk.distance, 0.0);
    assert_close(walk.heading, 0.0);
}

#[test]
fn w2_a_bad_stop_short_counts_as_zero() {
    let plain = walk_to([0.0, 0.0], 0.0, [0.0, 3.0], 0.0);
    let nan = walk_to([0.0, 0.0], 0.0, [0.0, 3.0], f64::NAN);
    let inf = walk_to([0.0, 0.0], 0.0, [0.0, 3.0], f64::INFINITY);
    let negative = walk_to([0.0, 0.0], 0.0, [0.0, 3.0], -2.0);
    assert_close(plain.distance, 3.0);
    assert_close(nan.distance, 3.0);
    assert_close(inf.distance, 3.0);
    assert_close(negative.distance, 3.0);
}

#[test]
fn w2_non_finite_inputs_give_a_standing_walk() {
    let still = walk_to([0.0, 0.0], 0.0, [0.0, 3.0], 0.5);
    let bad = [
        walk_to([f64::NAN, 0.0], 0.0, [0.0, 3.0], 0.5),
        walk_to([0.0, 0.0], f64::INFINITY, [0.0, 3.0], 0.5),
        walk_to([0.0, 0.0], 0.0, [0.0, f64::NEG_INFINITY], 0.5),
    ];
    for walk in bad {
        assert_close(walk.turn, 0.0);
        assert_close(walk.distance, 0.0);
        assert_close(walk.heading, 0.0);
    }
    assert_close(still.distance, 2.5);
}

// ---- H1: height ----

#[test]
fn h1_height_scale_worked_examples() {
    let rest = DEFAULT_HEIGHT_M;
    assert_close(height_scale(1.68, rest).expect("in range"), 1.0);
    assert_close(height_scale(1.905, rest).expect("in range"), 1.133929);
    assert_close(height_scale(1.2, rest).expect("in range"), 0.714286);
    assert_close(
        height_scale(MIN_HEIGHT_M, rest).expect("inner end"),
        0.297619,
    );
}

#[test]
fn h1_out_of_range_or_bad_heights_give_none() {
    assert!(height_scale(0.3, DEFAULT_HEIGHT_M).is_none());
    assert!(height_scale(2.6, DEFAULT_HEIGHT_M).is_none());
    assert!(height_scale(f64::NAN, DEFAULT_HEIGHT_M).is_none());
    assert!(height_scale(f64::INFINITY, DEFAULT_HEIGHT_M).is_none());
    assert!(height_scale(DEFAULT_HEIGHT_M, 0.0).is_none());
    assert!(height_scale(DEFAULT_HEIGHT_M, -1.68).is_none());
    assert!(height_scale(DEFAULT_HEIGHT_M, f64::NAN).is_none());
    assert!(height_scale(DEFAULT_HEIGHT_M, f64::INFINITY).is_none());
    assert!(height_scale(MAX_HEIGHT_M, DEFAULT_HEIGHT_M).is_some());
}

// ---- no input may panic ----

#[test]
fn nan_inputs_never_panic_in_look_at() {
    let d = default_limits();
    let nan = f64::NAN;
    assert!(look_at([nan, 0.0, 0.0], [0.0, 1.0, 0.0], d, 0.5).is_none());
    assert!(look_at(EYE, [0.0, nan, 0.0], d, 0.5).is_none());
    let bad_limits = Limits {
        yaw_max: nan,
        pitch_up: nan,
        pitch_down: nan,
    };
    let look = look_at(EYE, [-1.0, 1.0, 1.6], bad_limits, nan).expect("still answers");
    assert_close(look.yaw, 0.0);
    assert_close(look.pitch, 0.0);
    assert!(look.clamped);
}

#[test]
fn infinite_inputs_never_panic_in_point_at() {
    let inf = f64::INFINITY;
    let pole = [0.0, 0.0, -1.0];
    let shoulder = [0.0, 0.0, 0.0];
    let target = [0.0, 0.4, 0.0];
    assert_eq!(
        point_at([inf, 0.0, 0.0], target, 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    assert_eq!(
        point_at(shoulder, [0.0, -inf, 0.0], 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    assert_eq!(
        point_at(shoulder, target, 0.3, -inf, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(shoulder, target, 0.3, 0.25, [0.0, inf, -1.0]),
        Err(IkError::PoleAlongArm)
    );
    let huge = point_at(shoulder, [0.0, f64::MAX, 0.0], 0.3, 0.25, pole).expect("no panic");
    assert!(!huge.reached);
}

#[test]
fn nan_and_infinite_scalars_never_panic() {
    let nan = f64::NAN;
    let inf = f64::INFINITY;
    assert!(crossfade_duration(nan).is_finite());
    assert!(crossfade_duration(inf).is_finite());
    assert!(fade_weight(nan, inf).is_finite());
    assert!(fade_weight(inf, nan).is_finite());
    assert!(wrap_angle(nan).is_finite());
    assert!(wrap_angle(inf).is_finite());
    assert!(wrap_angle(-inf).is_finite());
    assert!(walk_to([nan, inf], nan, [nan, 0.0], inf).distance == 0.0);
    assert!(height_scale(nan, inf).is_none());
    assert!(height_scale(1.68, nan).is_none());
    // Enums take no floats, but they must still answer.
    assert_eq!(clip_for(Event::Idle), Clip::Idle);
    assert!(caps(Tier::Muted).frame_rate.is_none());
}
