//! Tests added by Claude after mutation testing of the Crew's clips module (campaign mk-core-22):
//! the Crew's tests only turn about z, and none samples a track at its exact last key with a
//! rotation that is not the identity. Each case below failed against a seeded defect.

use mk_sancho::clips::{nlerp, sample_track, Key, Track, IDENTITY};

/// The turn by `deg` degrees about one axis (0 = x, 1 = y, 2 = z) as [x, y, z, w].
fn turn(axis: usize, deg: f64) -> [f64; 4] {
    let half = deg.to_radians() / 2.0;
    let mut q = [0.0; 4];
    q[axis] = half.sin();
    q[3] = half.cos();
    q
}

fn close(a: [f64; 4], b: [f64; 4]) {
    for i in 0..4 {
        assert!((a[i] - b[i]).abs() < 1e-6, "{a:?} is not {b:?}");
    }
}

#[test]
fn nlerp_blends_every_axis_the_same_way() {
    for axis in 0..3 {
        close(nlerp(IDENTITY, turn(axis, 90.0), 0.5), turn(axis, 45.0));
    }
}

#[test]
fn the_shorter_way_round_reads_the_x_component_too() {
    // 200 and 150 degrees about x are 50 degrees apart: no flip, and the middle is 175 degrees.
    // (Their w terms alone have opposite signs, so a dot product that skipped x would flip.)
    for axis in 0..3 {
        close(
            nlerp(turn(axis, 200.0), turn(axis, 150.0), 0.5),
            turn(axis, 175.0),
        );
    }
}

#[test]
fn a_track_sampled_exactly_at_its_last_key_gives_that_key() {
    let track = Track {
        bone: 0,
        keys: vec![
            Key {
                time_s: 0.0,
                rotation: IDENTITY,
            },
            Key {
                time_s: 1.0,
                rotation: turn(2, 90.0),
            },
        ],
    };
    close(sample_track(&track, 1.0), turn(2, 90.0));
    close(sample_track(&track, 0.0), IDENTITY);
}
