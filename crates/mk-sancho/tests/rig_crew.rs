//! Tests for mk_sancho::rig written by Gordon (the Crew, campaign mk-core-14) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_sancho::rig`, written from the contract alone.

use mk_sancho::rig::{
    caps, clip_for, crossfade_duration, fade_weight, height_scale, look_at, point_at, walk_to,
    wrap_angle, Caps, Clip, Event, IkError, Limits, Tier, Vec3, Walk, CLIPS, CROSSFADE_MAX_S,
    CROSSFADE_MIN_S, CROSSFADE_S, DEFAULT_HEIGHT_M, MAX_HEIGHT_M, MIN_HEIGHT_M,
};

const PI: f64 = std::f64::consts::PI;
const EYE: Vec3 = [0.0, 0.0, 1.6];

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}

fn assert_close(got: f64, want: f64, tol: f64) {
    assert!((got - want).abs() < tol, "got {got}, want {want}");
}

fn assert_walk(got: Walk, turn: f64, distance: f64, heading: f64) {
    assert_close(got.turn, turn, 1e-9);
    assert_close(got.distance, distance, 1e-9);
    assert_close(got.heading, heading, 1e-9);
}

#[test]
fn constants_match_contract() {
    assert_eq!(DEFAULT_HEIGHT_M, 1.68);
    assert_eq!(MIN_HEIGHT_M, 0.5);
    assert_eq!(MAX_HEIGHT_M, 2.5);
    assert_eq!(CROSSFADE_S, 0.25);
    assert_eq!(CROSSFADE_MIN_S, 0.2);
    assert_eq!(CROSSFADE_MAX_S, 0.3);
    // 70 degrees either side, 30 up, 40 down
    assert_close(Limits::DEFAULT.yaw_max, 7.0 * PI / 18.0, 1e-12);
    assert_close(Limits::DEFAULT.pitch_up, PI / 6.0, 1e-12);
    assert_close(Limits::DEFAULT.pitch_down, 2.0 * PI / 9.0, 1e-12);
}

#[test]
fn l1_look_at_straight_ahead() {
    let look = look_at(EYE, [0.0, 2.0, 1.6], Limits::DEFAULT, 0.5).expect("ahead is visible");
    assert_eq!(look.yaw, 0.0);
    assert_eq!(look.pitch, 0.0);
    assert_eq!(look.neck_yaw, 0.0);
    assert_eq!(look.neck_pitch, 0.0);
    assert_eq!(look.head_yaw, 0.0);
    assert_eq!(look.head_pitch, 0.0);
    assert!(!look.clamped);
}

#[test]
fn l1_look_at_left_is_positive_yaw() {
    let look = look_at(EYE, [-1.0, 1.0, 1.6], Limits::DEFAULT, 0.5).expect("visible");
    assert_close(look.yaw, PI / 4.0, 1e-6);
    assert!(look.yaw > 0.0, "positive yaw turns to his left");
    assert_close(look.pitch, 0.0, 1e-12);
    assert_close(look.neck_yaw, PI / 8.0, 1e-6);
    assert_close(look.head_yaw, PI / 8.0, 1e-6);
    assert!(!look.clamped);
}

#[test]
fn l1_look_at_clamps_pitch_up() {
    // raw pitch atan2(1, sqrt(2)) = 0.615480, clamped to 30 degrees
    let look = look_at(EYE, [1.0, 1.0, 2.6], Limits::DEFAULT, 0.5).expect("visible");
    assert_close(look.yaw, -PI / 4.0, 1e-6);
    assert_close(look.pitch, PI / 6.0, 1e-6);
    assert_close(look.neck_yaw, -PI / 8.0, 1e-6);
    assert_close(look.neck_pitch, PI / 12.0, 1e-6);
    assert_close(look.head_yaw, -PI / 8.0, 1e-6);
    assert_close(look.head_pitch, PI / 12.0, 1e-6);
    assert!(look.clamped);
}

