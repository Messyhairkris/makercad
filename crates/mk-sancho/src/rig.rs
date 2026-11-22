//! Sancho's light rig math (doc 07 section 7.3.1): look-at, point-at, crossfades, the event
//!
//! Written by Deepak (the Crew, campaign mk-core-14, 2026-11-22) from the contract
//! crew/contracts/mk_rig.md (ATLAS-api-harness), which Claude finished from Gordon's brief.
//! table, what each hardware tier runs, walking, and height.

use std::f64::consts::{PI, TAU};

/// A point or direction in metres.
pub type Vec3 = [f64; 3];

/// Sancho's height when nothing else is asked for, in metres.
pub const DEFAULT_HEIGHT_M: f64 = 1.68;
/// The shortest Sancho may be scaled to, in metres.
pub const MIN_HEIGHT_M: f64 = 0.5;
/// The tallest Sancho may be scaled to, in metres.
pub const MAX_HEIGHT_M: f64 = 2.5;
/// The crossfade used when a caller asks for something unusable, in seconds.
pub const CROSSFADE_S: f64 = 0.25;
/// The shortest crossfade allowed, in seconds.
pub const CROSSFADE_MIN_S: f64 = 0.2;
/// The longest crossfade allowed, in seconds.
pub const CROSSFADE_MAX_S: f64 = 0.3;

/// How far the head and neck together may turn, in radians (all three are magnitudes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// The most the head and neck together may yaw, either side.
    pub yaw_max: f64,
    /// The most they may pitch up.
    pub pitch_up: f64,
    /// The most they may pitch down.
    pub pitch_down: f64,
}

impl Limits {
    /// 70 degrees either side, 30 up, 40 down: v1 starting values, to tune by eye.
    pub const DEFAULT: Limits = Limits {
        yaw_max: 1.2217304763960306,
        pitch_up: 0.5235987755982988,
        pitch_down: 0.6981317007977318,
    };
}

/// A look-at answer: the whole turn and its split between neck and head.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// The whole turn about +z, positive to his left.
    pub yaw: f64,
    /// The whole pitch, positive looking up.
    pub pitch: f64,
    /// The neck's share of the yaw.
    pub neck_yaw: f64,
    /// The neck's share of the pitch.
    pub neck_pitch: f64,
    /// The head's share of the yaw.
    pub head_yaw: f64,
    /// The head's share of the pitch.
    pub head_pitch: f64,
    /// True when a limit cut the raw turn down.
    pub clamped: bool,
}

/// A point-at answer: where the elbow and hand go.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arm {
    /// The elbow's position.
    pub elbow: Vec3,
    /// The hand's position.
    pub hand: Vec3,
    /// The inside angle at the elbow: pi is a straight arm, 0 fully folded.
    pub elbow_angle: f64,
    /// The hand is on the target (false: he points as far as he can).
    pub reached: bool,
}

/// Why a point-at could not be solved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IkError {
    /// An arm bone's length is not a positive, finite number.
    BadLength,
    /// The target is not a finite point away from the shoulder.
    NoDirection,
    /// The pole lies along the arm, so the elbow has no side to bend toward.
    PoleAlongArm,
}

/// Sancho's clips (doc 07 section 7.3.1's clip set), in the doc's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Clip {
    /// Standing idle.
    Idle,
    /// A greeting wave.
    Wave,
    /// Pointing at something.
    Point,
    /// Thinking it over.
    Think,
    /// Worried about something.
    Worried,
    /// Celebrating a milestone.
    Celebrate,
    /// Walking to a target.
    WalkTo,
    /// Sitting down.
    Sit,
    /// Carrying the board.
    CarryBoard,
}

/// Every clip, in the doc's order.
pub const CLIPS: [Clip; 9] = [
    Clip::Idle,
    Clip::Wave,
    Clip::Point,
    Clip::Think,
    Clip::Worried,
    Clip::Celebrate,
    Clip::WalkTo,
    Clip::Sit,
    Clip::CarryBoard,
];

/// What happened that Sancho reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    /// Nothing in particular.
    Idle,
    /// Someone arrived.
    Greet,
    /// A finding was raised.
    FindingRaised,
    /// An unsafe condition was raised.
    UnsafeRaised,
    /// A milestone was reached.
    Milestone,
    /// He was asked something.
    Asked,
    /// He is walking to a target.
    WalkingToTarget,
}

/// The hardware tiers of doc 07's table (the probe picks one; Muted is the user's switch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// A capable machine.
    Capable,
    /// A modest machine.
    Modest,
    /// A potato.
    Potato,
    /// The user muted him.
    Muted,
}

