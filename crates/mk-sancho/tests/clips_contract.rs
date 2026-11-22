//! Contract tests for mk_sancho::clips, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_clips.md. They were the hidden tests of campaign mk-core-22.

use mk_sancho::clips::{
    bake, blend, clip_time, frame_index, nlerp, normalize, sample, sample_track, validate, Baked,
    Clip, ClipError, Key, Pose, Quat, Track, IDENTITY,
};

/// The turn by `deg` degrees about +z, as a quaternion [x, y, z, w].
fn q(deg: f64) -> Quat {
    let half = deg.to_radians() / 2.0;
    [0.0, 0.0, half.sin(), half.cos()]
}

fn close(a: Quat, b: Quat, eps: f64) -> bool {
    a.iter().zip(b.iter()).all(|(&x, &y)| (x - y).abs() <= eps)
}

fn pose_close(a: &Pose, b: &Pose, eps: f64) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(&x, &y)| close(x, y, eps))
}

/// Track T of the contract: bone 0, keys (0, IDENTITY), (1, q(90)), (2, IDENTITY).
fn track_t() -> Track {
    Track {
        bone: 0,
        keys: vec![
            Key {
                time_s: 0.0,
                rotation: IDENTITY,
            },
            Key {
                time_s: 1.0,
                rotation: q(90.0),
            },
            Key {
                time_s: 2.0,
                rotation: IDENTITY,
            },
        ],
    }
}

/// Clip W: name "wave", 2 s, looping, one track T.
fn wave() -> Clip {
    Clip {
        name: "wave".to_string(),
        duration_s: 2.0,
        looping: true,
        tracks: vec![track_t()],
    }
}

/// Clip S: the same as W but not looping.
fn still() -> Clip {
    Clip {
        name: "wave".to_string(),
        duration_s: 2.0,
        looping: false,
        tracks: vec![track_t()],
    }
}

#[test]
fn q1_normalize_scales_to_unit_length() {
    assert!(matches!(
        normalize([0.0, 0.0, 0.0, 2.0]),
        Some(v) if close(v, IDENTITY, 1e-12)
    ));
}

#[test]
fn q1_normalize_zero_is_none() {
    assert!(normalize([0.0, 0.0, 0.0, 0.0]).is_none());
}

#[test]
fn q1_normalize_rejects_non_finite_and_tiny() {
    assert!(normalize([f64::NAN, 0.0, 0.0, 1.0]).is_none());
    assert!(normalize([0.0, f64::INFINITY, 0.0, 1.0]).is_none());
    assert!(normalize([0.0, 0.0, 0.0, 1e-13]).is_none());
}

#[test]
fn q2_nlerp_midpoint_is_q45() {
    assert!(close(nlerp(IDENTITY, q(90.0), 0.5), q(45.0), 1e-6));
}

#[test]
fn q2_nlerp_quarter_matches_the_worked_row() {
    let got = nlerp(IDENTITY, q(90.0), 0.25);
    assert!(close(got, [0.0, 0.0, 0.1874, 0.9823], 1e-4));
}

#[test]
fn q2_nlerp_takes_the_shorter_way_round() {
    // The worked row's b is q(90) with both halves negated: the far side of the sphere.
    let half = 0.5_f64.sqrt();
    let b = [0.0, 0.0, -half, -half];
    assert!(close(nlerp(IDENTITY, b, 0.5), q(45.0), 1e-6));
}

#[test]
fn q2_nlerp_clamps_the_weight() {
    assert!(close(nlerp(IDENTITY, q(90.0), 7.0), q(90.0), 1e-6));
    assert!(close(nlerp(IDENTITY, q(90.0), -3.0), IDENTITY, 1e-6));
}

#[test]
fn q2_nlerp_nan_weight_is_zero() {
    assert!(close(nlerp(IDENTITY, q(90.0), f64::NAN), IDENTITY, 1e-6));
}

#[test]
fn q2_nlerp_normalizes_its_inputs() {
    assert!(close(
        nlerp([0.0, 0.0, 0.0, 2.0], [0.0, 0.0, 0.0, 2.0], 0.5),
        IDENTITY,
        1e-12
    ));
    assert!(close(
        nlerp([0.0, 0.0, 0.0, 0.0], q(90.0), 1.0),
        q(90.0),
        1e-6
    ));
}

#[test]
fn t1_at_or_before_the_first_key() {
    assert!(close(sample_track(&track_t(), -1.0), IDENTITY, 1e-6));
    assert!(close(sample_track(&track_t(), 0.0), IDENTITY, 1e-6));
}

