//! Sancho's animation clips (doc 07 section 7.3.1): sampling, blending and baking.
//!
//! Written by Hoss (the Crew, campaign mk-core-22, 2026-11-22) from the contract
//! crew/contracts/mk_clips.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

/// A rotation as a quaternion [x, y, z, w] (glTF's order).
pub type Quat = [f64; 4];

/// One rotation per bone, by bone index.
pub type Pose = Vec<Quat>;

/// The rotation that turns nothing.
pub const IDENTITY: Quat = [0.0, 0.0, 0.0, 1.0];

/// One key: a time in seconds and the bone's rotation then.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub time_s: f64,
    pub rotation: Quat,
}

/// The keys of one bone, in time order.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub bone: usize,
    pub keys: Vec<Key>,
}

/// A clip: its name (the persona file maps names to events), length, and whether it loops.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub name: String,
    pub duration_s: f64,
    pub looping: bool,
    pub tracks: Vec<Track>,
}

/// A clip baked to whole frames at `fps`.
#[derive(Debug, Clone, PartialEq)]
pub struct Baked {
    pub fps: f64,
    pub duration_s: f64,
    pub looping: bool,
    pub frames: Vec<Pose>,
}

/// Why a clip cannot be played; track and key are indexes into the clip's lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipError {
    BadDuration,
    BoneOutOfRange { track: usize },
    NoKeys { track: usize },
    BadTime { track: usize, key: usize },
    OutOfOrder { track: usize, key: usize },
    BadRotation { track: usize, key: usize },
}

/// Normalize a quaternion. Returns `None` if length is not finite or < 1e-12.
pub fn normalize(q: Quat) -> Option<Quat> {
    let len_sq = q.iter().map(|v| v * v).sum::<f64>();
    if !len_sq.is_finite() {
        return None;
    }
    let len = len_sq.sqrt();
    if len < 1e-12 {
        return None;
    }
    Some([q[0] / len, q[1] / len, q[2] / len, q[3] / len])
}

/// Normalized linear interpolation between two quaternions.
pub fn nlerp(a: Quat, b: Quat, t: f64) -> Quat {
    // Q2: normalize inputs, fallback to IDENTITY if impossible.
    let a_n = normalize(a).unwrap_or(IDENTITY);
    let mut b_n = normalize(b).unwrap_or(IDENTITY);

    // Clamp t to [0,1], NaN => 0.
    let t_clamped = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };

    // Shorter-way check.
    let dot = a_n[0] * b_n[0] + a_n[1] * b_n[1] + a_n[2] * b_n[2] + a_n[3] * b_n[3];
    if dot < 0.0 {
        b_n = [-b_n[0], -b_n[1], -b_n[2], -b_n[3]];
    }

    // Linear blend.
    let blended = [
        a_n[0] * (1.0 - t_clamped) + b_n[0] * t_clamped,
        a_n[1] * (1.0 - t_clamped) + b_n[1] * t_clamped,
        a_n[2] * (1.0 - t_clamped) + b_n[2] * t_clamped,
        a_n[3] * (1.0 - t_clamped) + b_n[3] * t_clamped,
    ];

    // Normalize result, fallback to a_n.
    normalize(blended).unwrap_or(a_n)
}

/// Sample a single track at a given time.
pub fn sample_track(track: &Track, time_s: f64) -> Quat {
    // T1: NaN time => 0.
    let time = if time_s.is_nan() { 0.0 } else { time_s };

    if track.keys.is_empty() {
        return IDENTITY;
    }

    // Before first key.
    if time <= track.keys[0].time_s {
        return normalize(track.keys[0].rotation).unwrap_or(IDENTITY);
    }

    // After last key.
    let last_idx = track.keys.len() - 1;
    if time >= track.keys[last_idx].time_s {
        return normalize(track.keys[last_idx].rotation).unwrap_or(IDENTITY);
    }

    // Find interval.
    for i in 0..last_idx {
        let cur = &track.keys[i];
        let next = &track.keys[i + 1];
        if time >= cur.time_s && time < next.time_s {
            let span = next.time_s - cur.time_s;
            let t = if span == 0.0 {
                0.0
            } else {
                (time - cur.time_s) / span
            };
            return nlerp(cur.rotation, next.rotation, t);
        }
    }

    // Should not reach here.
    IDENTITY
}

