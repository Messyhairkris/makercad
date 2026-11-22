//! Tests for mk_sancho::clips written by Deepak (the Crew, campaign mk-core-22) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_sancho::clips` (contract v1): sampling, blending, baking.

// The worked rows write 0.7071... and 1.4142... to the digits the contract prints; they are
// compared within a tolerance on purpose.
#![allow(clippy::approx_constant)]

use mk_sancho::clips::{
    bake, blend, clip_time, frame_index, nlerp, normalize, sample, sample_track, validate, Baked,
    Clip, ClipError, Key, Pose, Quat, Track, IDENTITY,
};

/// The turn by `deg` degrees about +z: [0, 0, sin(a/2), cos(a/2)].
fn q(deg: f64) -> Quat {
    let half = deg.to_radians() / 2.0;
    [0.0, 0.0, half.sin(), half.cos()]
}

fn assert_close(got: f64, want: f64, tol: f64) {
    assert!(
        (got - want).abs() <= tol,
        "got {got}, want {want} (tol {tol})"
    );
}

fn assert_quat(got: Quat, want: Quat, tol: f64) {
    for i in 0..4 {
        assert_close(got[i], want[i], tol);
    }
}

fn assert_pose(got: &Pose, want: &[Quat], tol: f64) {
    assert_eq!(
        got.len(),
        want.len(),
        "pose length: got {got:?}, want {want:?}"
    );
    for (i, w) in want.iter().enumerate() {
        assert_quat(got[i], *w, tol);
    }
}

/// Track T from the worked examples: bone 0, keys (0, I), (1, q(90)), (2, I).
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

fn clip_wave() -> Clip {
    Clip {
        name: "wave".to_string(),
        duration_s: 2.0,
        looping: true,
        tracks: vec![track_t()],
    }
}

fn clip_wave_hold() -> Clip {
    Clip {
        looping: false,
        ..clip_wave()
    }
}

fn clip_with(tracks: Vec<Track>) -> Clip {
    Clip {
        tracks,
        ..clip_wave()
    }
}

fn clip_with_duration(duration_s: f64) -> Clip {
    Clip {
        duration_s,
        ..clip_wave()
    }
}

fn clip_with_keys(keys: Vec<Key>) -> Clip {
    clip_with(vec![Track { bone: 0, keys }])
}

// ---- Q1 normalize ----

#[test]
fn q1_normalize_scales_to_unit_length() {
    assert_quat(normalize([0.0, 0.0, 0.0, 2.0]).unwrap(), IDENTITY, 1e-9);
    // [0, 0, 3, 4] has length 5, so it becomes [0, 0, 0.6, 0.8].
    assert_quat(
        normalize([0.0, 0.0, 3.0, 4.0]).unwrap(),
        [0.0, 0.0, 0.6, 0.8],
        1e-9,
    );
}

#[test]
fn q1_normalize_rejects_zero_length() {
    assert_eq!(normalize([0.0, 0.0, 0.0, 0.0]), None);
    // 1e-13 is below the 1e-12 floor.
    assert_eq!(normalize([1e-13, 0.0, 0.0, 0.0]), None);
}

#[test]
fn q1_normalize_rejects_non_finite_length() {
    assert_eq!(normalize([f64::NAN, 0.0, 0.0, 1.0]), None);
    assert_eq!(normalize([f64::INFINITY, 0.0, 0.0, 1.0]), None);
    assert_eq!(normalize([0.0, 0.0, 0.0, f64::NEG_INFINITY]), None);
}

// ---- Q2 nlerp ----

#[test]
fn q2_nlerp_halfway_between_identity_and_q90_is_q45() {
    assert_quat(nlerp(IDENTITY, q(90.0), 0.5), q(45.0), 1e-6);
}

#[test]
fn q2_nlerp_quarter_is_not_slerp() {
    // 0.75*I + 0.25*q(90) = [0, 0, 0.176777, 0.926777], length 0.943486,
    // normalized to [0, 0, 0.18737, 0.98230] - not q(22.5) = [0, 0, 0.19509, 0.98079].
    assert_quat(
        nlerp(IDENTITY, q(90.0), 0.25),
        [0.0, 0.0, 0.1874, 0.9823],
        1e-4,
    );
}

#[test]
fn q2_nlerp_takes_the_shorter_way_round() {
    let negated = [0.0, 0.0, -0.707107, -0.707107];
    assert_quat(nlerp(IDENTITY, negated, 0.5), q(45.0), 1e-6);
}