#[test]
fn t1_between_two_keys_uses_nlerp() {
    assert!(close(sample_track(&track_t(), 0.5), q(45.0), 1e-6));
}

#[test]
fn t1_exactly_on_a_middle_key_returns_it() {
    assert!(close(sample_track(&track_t(), 1.0), q(90.0), 1e-6));
}

#[test]
fn t1_between_the_last_two_keys() {
    assert!(close(sample_track(&track_t(), 1.5), q(45.0), 1e-6));
}

#[test]
fn t1_at_or_after_the_last_key() {
    assert!(close(sample_track(&track_t(), 9.0), IDENTITY, 1e-6));
}

#[test]
fn t1_no_keys_is_identity() {
    let empty = Track {
        bone: 0,
        keys: Vec::new(),
    };
    assert!(close(sample_track(&empty, 0.5), IDENTITY, 1e-12));
}

#[test]
fn t1_nan_time_counts_as_zero() {
    let got = sample_track(&track_t(), f64::NAN);
    assert!(close(got, sample_track(&track_t(), 0.0), 1e-12));
}

#[test]
fn c1_looping_wraps_the_time() {
    assert!((clip_time(2.0, true, 2.5) - 0.5).abs() < 1e-12);
    assert!((clip_time(2.0, true, -0.5) - 1.5).abs() < 1e-12);
    assert!((clip_time(2.0, true, 2.0) - 0.0).abs() < 1e-12);
}

#[test]
fn c1_not_looping_clamps_the_time() {
    assert!((clip_time(2.0, false, 3.0) - 2.0).abs() < 1e-12);
    assert!((clip_time(2.0, false, -1.0) - 0.0).abs() < 1e-12);
}

#[test]
fn c1_bad_duration_is_zero() {
    assert!((clip_time(0.0, true, 1.0) - 0.0).abs() < 1e-12);
    assert!((clip_time(f64::NAN, true, 1.0) - 0.0).abs() < 1e-12);
    assert!((clip_time(f64::INFINITY, false, 1.0) - 0.0).abs() < 1e-12);
    assert!((clip_time(-2.0, false, 1.0) - 0.0).abs() < 1e-12);
}

#[test]
fn c1_non_finite_time_counts_as_zero() {
    assert!((clip_time(2.0, false, f64::NAN) - 0.0).abs() < 1e-12);
    assert!((clip_time(2.0, false, f64::INFINITY) - 0.0).abs() < 1e-12);
    assert!((clip_time(2.0, true, f64::NEG_INFINITY) - 0.0).abs() < 1e-12);
}

#[test]
fn c2_sample_looping_clip_midway() {
    let got = sample(&wave(), 2, 2.5);
    assert!(pose_close(&got, &vec![q(45.0), IDENTITY], 1e-6));
}

#[test]
fn c2_sample_non_looping_holds_the_last_key() {
    let got = sample(&still(), 2, 3.0);
    assert!(pose_close(&got, &vec![IDENTITY, IDENTITY], 1e-6));
}

#[test]
fn c2_sample_with_no_bones_is_empty() {
    assert!(sample(&wave(), 0, 1.0).is_empty());
}

#[test]
fn c2_first_track_for_a_bone_wins() {
    let mut clip = wave();
    clip.tracks.push(Track {
        bone: 0,
        keys: vec![Key {
            time_s: 0.0,
            rotation: q(90.0),
        }],
    });
    let got = sample(&clip, 2, 0.0);
    assert!(pose_close(&got, &vec![IDENTITY, IDENTITY], 1e-6));
}

#[test]
fn c2_tracks_for_other_bones_are_ignored() {
    let clip = Clip {
        name: "out-of-range".to_string(),
        duration_s: 1.0,
        looping: false,
        tracks: vec![Track {
            bone: 5,
            keys: vec![Key {
                time_s: 0.0,
                rotation: q(90.0),
            }],
        }],
    };
    let got = sample(&clip, 2, 0.0);
    assert!(pose_close(&got, &vec![IDENTITY, IDENTITY], 1e-6));
}

#[test]
fn b1_blend_halfway() {
    let got = blend(&vec![IDENTITY], &vec![q(90.0)], 0.5);
    assert!(pose_close(&got, &vec![q(45.0)], 1e-6));
}