#[test]
fn l1_look_at_clamps_yaw() {
    // raw yaw atan2(2, -0.5) = 1.815775, clamped to 70 degrees
    let look = look_at(EYE, [-2.0, -0.5, 1.6], Limits::DEFAULT, 0.5).expect("visible");
    assert_close(look.yaw, 7.0 * PI / 18.0, 1e-6);
    assert_close(look.pitch, 0.0, 1e-12);
    assert!(look.clamped);
}

#[test]
fn l1_look_at_clamps_pitch_down() {
    // raw pitch atan2(-1, 1) = -pi/4, clamped to -40 degrees
    let look = look_at(EYE, [0.0, 1.0, 0.6], Limits::DEFAULT, 0.5).expect("visible");
    assert_close(look.yaw, 0.0, 1e-12);
    assert_close(look.pitch, -2.0 * PI / 9.0, 1e-6);
    assert_close(look.neck_pitch, -PI / 9.0, 1e-6);
    assert_close(look.head_pitch, -PI / 9.0, 1e-6);
    assert!(look.clamped);
}

#[test]
fn l1_look_at_zero_distance_is_none() {
    assert_eq!(look_at(EYE, EYE, Limits::DEFAULT, 0.5), None);
    assert_eq!(
        look_at([0.0, 0.0, 0.0], [1e-12, 0.0, 0.0], Limits::DEFAULT, 0.5),
        None
    );
}

#[test]
fn l1_look_at_non_finite_delta_is_none() {
    assert_eq!(
        look_at(EYE, [f64::NAN, 2.0, 1.6], Limits::DEFAULT, 0.5),
        None
    );
    assert_eq!(
        look_at(EYE, [0.0, f64::INFINITY, 1.6], Limits::DEFAULT, 0.5),
        None
    );
    assert_eq!(
        look_at([f64::NEG_INFINITY, 0.0, 1.6], EYE, Limits::DEFAULT, 0.5),
        None
    );
}

#[test]
fn l1_look_at_negative_limit_counts_as_zero() {
    let limits = Limits {
        yaw_max: -1.0,
        pitch_up: 0.5,
        pitch_down: 0.5,
    };
    let look = look_at(EYE, [-1.0, 1.0, 1.6], limits, 0.5).expect("visible");
    assert_eq!(look.yaw, 0.0);
    assert_eq!(look.pitch, 0.0);
    assert!(look.clamped);
}

#[test]
fn l1_look_at_non_finite_limit_counts_as_zero() {
    let limits = Limits {
        yaw_max: f64::NAN,
        pitch_up: f64::INFINITY,
        pitch_down: f64::NEG_INFINITY,
    };
    let look = look_at(EYE, [-1.0, 1.0, 1.6], limits, 0.5).expect("visible");
    assert_eq!(look.yaw, 0.0);
    assert_eq!(look.pitch, 0.0);
    assert!(look.clamped);
}

#[test]
fn l2_neck_share_splits_the_turn() {
    let all_neck = look_at(EYE, [-1.0, 1.0, 1.6], Limits::DEFAULT, 1.0).expect("visible");
    assert_close(all_neck.neck_yaw, PI / 4.0, 1e-6);
    assert_close(all_neck.head_yaw, 0.0, 1e-12);
    let all_head = look_at(EYE, [-1.0, 1.0, 1.6], Limits::DEFAULT, 0.0).expect("visible");
    assert_close(all_head.neck_yaw, 0.0, 1e-12);
    assert_close(all_head.head_yaw, PI / 4.0, 1e-6);
}

#[test]
fn l2_neck_share_clamped_high() {
    let look = look_at(EYE, [-1.0, 1.0, 1.6], Limits::DEFAULT, 2.0).expect("visible");
    assert_close(look.neck_yaw, PI / 4.0, 1e-6);
    assert_close(look.head_yaw, 0.0, 1e-12);
}