#[test]
fn q2_nlerp_clamps_the_weight() {
    assert_quat(nlerp(IDENTITY, q(90.0), 7.0), q(90.0), 1e-6);
    assert_quat(nlerp(IDENTITY, q(90.0), -3.0), IDENTITY, 1e-6);
}

#[test]
fn q2_nlerp_nan_weight_counts_as_zero() {
    assert_quat(nlerp(IDENTITY, q(90.0), f64::NAN), IDENTITY, 1e-6);
}

#[test]
fn q2_nlerp_normalizes_its_inputs_first() {
    // b is q(90) scaled by 3; normalizing it first must still give q(45) at t = 0.5.
    assert_quat(
        nlerp(IDENTITY, [0.0, 0.0, 2.121320, 2.121320], 0.5),
        q(45.0),
        1e-6,
    );
}

// ---- T1 sample_track ----

#[test]
fn t1_sample_track_at_or_before_first_key() {
    assert_quat(sample_track(&track_t(), -1.0), IDENTITY, 1e-6);
    assert_quat(sample_track(&track_t(), 0.0), IDENTITY, 1e-6);
}

#[test]
fn t1_sample_track_between_keys() {
    assert_quat(sample_track(&track_t(), 0.5), q(45.0), 1e-6);
    assert_quat(sample_track(&track_t(), 1.5), q(45.0), 1e-6);
}

#[test]
fn t1_sample_track_at_key_times() {
    assert_quat(sample_track(&track_t(), 1.0), q(90.0), 1e-6);
    assert_quat(sample_track(&track_t(), 2.0), IDENTITY, 1e-6);
}

#[test]
fn t1_sample_track_at_or_after_last_key() {
    assert_quat(sample_track(&track_t(), 9.0), IDENTITY, 1e-6);
}

#[test]
fn t1_sample_track_with_no_keys_is_identity() {
    let empty = Track {
        bone: 0,
        keys: vec![],
    };
    assert_quat(sample_track(&empty, 0.5), IDENTITY, 1e-6);
}

#[test]
fn t1_sample_track_nan_time_counts_as_zero() {
    assert_quat(sample_track(&track_t(), f64::NAN), IDENTITY, 1e-6);
}

#[test]
fn t1_sample_track_normalizes_the_key_rotation() {
    // The first key's rotation is q(90) scaled by 2; the answer is still q(90).
    let track = Track {
        bone: 0,
        keys: vec![Key {
            time_s: 0.0,
            rotation: [0.0, 0.0, 1.414214, 1.414214],
        }],
    };
    assert_quat(sample_track(&track, 0.0), q(90.0), 1e-6);
}

// ---- C1 clip_time ----

#[test]
fn c1_clip_time_looping_wraps() {
    assert_close(clip_time(2.0, true, 2.5), 0.5, 1e-9);
    assert_close(clip_time(2.0, true, -0.5), 1.5, 1e-9);
    assert_close(clip_time(2.0, true, 2.0), 0.0, 1e-9);
}

#[test]
fn c1_clip_time_not_looping_clamps() {
    assert_close(clip_time(2.0, false, 3.0), 2.0, 1e-9);
    assert_close(clip_time(2.0, false, -1.0), 0.0, 1e-9);
    assert_close(clip_time(2.0, false, 1.25), 1.25, 1e-9);
}

#[test]
fn c1_clip_time_bad_duration_gives_zero() {
    assert_close(clip_time(0.0, true, 1.0), 0.0, 1e-9);
    assert_close(clip_time(-2.0, false, 1.0), 0.0, 1e-9);
    assert_close(clip_time(f64::NAN, true, 1.0), 0.0, 1e-9);
    assert_close(clip_time(f64::INFINITY, true, 1.0), 0.0, 1e-9);
}

#[test]
fn c1_clip_time_non_finite_time_counts_as_zero() {
    assert_close(clip_time(2.0, true, f64::NAN), 0.0, 1e-9);
    assert_close(clip_time(2.0, true, f64::INFINITY), 0.0, 1e-9);
    assert_close(clip_time(2.0, false, f64::NEG_INFINITY), 0.0, 1e-9);
}

// ---- C2 sample ----

#[test]
fn c2_sample_looping_wraps_the_time() {
    assert_pose(&sample(&clip_wave(), 2, 2.5), &[q(45.0), IDENTITY], 1e-6);
}

#[test]
fn c2_sample_not_looping_holds_the_last_key() {
    assert_pose(
        &sample(&clip_wave_hold(), 2, 3.0),
        &[IDENTITY, IDENTITY],
        1e-6,
    );
}