#[test]
fn b1_missing_bones_in_b_stay_as_a() {
    let got = blend(&vec![IDENTITY, q(90.0)], &vec![q(90.0)], 1.0);
    assert!(pose_close(&got, &vec![q(90.0), q(90.0)], 1e-6));
}

#[test]
fn b1_weight_is_clamped() {
    assert!(pose_close(
        &blend(&vec![IDENTITY], &vec![q(90.0)], 2.0),
        &vec![q(90.0)],
        1e-6
    ));
    assert!(pose_close(
        &blend(&vec![IDENTITY], &vec![q(90.0)], -1.0),
        &vec![IDENTITY],
        1e-6
    ));
}

#[test]
fn b1_nan_weight_is_zero() {
    let got = blend(&vec![IDENTITY], &vec![q(90.0)], f64::NAN);
    assert!(pose_close(&got, &vec![IDENTITY], 1e-6));
}

#[test]
fn b1_result_has_the_length_of_a() {
    assert!(blend(&Vec::new(), &vec![q(90.0)], 0.5).is_empty());
    assert!(pose_close(
        &blend(&vec![IDENTITY, q(90.0)], &Vec::new(), 0.5),
        &vec![IDENTITY, q(90.0)],
        1e-6
    ));
}

#[test]
fn v1_valid_clip_is_ok() {
    assert!(validate(&wave(), 1).is_ok());
    assert!(validate(&still(), 2).is_ok());
}

#[test]
fn v1_bone_out_of_range() {
    assert!(matches!(
        validate(&wave(), 0),
        Err(ClipError::BoneOutOfRange { track: 0 })
    ));
}

#[test]
fn v1_bad_duration() {
    let mut clip = wave();
    clip.duration_s = 0.0;
    assert!(matches!(validate(&clip, 1), Err(ClipError::BadDuration)));
    clip.duration_s = f64::NAN;
    assert!(matches!(validate(&clip, 1), Err(ClipError::BadDuration)));
    clip.duration_s = f64::INFINITY;
    assert!(matches!(validate(&clip, 1), Err(ClipError::BadDuration)));
}

#[test]
fn v1_out_of_order_key() {
    let mut clip = wave();
    clip.tracks[0].keys = vec![
        Key {
            time_s: 0.0,
            rotation: IDENTITY,
        },
        Key {
            time_s: 0.0,
            rotation: q(90.0),
        },
    ];
    assert!(matches!(
        validate(&clip, 1),
        Err(ClipError::OutOfOrder { track: 0, key: 1 })
    ));
}

#[test]
fn v1_bad_key_time() {
    let mut clip = wave();
    clip.tracks[0].keys = vec![
        Key {
            time_s: 0.0,
            rotation: IDENTITY,
        },
        Key {
            time_s: 3.0,
            rotation: q(90.0),
        },
    ];
    assert!(matches!(
        validate(&clip, 1),
        Err(ClipError::BadTime { track: 0, key: 1 })
    ));
}

#[test]
fn v1_bad_rotation() {
    let mut clip = wave();
    clip.tracks[0].keys = vec![Key {
        time_s: 0.0,
        rotation: [0.0, 0.0, 0.0, 0.0],
    }];
    assert!(matches!(
        validate(&clip, 1),
        Err(ClipError::BadRotation { track: 0, key: 0 })
    ));
}

#[test]
fn v1_no_keys() {
    let mut clip = wave();
    clip.tracks[0].keys = Vec::new();
    assert!(matches!(
        validate(&clip, 1),
        Err(ClipError::NoKeys { track: 0 })
    ));
}

#[test]
fn v1_reports_the_first_problem_in_order() {
    // Bad duration outranks both an out-of-range bone and an empty track.
    let clip = Clip {
        name: "bad".to_string(),
        duration_s: 0.0,
        looping: false,
        tracks: vec![Track {
            bone: 9,
            keys: Vec::new(),
        }],
    };
    assert!(matches!(validate(&clip, 1), Err(ClipError::BadDuration)));
}

#[test]
fn k1_bakes_a_non_looping_clip() {
    let b = bake(&still(), 1, 4.0).expect("bake should succeed");
    assert!((b.fps - 4.0).abs() < 1e-12);
    assert!((b.duration_s - 2.0).abs() < 1e-12);
    assert!(!b.looping);
    assert!(b.frames.len() == 9);
    assert!(pose_close(&b.frames[2], &vec![q(45.0)], 1e-6));
    assert!(pose_close(&b.frames[8], &vec![IDENTITY], 1e-6));
}