/// Compute the effective time inside a clip.
pub fn clip_time(duration_s: f64, looping: bool, time_s: f64) -> f64 {
    // C1: bad duration => 0.
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return 0.0;
    }
    // NaN or infinite time => 0.
    if !time_s.is_finite() {
        return 0.0;
    }
    if looping {
        // rem_euclid works for negative values as required.
        time_s.rem_euclid(duration_s)
    } else {
        time_s.max(0.0).min(duration_s)
    }
}

/// Sample a whole clip, producing a pose for `bones` bones.
pub fn sample(clip: &Clip, bones: usize, time_s: f64) -> Pose {
    let mut pose = vec![IDENTITY; bones];
    let mut set = vec![false; bones];
    let t = clip_time(clip.duration_s, clip.looping, time_s);

    for track in &clip.tracks {
        if track.bone < bones && !set[track.bone] {
            pose[track.bone] = sample_track(track, t);
            set[track.bone] = true;
        }
    }
    pose
}

/// Blend two poses with a weight.
pub fn blend(a: &Pose, b: &Pose, weight: f64) -> Pose {
    // B1: clamp weight.
    let w = if weight.is_nan() {
        0.0
    } else {
        weight.clamp(0.0, 1.0)
    };
    let len = a.len();
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let a_q = a[i];
        let b_q = if i < b.len() { b[i] } else { a_q };
        out.push(nlerp(a_q, b_q, w));
    }
    out
}

/// Validate a clip against the given bone count.
pub fn validate(clip: &Clip, bones: usize) -> Result<(), ClipError> {
    // V1: duration check.
    if !clip.duration_s.is_finite() || clip.duration_s <= 0.0 {
        return Err(ClipError::BadDuration);
    }

    for (track_idx, track) in clip.tracks.iter().enumerate() {
        if track.bone >= bones {
            return Err(ClipError::BoneOutOfRange { track: track_idx });
        }
        if track.keys.is_empty() {
            return Err(ClipError::NoKeys { track: track_idx });
        }
        let mut prev_time = -1.0_f64;
        for (key_idx, key) in track.keys.iter().enumerate() {
            // BadTime.
            if !key.time_s.is_finite() || key.time_s < 0.0 || key.time_s > clip.duration_s {
                return Err(ClipError::BadTime {
                    track: track_idx,
                    key: key_idx,
                });
            }
            // OutOfOrder.
            if key_idx > 0 && key.time_s <= prev_time {
                return Err(ClipError::OutOfOrder {
                    track: track_idx,
                    key: key_idx,
                });
            }
            prev_time = key.time_s;
            // BadRotation.
            if normalize(key.rotation).is_none() {
                return Err(ClipError::BadRotation {
                    track: track_idx,
                    key: key_idx,
                });
            }
        }
    }
    Ok(())
}

/// Bake a clip into whole frames.
pub fn bake(clip: &Clip, bones: usize, fps: f64) -> Option<Baked> {
    // K1: check fps and duration.
    if !fps.is_finite() || fps <= 0.0 || !clip.duration_s.is_finite() || clip.duration_s <= 0.0 {
        return None;
    }
    let k_f = (clip.duration_s * fps + 1e-9).floor();
    let k = k_f as usize;
    let frame_count = if clip.looping {
        std::cmp::max(k, 1)
    } else {
        k + 1
    };
    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let time = i as f64 / fps;
        frames.push(sample(clip, bones, time));
    }
    Some(Baked {
        fps,
        duration_s: clip.duration_s,
        looping: clip.looping,
        frames,
    })
}

/// Compute the frame index for a given time in a baked clip.
pub fn frame_index(baked: &Baked, time_s: f64) -> usize {
    // K2.
    if baked.frames.is_empty() {
        return 0;
    }
    let t = clip_time(baked.duration_s, baked.looping, time_s);
    let i_f = (t * baked.fps + 1e-9).floor();
    let i = i_f as usize;
    std::cmp::min(i, baked.frames.len() - 1)
}