#[test]
fn c2_sample_with_no_bones_is_an_empty_pose() {
    assert_eq!(sample(&clip_wave(), 0, 1.0), Vec::<Quat>::new());
}

#[test]
fn c2_sample_first_track_for_a_bone_wins() {
    let second = Track {
        bone: 0,
        keys: vec![Key {
            time_s: 0.0,
            rotation: q(90.0),
        }],
    };
    let clip = clip_with(vec![track_t(), second]);
    assert_pose(&sample(&clip, 2, 0.0), &[IDENTITY, IDENTITY], 1e-6);
}

#[test]
fn c2_sample_ignores_tracks_for_other_bones() {
    let other = Track {
        bone: 5,
        keys: vec![Key {
            time_s: 0.0,
            rotation: q(90.0),
        }],
    };
    let clip = clip_with(vec![other]);
    assert_pose(&sample(&clip, 2, 0.0), &[IDENTITY, IDENTITY], 1e-6);
}

// ---- B1 blend ----

#[test]
fn b1_blend_halfway() {
    assert_pose(
        &blend(&vec![IDENTITY], &vec![q(90.0)], 0.5),
        &[q(45.0)],
        1e-6,
    );
}

#[test]
fn b1_blend_keeps_a_bones_that_b_does_not_have() {
    assert_pose(
        &blend(&vec![IDENTITY, q(90.0)], &vec![q(90.0)], 1.0),
        &[q(90.0), q(90.0)],
        1e-6,
    );
}

#[test]
fn b1_blend_clamps_the_weight() {
    assert_pose(
        &blend(&vec![IDENTITY], &vec![q(90.0)], 2.0),
        &[q(90.0)],
        1e-6,
    );
    assert_pose(
        &blend(&vec![IDENTITY], &vec![q(90.0)], -1.0),
        &[IDENTITY],
        1e-6,
    );
    assert_pose(
        &blend(&vec![IDENTITY], &vec![q(90.0)], f64::NAN),
        &[IDENTITY],
        1e-6,
    );
}

// ---- V1 validate ----

#[test]
fn v1_validate_accepts_a_good_clip() {
    assert_eq!(validate(&clip_wave(), 1), Ok(()));
    // A key exactly at the duration is allowed (only > duration is BadTime).
    assert_eq!(validate(&clip_wave(), 1), Ok(()));
}

#[test]
fn v1_validate_bone_out_of_range() {
    assert_eq!(
        validate(&clip_wave(), 0),
        Err(ClipError::BoneOutOfRange { track: 0 })
    );
}

#[test]
fn v1_validate_bad_duration_comes_first() {
    assert_eq!(
        validate(&clip_with_duration(0.0), 1),
        Err(ClipError::BadDuration)
    );
    assert_eq!(
        validate(&clip_with_duration(f64::NAN), 1),
        Err(ClipError::BadDuration)
    );
    // BadDuration is reported before the bone check.
    assert_eq!(
        validate(&clip_with_duration(0.0), 0),
        Err(ClipError::BadDuration)
    );
}

#[test]
fn v1_validate_out_of_order() {
    let keys = vec![
        Key {
            time_s: 0.0,
            rotation: IDENTITY,
        },
        Key {
            time_s: 0.0,
            rotation: q(90.0),
        },
    ];
    assert_eq!(
        validate(&clip_with_keys(keys), 1),
        Err(ClipError::OutOfOrder { track: 0, key: 1 })
    );
}

#[test]
fn v1_validate_bad_time() {
    let keys = vec![
        Key {
            time_s: 0.0,
            rotation: IDENTITY,
        },
        Key {
            time_s: 3.0,
            rotation: q(90.0),
        },
    ];
    assert_eq!(
        validate(&clip_with_keys(keys), 1),
        Err(ClipError::BadTime { track: 0, key: 1 })
    );
    let negative = vec![Key {
        time_s: -0.5,
        rotation: IDENTITY,
    }];
    assert_eq!(
        validate(&clip_with_keys(negative), 1),
        Err(ClipError::BadTime { track: 0, key: 0 })
    );
}

#[test]
fn v1_validate_bad_rotation() {
    let keys = vec![Key {
        time_s: 0.0,
        rotation: [0.0, 0.0, 0.0, 0.0],
    }];
    assert_eq!(
        validate(&clip_with_keys(keys), 1),
        Err(ClipError::BadRotation { track: 0, key: 0 })
    );
}

#[test]
fn v1_validate_no_keys() {
    let clip = clip_with(vec![Track {
        bone: 0,
        keys: vec![],
    }]);
    assert_eq!(validate(&clip, 1), Err(ClipError::NoKeys { track: 0 }));
}