/// What a tier runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caps {
    /// Whether skinning runs.
    pub skinning: bool,
    /// Whether clips run.
    pub clips: bool,
    /// Whether crossfades run.
    pub crossfade: bool,
    /// Whether the look-at runs.
    pub look_at: bool,
    /// Whether the point-at runs.
    pub point_at: bool,
    /// Whether animation is baked.
    pub baked: bool,
    /// The frame rate cap, when there is one.
    pub frame_rate: Option<f64>,
}

/// A walk: turn in place by `turn`, then go `distance` straight ahead, ending facing `heading`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walk {
    /// The turn in place, positive to his left.
    pub turn: f64,
    /// The straight-line distance to travel.
    pub distance: f64,
    /// The heading he ends up facing.
    pub heading: f64,
}

impl Clip {
    /// The persona file's name for this clip.
    pub fn key(self) -> &'static str {
        match self {
            Clip::Idle => "idle",
            Clip::Wave => "wave",
            Clip::Point => "point",
            Clip::Think => "think",
            Clip::Worried => "worried",
            Clip::Celebrate => "celebrate",
            Clip::WalkTo => "walk_to",
            Clip::Sit => "sit",
            Clip::CarryBoard => "carry_board",
        }
    }
}

/// True when all three components are finite.
fn finite3(v: Vec3) -> bool {
    v[0].is_finite() && v[1].is_finite() && v[2].is_finite()
}

/// The length of a 3-vector, without overflowing on large components.
fn length3(v: Vec3) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}

/// Clamp `x` into `[lo, hi]` with comparisons: `f64::clamp` panics on bad bounds.
fn clamp_range(x: f64, lo: f64, hi: f64) -> f64 {
    let mut v = x;
    if v < lo {
        v = lo;
    }
    if v > hi {
        v = hi;
    }
    v
}

/// A limit as a magnitude: negative or not finite counts as 0.
fn magnitude(limit: f64) -> f64 {
    if limit.is_finite() && limit > 0.0 {
        limit
    } else {
        0.0
    }
}

/// The neck's share: clamped to [0, 1], with NaN giving 0.5.
fn neck_share_of(neck_share: f64) -> f64 {
    if neck_share.is_nan() {
        0.5
    } else {
        clamp_range(neck_share, 0.0, 1.0)
    }
}

/// L1 and L2: the whole turn from eye to target, split between neck and head.
pub fn look_at(eye: Vec3, target: Vec3, limits: Limits, neck_share: f64) -> Option<Look> {
    let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    if !finite3(d) || length3(d) < 1e-9 {
        return None;
    }
    let raw_yaw = (-d[0]).atan2(d[1]);
    let raw_pitch = d[2].atan2(d[0].hypot(d[1]));
    let yaw_max = magnitude(limits.yaw_max);
    let pitch_up = magnitude(limits.pitch_up);
    let pitch_down = magnitude(limits.pitch_down);
    let yaw = clamp_range(raw_yaw, -yaw_max, yaw_max);
    let pitch = clamp_range(raw_pitch, -pitch_down, pitch_up);
    let clamped = yaw != raw_yaw || pitch != raw_pitch;
    let s = neck_share_of(neck_share);
    let neck_yaw = s * yaw;
    let neck_pitch = s * pitch;
    Some(Look {
        yaw,
        pitch,
        neck_yaw,
        neck_pitch,
        head_yaw: yaw - neck_yaw,
        head_pitch: pitch - neck_pitch,
        clamped,
    })
}

/// P1 and P2: the two-bone solve for one arm, bending toward the pole.
pub fn point_at(
    shoulder: Vec3,
    target: Vec3,
    upper: f64,
    lower: f64,
    pole: Vec3,
) -> Result<Arm, IkError> {
    if !upper.is_finite() || upper <= 0.0 || !lower.is_finite() || lower <= 0.0 {
        return Err(IkError::BadLength);
    }
    if !finite3(shoulder) || !finite3(target) {
        return Err(IkError::NoDirection);
    }
    let d = [
        target[0] - shoulder[0],
        target[1] - shoulder[1],
        target[2] - shoulder[2],
    ];
    let dist = length3(d);
    if dist < 1e-9 {
        return Err(IkError::NoDirection);
    }
    let u = [d[0] / dist, d[1] / dist, d[2] / dist];
    if !finite3(pole) {
        return Err(IkError::PoleAlongArm);
    }
    let along = pole[0] * u[0] + pole[1] * u[1] + pole[2] * u[2];
    let p = [
        pole[0] - along * u[0],
        pole[1] - along * u[1],
        pole[2] - along * u[2],
    ];
    let p_len = length3(p);
    if p_len < 1e-9 {
        return Err(IkError::PoleAlongArm);
    }
    let n = [p[0] / p_len, p[1] / p_len, p[2] / p_len];
    let a = upper;
    let b = lower;
    let lo = (a - b).abs();
    let hi = a + b;
    let dc = clamp_range(dist, lo, hi);
    let reached = dist >= lo && dist <= hi;
    let x = (a * a - b * b + dc * dc) / (2.0 * dc);
    let h = (a * a - x * x).max(0.0).sqrt();
    let elbow = [
        shoulder[0] + x * u[0] + h * n[0],
        shoulder[1] + x * u[1] + h * n[1],
        shoulder[2] + x * u[2] + h * n[2],
    ];
    let hand = [
        shoulder[0] + dc * u[0],
        shoulder[1] + dc * u[1],
        shoulder[2] + dc * u[2],
    ];
    let cos_angle = (a * a + b * b - dc * dc) / (2.0 * a * b);
    let elbow_angle = clamp_range(cos_angle, -1.0, 1.0).acos();
    Ok(Arm {
        elbow,
        hand,
        elbow_angle,
        reached,
    })
}