#[test]
fn l2_neck_share_clamped_low() {
    let look = look_at(EYE, [-1.0, 1.0, 1.6], Limits::DEFAULT, -3.0).expect("visible");
    assert_close(look.neck_yaw, 0.0, 1e-12);
    assert_close(look.head_yaw, PI / 4.0, 1e-6);
}

#[test]
fn l2_neck_share_nan_is_half() {
    let look = look_at(EYE, [-1.0, 1.0, 1.6], Limits::DEFAULT, f64::NAN).expect("visible");
    assert_close(look.neck_yaw, PI / 8.0, 1e-6);
    assert_close(look.head_yaw, PI / 8.0, 1e-6);
}

#[test]
fn l2_split_sums_to_whole() {
    for share in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let look = look_at(EYE, [1.0, 1.0, 2.6], Limits::DEFAULT, share).expect("visible");
        assert_close(look.neck_yaw + look.head_yaw, look.yaw, 1e-12);
        assert_close(look.neck_pitch + look.head_pitch, look.pitch, 1e-12);
    }
}

#[test]
fn p1_bad_length_errors() {
    let s = [0.0, 0.0, 0.0];
    let t = [0.0, 0.4, 0.0];
    let pole = [0.0, 0.0, -1.0];
    assert_eq!(point_at(s, t, 0.0, 0.25, pole), Err(IkError::BadLength));
    assert_eq!(point_at(s, t, -0.3, 0.25, pole), Err(IkError::BadLength));
    assert_eq!(
        point_at(s, t, f64::NAN, 0.25, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(s, t, f64::INFINITY, 0.25, pole),
        Err(IkError::BadLength)
    );
    assert_eq!(point_at(s, t, 0.3, 0.0, pole), Err(IkError::BadLength));
    assert_eq!(point_at(s, t, 0.3, -0.25, pole), Err(IkError::BadLength));
    assert_eq!(point_at(s, t, 0.3, f64::NAN, pole), Err(IkError::BadLength));
}

#[test]
fn p1_bad_length_checked_before_direction() {
    // both the length and the direction are bad: BadLength is reported first
    assert_eq!(
        point_at(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            0.25,
            [0.0, 0.0, -1.0]
        ),
        Err(IkError::BadLength)
    );
    assert_eq!(
        point_at(
            [f64::NAN, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.3,
            -1.0,
            [0.0, 0.0, -1.0]
        ),
        Err(IkError::BadLength)
    );
}

#[test]
fn p1_no_direction_errors() {
    let pole = [0.0, 0.0, -1.0];
    assert_eq!(
        point_at([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    assert_eq!(
        point_at([0.0, 0.0, 0.0], [1e-12, 0.0, 0.0], 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    assert_eq!(
        point_at([f64::NAN, 0.0, 0.0], [0.0, 0.4, 0.0], 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
    assert_eq!(
        point_at([0.0, 0.0, 0.0], [0.0, f64::INFINITY, 0.0], 0.3, 0.25, pole),
        Err(IkError::NoDirection)
    );
}

#[test]
fn p1_no_direction_checked_before_pole() {
    // target on the shoulder and a pole along the arm: NoDirection is reported first
    assert_eq!(
        point_at([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.3, 0.25, [0.0, 2.0, 0.0]),
        Err(IkError::NoDirection)
    );
}

#[test]
fn p1_pole_along_arm_errors() {
    let s = [0.0, 0.0, 0.0];
    let t = [0.0, 0.4, 0.0];
    assert_eq!(
        point_at(s, t, 0.3, 0.25, [0.0, 2.0, 0.0]),
        Err(IkError::PoleAlongArm)
    );
    assert_eq!(
        point_at(s, t, 0.3, 0.25, [0.0, -3.0, 0.0]),
        Err(IkError::PoleAlongArm)
    );
    assert_eq!(
        point_at(s, t, 0.3, 0.25, [0.0, 0.0, 0.0]),
        Err(IkError::PoleAlongArm)
    );
    assert_eq!(
        point_at(s, t, 0.3, 0.25, [f64::NAN, 0.0, -1.0]),
        Err(IkError::PoleAlongArm)
    );
    assert_eq!(
        point_at(s, t, 0.3, 0.25, [0.0, f64::INFINITY, -1.0]),
        Err(IkError::PoleAlongArm)
    );
}

#[test]
fn p2_worked_example_reach() {
    // d = 0.4 inside [0.05, 0.55]; x = (0.09 - 0.0625 + 0.16) / 0.8 = 0.234375;
    // h = sqrt(0.09 - x*x) = 0.187265; cos = (0.09 + 0.0625 - 0.16) / 0.15 = -0.05
    let arm = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    let h = (0.3_f64 * 0.3 - 0.234375 * 0.234375).sqrt();
    assert_close(arm.elbow[0], 0.0, 1e-9);
    assert_close(arm.elbow[1], 0.234375, 1e-9);
    assert_close(arm.elbow[2], -h, 1e-9);
    assert_close(arm.hand[0], 0.0, 1e-9);
    assert_close(arm.hand[1], 0.4, 1e-9);
    assert_close(arm.hand[2], 0.0, 1e-9);
    assert_close(arm.elbow_angle, (-0.05_f64).acos(), 1e-9);
    assert!(arm.reached);
}

#[test]
fn p2_worked_example_too_far() {
    // d = 1.0 beyond 0.55: dc = 0.55, x = 0.33 / 1.1 = 0.3, h = 0
    let arm = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    assert_close(arm.elbow[0], 0.0, 1e-9);
    assert_close(arm.elbow[1], 0.3, 1e-9);
    assert_close(arm.elbow[2], 0.0, 1e-9);
    assert_close(arm.hand[0], 0.0, 1e-9);
    assert_close(arm.hand[1], 0.55, 1e-9);
    assert_close(arm.hand[2], 0.0, 1e-9);
    assert_close(arm.elbow_angle, PI, 1e-9);
    assert!(!arm.reached);
}

#[test]
fn p2_worked_example_too_close() {
    // d = 0.02 below 0.05: dc = 0.05, x = 0.03 / 0.1 = 0.3, h = 0
    let arm = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.02, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    assert_close(arm.elbow[0], 0.0, 1e-9);
    assert_close(arm.elbow[1], 0.3, 1e-9);
    assert_close(arm.elbow[2], 0.0, 1e-9);
    assert_close(arm.hand[0], 0.0, 1e-9);
    assert_close(arm.hand[1], 0.05, 1e-9);
    assert_close(arm.hand[2], 0.0, 1e-9);
    assert_close(arm.elbow_angle, 0.0, 1e-9);
    assert!(!arm.reached);
}

#[test]
fn p2_pole_component_along_arm_ignored() {
    // the pole's part along the arm is dropped, so (0, 0.5, -1) gives the same answer
    let a = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        0.3,
        0.25,
        [0.0, 0.0, -1.0],
    )
    .expect("solvable");
    let b = point_at(
        [0.0, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        0.3,
        0.25,
        [0.0, 0.5, -1.0],
    )
    .expect("solvable");
    assert_close(a.elbow[0], b.elbow[0], 1e-9);
    assert_close(a.elbow[1], b.elbow[1], 1e-9);
    assert_close(a.elbow[2], b.elbow[2], 1e-9);
    assert_close(a.hand[0], b.hand[0], 1e-9);
    assert_close(a.hand[1], b.hand[1], 1e-9);
    assert_close(a.hand[2], b.hand[2], 1e-9);
    assert_close(a.elbow_angle, b.elbow_angle, 1e-9);
    assert_eq!(a.reached, b.reached);
}

#[test]
fn p2_identities_over_grid() {
    let shoulder = [0.1, -0.2, 1.3];
    let upper: f64 = 0.3;
    let lower: f64 = 0.25;
    let pole = [0.3, 0.4, -1.0];
    let reach_min = (upper - lower).abs();
    let reach_max = upper + lower;
    for dx in [-0.4, 0.0, 0.4] {
        for dy in [0.1, 0.3, 0.6] {
            for dz in [-0.2, 0.0, 0.2] {
                let target = [shoulder[0] + dx, shoulder[1] + dy, shoulder[2] + dz];
                let arm = point_at(shoulder, target, upper, lower, pole)
                    .expect("grid target should solve");
                let delta = sub(target, shoulder);
                let d = len(delta);
                let u = [delta[0] / d, delta[1] / d, delta[2] / d];
                let pu = dot(pole, u);
                let p = [
                    pole[0] - pu * u[0],
                    pole[1] - pu * u[1],
                    pole[2] - pu * u[2],
                ];
                let plen = len(p);
                let n = [p[0] / plen, p[1] / plen, p[2] / plen];
                let dc = d.max(reach_min).min(reach_max);
                assert_close(len(sub(arm.elbow, shoulder)), upper, 1e-9);
                assert_close(len(sub(arm.hand, arm.elbow)), lower, 1e-9);
                assert!(
                    dot(sub(arm.elbow, shoulder), n) >= -1e-12,
                    "elbow off the pole side"
                );
                assert_close(len(sub(arm.hand, shoulder)), dc, 1e-9);
                assert_eq!(arm.reached, (reach_min..=reach_max).contains(&d));
                let cos_want = ((upper * upper + lower * lower - dc * dc) / (2.0 * upper * lower))
                    .clamp(-1.0, 1.0);
                assert_close(arm.elbow_angle, cos_want.acos(), 1e-9);
            }
        }
    }
}

#[test]
fn p2_reached_flag_inclusive_bounds() {
    let s = [0.0, 0.0, 0.0];
    let pole = [0.0, 0.0, -1.0];
    // upper 0.5, lower 0.25: the reach is [0.25, 0.75], both ends inclusive
    let at_max = point_at(s, [0.0, 0.75, 0.0], 0.5, 0.25, pole).expect("solvable");
    assert!(at_max.reached);
    assert_close(at_max.hand[1], 0.75, 1e-9);
    assert_close(at_max.elbow_angle, PI, 1e-9);
    let at_min = point_at(s, [0.0, 0.25, 0.0], 0.5, 0.25, pole).expect("solvable");
    assert!(at_min.reached);
    assert_close(at_min.hand[1], 0.25, 1e-9);
    assert_close(at_min.elbow_angle, 0.0, 1e-9);
    let past_max = point_at(s, [0.0, 0.7500001, 0.0], 0.5, 0.25, pole).expect("solvable");
    assert!(!past_max.reached);
    assert_close(past_max.hand[1], 0.75, 1e-9);
    let past_min = point_at(s, [0.0, 0.2499999, 0.0], 0.5, 0.25, pole).expect("solvable");
    assert!(!past_min.reached);
    assert_close(past_min.hand[1], 0.25, 1e-9);
}

#[test]
fn c1_crossfade_duration_clamps() {
    assert_eq!(crossfade_duration(0.1), 0.2);
    assert_eq!(crossfade_duration(0.0), 0.2);
    assert_eq!(crossfade_duration(-1.0), 0.2);
    assert_eq!(crossfade_duration(0.2), 0.2);
    assert_eq!(crossfade_duration(0.25), 0.25);
    assert_eq!(crossfade_duration(0.3), 0.3);
    assert_eq!(crossfade_duration(0.5), 0.3);
}

#[test]
fn c1_crossfade_duration_non_finite() {
    assert_eq!(crossfade_duration(f64::NAN), CROSSFADE_S);
    assert_eq!(crossfade_duration(f64::INFINITY), CROSSFADE_S);
    assert_eq!(crossfade_duration(f64::NEG_INFINITY), CROSSFADE_S);
}

#[test]
fn c2_fade_weight_worked_examples() {
    assert_eq!(fade_weight(0.0, 0.25), 0.0);
    assert_eq!(fade_weight(0.0625, 0.25), 0.15625);
    assert_eq!(fade_weight(0.125, 0.25), 0.5);
    assert_eq!(fade_weight(0.25, 0.25), 1.0);
    assert_eq!(fade_weight(1.0, 0.25), 1.0);
    assert_eq!(fade_weight(-1.0, 0.25), 0.0);
}

#[test]
fn c2_fade_weight_bad_duration_is_one() {
    assert_eq!(fade_weight(0.1, 0.0), 1.0);
    assert_eq!(fade_weight(0.1, -0.5), 1.0);
    assert_eq!(fade_weight(0.1, f64::NAN), 1.0);
    assert_eq!(fade_weight(0.1, f64::INFINITY), 1.0);
}

#[test]
fn c2_fade_weight_nan_elapsed_is_zero() {
    assert_eq!(fade_weight(f64::NAN, 0.25), 0.0);
}

#[test]
fn c2_fade_weight_monotonic_rise() {
    let duration: f64 = 0.25;
    let mut prev = fade_weight(0.0, duration);
    assert_close(prev, 0.0, 1e-12);
    for i in 1..=100 {
        let t = i as f64 / 100.0;
        let w = fade_weight(t * duration, duration);
        assert!(w >= prev - 1e-12, "weight fell at t = {t}");
        prev = w;
    }
    assert_close(prev, 1.0, 1e-12);
    // smoothstep is symmetric about the middle: w(t) + w(1 - t) = 1
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        let a = fade_weight(t * duration, duration);
        let b = fade_weight((1.0 - t) * duration, duration);
        assert_close(a + b, 1.0, 1e-12);
    }
}

#[test]
fn e1_clip_for_every_event() {
    assert_eq!(clip_for(Event::Idle), Clip::Idle);
    assert_eq!(clip_for(Event::Greet), Clip::Wave);
    assert_eq!(clip_for(Event::FindingRaised), Clip::Point);
    assert_eq!(clip_for(Event::UnsafeRaised), Clip::Worried);
    assert_eq!(clip_for(Event::Milestone), Clip::Celebrate);
    assert_eq!(clip_for(Event::Asked), Clip::Think);
    assert_eq!(clip_for(Event::WalkingToTarget), Clip::WalkTo);
}

#[test]
fn e2_clip_keys() {
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
fn e2_clips_array_is_the_doc_order() {
    assert_eq!(
        CLIPS,
        [
            Clip::Idle,
            Clip::Wave,
            Clip::Point,
            Clip::Think,
            Clip::Worried,
            Clip::Celebrate,
            Clip::WalkTo,
            Clip::Sit,
            Clip::CarryBoard,
        ]
    );
}

#[test]
fn t1_caps_every_tier() {
    assert_eq!(
        caps(Tier::Capable),
        Caps {
            skinning: true,
            clips: true,
            crossfade: true,
            look_at: true,
            point_at: true,
            baked: false,
            frame_rate: None,
        }
    );
    assert_eq!(
        caps(Tier::Modest),
        Caps {
            skinning: true,
            clips: true,
            crossfade: true,
            look_at: false,
            point_at: false,
            baked: false,
            frame_rate: None,
        }
    );
    assert_eq!(
        caps(Tier::Potato),
        Caps {
            skinning: false,
            clips: true,
            crossfade: false,
            look_at: false,
            point_at: false,
            baked: true,
            frame_rate: Some(12.0),
        }
    );
    assert_eq!(
        caps(Tier::Muted),
        Caps {
            skinning: false,
            clips: false,
            crossfade: false,
            look_at: false,
            point_at: false,
            baked: false,
            frame_rate: None,
        }
    );
}

#[test]
fn w1_wrap_angle_worked_examples() {
    assert_close(wrap_angle(PI), PI, 1e-12);
    assert_close(wrap_angle(-PI), PI, 1e-12);
    assert_close(wrap_angle(3.0 * PI / 2.0), -PI / 2.0, 1e-12);
    assert_close(wrap_angle(-3.0 * PI / 2.0), PI / 2.0, 1e-12);
    assert_close(wrap_angle(7.0), 7.0 - 2.0 * PI, 1e-12);
    assert_eq!(wrap_angle(0.0), 0.0);
}

#[test]
fn w1_wrap_angle_non_finite() {
    assert_eq!(wrap_angle(f64::NAN), 0.0);
    assert_eq!(wrap_angle(f64::INFINITY), 0.0);
    assert_eq!(wrap_angle(f64::NEG_INFINITY), 0.0);
}

#[test]
fn w2_walk_to_worked_examples() {
    assert_walk(walk_to([0.0, 0.0], 0.0, [0.0, 3.0], 0.5), 0.0, 2.5, 0.0);
    assert_walk(
        walk_to([0.0, 0.0], 0.0, [-3.0, 0.0], 0.5),
        PI / 2.0,
        2.5,
        PI / 2.0,
    );
    assert_walk(
        walk_to([0.0, 0.0], PI / 2.0, [3.0, 0.0], 0.5),
        PI,
        2.5,
        -PI / 2.0,
    );
    assert_walk(
        walk_to([1.0, 1.0], 4.0, [1.0, 1.0], 0.5),
        0.0,
        0.0,
        4.0 - 2.0 * PI,
    );
    assert_walk(
        walk_to([0.0, 0.0], PI / 2.0, [0.0, 0.3], 0.5),
        -PI / 2.0,
        0.0,
        0.0,
    );
}

#[test]
fn w2_walk_to_non_finite_inputs() {
    let zero = Walk {
        turn: 0.0,
        distance: 0.0,
        heading: 0.0,
    };
    assert_eq!(walk_to([f64::NAN, 0.0], 0.0, [0.0, 3.0], 0.5), zero);
    assert_eq!(walk_to([0.0, f64::INFINITY], 0.0, [0.0, 3.0], 0.5), zero);
    assert_eq!(walk_to([0.0, 0.0], f64::NAN, [0.0, 3.0], 0.5), zero);
    assert_eq!(
        walk_to([0.0, 0.0], f64::NEG_INFINITY, [0.0, 3.0], 0.5),
        zero
    );
    assert_eq!(walk_to([0.0, 0.0], 0.0, [f64::NAN, 3.0], 0.5), zero);
    assert_eq!(walk_to([0.0, 0.0], 0.0, [0.0, f64::INFINITY], 0.5), zero);
}

#[test]
fn w2_walk_to_stop_short_bad_counts_as_zero() {
    assert_walk(walk_to([0.0, 0.0], 0.0, [0.0, 3.0], -1.0), 0.0, 3.0, 0.0);
    assert_walk(
        walk_to([0.0, 0.0], 0.0, [0.0, 3.0], f64::NAN),
        0.0,
        3.0,
        0.0,
    );
    assert_walk(
        walk_to([0.0, 0.0], 0.0, [0.0, 3.0], f64::INFINITY),
        0.0,
        3.0,
        0.0,
    );
    // a stop_short longer than the walk leaves no distance but keeps the heading
    assert_walk(walk_to([0.0, 0.0], 0.0, [0.0, 3.0], 5.0), 0.0, 0.0, 0.0);
}

#[test]
fn w2_walk_to_close_target_still_turns() {
    // already inside stop_short: no distance, but he still turns to face the target
    assert_walk(
        walk_to([0.0, 0.0], PI / 2.0, [0.0, 0.3], 0.5),
        -PI / 2.0,
        0.0,
        0.0,
    );
    assert_walk(walk_to([2.0, 2.0], 0.0, [2.0, 2.1], 0.5), 0.0, 0.0, 0.0);
    // a target behind and to his right: heading -3pi/4, turn -3pi/4, no distance
    assert_walk(
        walk_to([0.0, 0.0], 0.0, [0.2, -0.2], 0.5),
        -3.0 * PI / 4.0,
        0.0,
        -3.0 * PI / 4.0,
    );
}

#[test]
fn h1_height_scale_worked_examples() {
    assert_eq!(height_scale(1.68, 1.68), Some(1.0));
    let a = height_scale(1.905, 1.68).expect("in range");
    assert_close(a, 1.1339285714285714, 1e-6);
    let b = height_scale(1.2, 1.68).expect("in range");
    assert_close(b, 0.7142857142857143, 1e-6);
    assert_eq!(height_scale(0.3, 1.68), None);
    assert_eq!(height_scale(1.68, 0.0), None);
    assert_eq!(height_scale(f64::NAN, 1.68), None);
    assert_eq!(height_scale(2.6, 1.68), None);
}

#[test]
fn h1_height_scale_bounds_inclusive() {
    let lo = height_scale(MIN_HEIGHT_M, 1.68).expect("0.5 is allowed");
    assert_close(lo, 0.5 / 1.68, 1e-12);
    let hi = height_scale(MAX_HEIGHT_M, 1.68).expect("2.5 is allowed");
    assert_close(hi, 2.5 / 1.68, 1e-12);
    assert_eq!(height_scale(0.4999999, 1.68), None);
    assert_eq!(height_scale(2.5000001, 1.68), None);
}

#[test]
fn h1_height_scale_bad_rest() {
    assert_eq!(height_scale(1.68, -1.0), None);
    assert_eq!(height_scale(1.68, f64::NAN), None);
    assert_eq!(height_scale(1.68, f64::INFINITY), None);
    assert_eq!(height_scale(f64::INFINITY, 1.68), None);
    assert_eq!(height_scale(f64::NEG_INFINITY, 1.68), None);
}

#[test]
fn no_panic_on_nan_or_infinite_input() {
    let bad = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
    for v in bad {
        let _ = look_at([v, 0.0, 0.0], [0.0, 1.0, 0.0], Limits::DEFAULT, v);
        let _ = look_at([0.0, 0.0, 0.0], [v, 1.0, 0.0], Limits::DEFAULT, 0.5);
        let _ = point_at([v, 0.0, 0.0], [0.0, 0.4, 0.0], 0.3, 0.25, [0.0, 0.0, -1.0]);
        let _ = point_at([0.0, 0.0, 0.0], [0.0, 0.4, 0.0], v, 0.25, [0.0, 0.0, -1.0]);
        let _ = point_at([0.0, 0.0, 0.0], [0.0, 0.4, 0.0], 0.3, v, [0.0, 0.0, -1.0]);
        let _ = point_at([0.0, 0.0, 0.0], [0.0, 0.4, 0.0], 0.3, 0.25, [v, 0.0, -1.0]);
        assert!(crossfade_duration(v).is_finite());
        assert!(fade_weight(v, 0.25).is_finite());
        assert!(fade_weight(0.1, v).is_finite());
        assert!(wrap_angle(v).is_finite());
        let zero = Walk {
            turn: 0.0,
            distance: 0.0,
            heading: 0.0,
        };
        assert_eq!(walk_to([v, 0.0], 0.0, [0.0, 3.0], 0.5), zero);
        assert_eq!(walk_to([0.0, 0.0], v, [0.0, 3.0], 0.5), zero);
        assert_eq!(walk_to([0.0, 0.0], 0.0, [v, 3.0], 0.5), zero);
        assert_eq!(height_scale(v, 1.68), None);
        assert_eq!(height_scale(1.68, v), None);
    }
    // a limit that is not finite counts as 0 and must not panic
    let bad_limits = Limits {
        yaw_max: f64::NAN,
        pitch_up: f64::INFINITY,
        pitch_down: f64::NEG_INFINITY,
    };
    let look = look_at(EYE, [-1.0, 1.0, 1.6], bad_limits, f64::NAN).expect("visible");
    assert_eq!(look.yaw, 0.0);
    assert_eq!(look.pitch, 0.0);
}