// ---- K1 bake ----

#[test]
fn k1_bake_not_looping_has_k_plus_one_frames() {
    let baked = bake(&clip_wave_hold(), 1, 4.0).unwrap();
    assert_eq!(baked.frames.len(), 9);
    assert_close(baked.fps, 4.0, 1e-9);
    assert_close(baked.duration_s, 2.0, 1e-9);
    assert!(!baked.looping);
    // Frame 2 is at 0.5 s, halfway up the first ramp.
    assert_pose(&baked.frames[2], &[q(45.0)], 1e-6);
    assert_pose(&baked.frames[0], &[IDENTITY], 1e-6);
    assert_pose(&baked.frames[8], &[IDENTITY], 1e-6);
}

#[test]
fn k1_bake_looping_does_not_repeat_the_end() {
    let baked = bake(&clip_wave(), 1, 4.0).unwrap();
    assert_eq!(baked.frames.len(), 8);
    assert!(baked.looping);
    // Frame 4 is at 1.0 s, the top of the wave.
    assert_pose(&baked.frames[4], &[q(90.0)], 1e-6);
    assert_pose(&baked.frames[0], &[IDENTITY], 1e-6);
}

#[test]
fn k1_bake_at_twelve_fps() {
    let baked = bake(&clip_wave(), 1, 12.0).unwrap();
    assert_eq!(baked.frames.len(), 24);
    // Frame 6 is at 0.5 s.
    assert_pose(&baked.frames[6], &[q(45.0)], 1e-6);
}

#[test]
fn k1_bake_rejects_bad_fps_and_duration() {
    assert_eq!(bake(&clip_wave(), 1, 0.0), None);
    assert_eq!(bake(&clip_wave(), 1, -4.0), None);
    assert_eq!(bake(&clip_wave(), 1, f64::NAN), None);
    assert_eq!(bake(&clip_wave(), 1, f64::INFINITY), None);
    assert_eq!(bake(&clip_with_duration(f64::NAN), 1, 4.0), None);
    assert_eq!(bake(&clip_with_duration(f64::INFINITY), 1, 4.0), None);
}

// ---- K2 frame_index ----

#[test]
fn k2_frame_index_rounds_down() {
    let baked = bake(&clip_wave_hold(), 1, 4.0).unwrap();
    assert_eq!(frame_index(&baked, 0.3), 1);
    assert_eq!(frame_index(&baked, 0.0), 0);
    assert_eq!(frame_index(&baked, 0.25), 1);
}

#[test]
fn k2_frame_index_clamps_to_the_last_frame() {
    let baked = bake(&clip_wave_hold(), 1, 4.0).unwrap();
    assert_eq!(frame_index(&baked, 5.0), 8);
}

#[test]
fn k2_frame_index_looping_wraps() {
    let baked = bake(&clip_wave(), 1, 4.0).unwrap();
    assert_eq!(frame_index(&baked, 2.1), 0);
    assert_eq!(frame_index(&baked, 1.99), 7);
}

#[test]
fn k2_frame_index_with_no_frames_is_zero() {
    let baked = Baked {
        fps: 4.0,
        duration_s: 2.0,
        looping: false,
        frames: vec![],
    };
    assert_eq!(frame_index(&baked, 1.0), 0);
}

// ---- no panics ----

#[test]
fn no_input_panics() {
    let empty = Clip {
        name: "empty".to_string(),
        duration_s: 1.0,
        looping: true,
        tracks: vec![],
    };
    assert_pose(
        &sample(&empty, 3, 0.5),
        &[IDENTITY, IDENTITY, IDENTITY],
        1e-6,
    );
    assert_pose(
        &sample(&clip_wave(), 2, f64::NAN),
        &[IDENTITY, IDENTITY],
        1e-6,
    );
    assert_pose(
        &sample(&clip_wave(), 2, f64::INFINITY),
        &[IDENTITY, IDENTITY],
        1e-6,
    );
    assert_pose(&sample(&clip_wave(), 2, 1e300), &[IDENTITY, IDENTITY], 1e-6);
    assert_pose(&blend(&vec![], &vec![q(90.0)], f64::NAN), &[], 1e-6);
    assert_quat(nlerp([0.0, 0.0, 0.0, 0.0], q(90.0), 0.5), q(45.0), 1e-6);
    let baked = bake(&empty, 2, 4.0).unwrap();
    assert_eq!(baked.frames.len(), 4);
    assert_eq!(frame_index(&baked, f64::NAN), 0);
    assert_eq!(frame_index(&baked, f64::INFINITY), 0);
}