#[test]
fn k1_bakes_a_looping_clip() {
    let b = bake(&wave(), 1, 4.0).expect("bake should succeed");
    assert!(b.looping);
    assert!(b.frames.len() == 8);
    assert!(pose_close(&b.frames[4], &vec![q(90.0)], 1e-6));
    assert!(pose_close(&b.frames[0], &vec![IDENTITY], 1e-6));
}

#[test]
fn k1_bakes_at_twelve_fps() {
    let b = bake(&wave(), 1, 12.0).expect("bake should succeed");
    assert!(b.frames.len() == 24);
}

#[test]
fn k1_rejects_bad_fps_or_duration() {
    assert!(bake(&wave(), 1, 0.0).is_none());
    assert!(bake(&wave(), 1, f64::NAN).is_none());
    assert!(bake(&wave(), 1, f64::INFINITY).is_none());
    let mut clip = wave();
    clip.duration_s = 0.0;
    assert!(bake(&clip, 1, 4.0).is_none());
}

#[test]
fn k1_looping_bakes_at_least_one_frame() {
    let mut clip = wave();
    clip.duration_s = 0.01;
    let b = bake(&clip, 1, 1.0).expect("bake should succeed");
    assert!(b.frames.len() == 1);
}

#[test]
fn k1_non_looping_bakes_k_plus_one_frames() {
    let mut clip = still();
    clip.duration_s = 0.01;
    let b = bake(&clip, 1, 1.0).expect("bake should succeed");
    assert!(b.frames.len() == 1);
}

#[test]
fn k2_frame_index_floors_the_time() {
    let b = bake(&still(), 1, 4.0).expect("bake should succeed");
    assert!(frame_index(&b, 0.3) == 1);
}

#[test]
fn k2_time_past_the_end_clamps_to_the_last_frame() {
    let b = bake(&still(), 1, 4.0).expect("bake should succeed");
    assert!(frame_index(&b, 5.0) == 8);
}

#[test]
fn k2_looping_time_wraps() {
    let b = bake(&wave(), 1, 4.0).expect("bake should succeed");
    assert!(frame_index(&b, 2.1) == 0);
}

#[test]
fn k2_just_before_the_end_is_the_last_frame() {
    let b = bake(&wave(), 1, 4.0).expect("bake should succeed");
    assert!(frame_index(&b, 1.99) == 7);
}

#[test]
fn k2_no_frames_is_zero() {
    let b = Baked {
        fps: 4.0,
        duration_s: 2.0,
        looping: true,
        frames: Vec::new(),
    };
    assert!(frame_index(&b, 1.0) == 0);
    assert!(frame_index(&b, f64::NAN) == 0);
}

#[test]
fn odd_inputs_never_panic() {
    let huge = 1e300;
    let nan = f64::NAN;
    let inf = f64::INFINITY;
    assert!(normalize([nan, huge, -inf, 0.0]).is_none());
    assert!(close(
        nlerp([nan, 0.0, 0.0, 1.0], [inf, 0.0, 0.0, 1.0], inf),
        IDENTITY,
        1e-12
    ));
    assert!(close(nlerp(IDENTITY, q(90.0), -inf), IDENTITY, 1e-12));

    let mut broken_clip = wave();
    broken_clip.duration_s = nan;
    assert!(sample(&broken_clip, 3, huge).len() == 3);
    assert!(sample(&wave(), 2, inf).len() == 2);
    assert!(sample(&wave(), 2, huge).len() == 2);

    let empty = Clip {
        name: String::new(),
        duration_s: 1.0,
        looping: true,
        tracks: Vec::new(),
    };
    assert!(sample(&empty, 4, 0.5) == vec![IDENTITY; 4]);
    assert!(bake(&wave(), 1, inf).is_none());
    assert!(bake(&wave(), 1, nan).is_none());

    let b = bake(&wave(), 1, 4.0).expect("bake should succeed");
    assert!(frame_index(&b, huge) < b.frames.len());
    assert!(frame_index(&b, inf) < b.frames.len());
    assert!(frame_index(&b, -inf) < b.frames.len());

    let mut broken = wave();
    broken.tracks[0].keys = vec![Key {
        time_s: nan,
        rotation: [nan; 4],
    }];
    assert!(matches!(
        validate(&broken, 1),
        Err(ClipError::BadTime { track: 0, key: 0 })
    ));
    assert!(close(sample_track(&broken.tracks[0], inf), IDENTITY, 1e-12));
    assert!(!blend(&vec![[nan; 4]], &vec![IDENTITY], inf).is_empty());
}