/// C1: the crossfade length actually used, in seconds.
pub fn crossfade_duration(requested_s: f64) -> f64 {
    if !requested_s.is_finite() {
        return CROSSFADE_S;
    }
    clamp_range(requested_s, CROSSFADE_MIN_S, CROSSFADE_MAX_S)
}

/// C2: the incoming clip's weight, a smoothstep of the elapsed fraction.
pub fn fade_weight(elapsed_s: f64, duration_s: f64) -> f64 {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return 1.0;
    }
    let t = if elapsed_s.is_nan() {
        0.0
    } else {
        clamp_range(elapsed_s / duration_s, 0.0, 1.0)
    };
    t * t * (3.0 - 2.0 * t)
}

/// E1: the clip Sancho plays for an event.
pub fn clip_for(event: Event) -> Clip {
    match event {
        Event::Idle => Clip::Idle,
        Event::Greet => Clip::Wave,
        Event::FindingRaised => Clip::Point,
        Event::UnsafeRaised => Clip::Worried,
        Event::Milestone => Clip::Celebrate,
        Event::Asked => Clip::Think,
        Event::WalkingToTarget => Clip::WalkTo,
    }
}

/// T1: what a hardware tier runs (doc 07's table).
pub fn caps(tier: Tier) -> Caps {
    match tier {
        Tier::Capable => Caps {
            skinning: true,
            clips: true,
            crossfade: true,
            look_at: true,
            point_at: true,
            baked: false,
            frame_rate: None,
        },
        Tier::Modest => Caps {
            skinning: true,
            clips: true,
            crossfade: true,
            look_at: false,
            point_at: false,
            baked: false,
            frame_rate: None,
        },
        Tier::Potato => Caps {
            skinning: false,
            clips: true,
            crossfade: false,
            look_at: false,
            point_at: false,
            baked: true,
            frame_rate: Some(12.0),
        },
        Tier::Muted => Caps {
            skinning: false,
            clips: false,
            crossfade: false,
            look_at: false,
            point_at: false,
            baked: false,
            frame_rate: None,
        },
    }
}

/// W1: bring an angle into (-pi, pi], so pi stays pi and -pi becomes pi.
pub fn wrap_angle(angle: f64) -> f64 {
    if !angle.is_finite() {
        return 0.0;
    }
    let r = angle.rem_euclid(TAU);
    if r > PI {
        r - TAU
    } else {
        r
    }
}

/// W2: turn in place to face the target, then walk straight at it.
pub fn walk_to(from: [f64; 2], facing: f64, to: [f64; 2], stop_short: f64) -> Walk {
    if !from[0].is_finite()
        || !from[1].is_finite()
        || !facing.is_finite()
        || !to[0].is_finite()
        || !to[1].is_finite()
    {
        return Walk {
            turn: 0.0,
            distance: 0.0,
            heading: 0.0,
        };
    }
    let stop = if stop_short.is_finite() && stop_short > 0.0 {
        stop_short
    } else {
        0.0
    };
    let vx = to[0] - from[0];
    let vy = to[1] - from[1];
    let len = vx.hypot(vy);
    if len < 1e-9 {
        return Walk {
            turn: 0.0,
            distance: 0.0,
            heading: wrap_angle(facing),
        };
    }
    let heading = (-vx).atan2(vy);
    let turn = wrap_angle(heading - facing);
    let distance = (len - stop).max(0.0);
    Walk {
        turn,
        distance,
        heading,
    }
}

/// H1: the uniform scale that makes a rest-height figure stand `height_m` tall.
pub fn height_scale(height_m: f64, rest_height_m: f64) -> Option<f64> {
    if !height_m.is_finite() || height_m < MIN_HEIGHT_M || height_m > MAX_HEIGHT_M {
        return None;
    }
    if !rest_height_m.is_finite() || rest_height_m <= 0.0 {
        return None;
    }
    Some(height_m / rest_height_m)
}
