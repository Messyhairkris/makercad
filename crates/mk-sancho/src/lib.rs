//! Sancho v1: MakerCAD's helper as a jointed wooden mannequin with fingers.
//!
//! Generated from code, so there is no model file with licence questions and the proportions
//! follow the height the maker gives (Sancho stands at the maker's height in their room). The
//! rig is a skeleton of 51 joints: hips, spine, chest, neck, head; per arm a shoulder, upper
//! arm, forearm, hand and three bones in each of five fingers; per leg a thigh, shin, foot and
//! toe. Every body part is a rigid wooden segment skinned to one joint, which is exactly how a
//! real artist's mannequin moves. Ball joints sit at the shoulders, elbows, wrists, hips, knees
//! and ankles.
//!
//! The output is a binary glTF 2.0 file (`.glb`) with two animations, `Wave` and `Grip`, that
//! show the rig working. glTF conventions: metres, +Y up, the figure faces +Z, so its left side
//! is +X. Rest pose: a T-pose, palms down.
//!
//! Sancho is a scale figure first (decision D27). [`figure`] reads any skinned humanoid glTF,
//! such as the placeholder [`PLACEHOLDER_GLB`] (BodyKun, D26), and poses it on the CPU: still,
//! posable, standing beside the design.
//!
//! [`rig`] is the light rig's math for when he moves (doc 07 section 7.3.1; the Crew's campaign
//! mk-core-14): look-at with clamped yaw and pitch split between neck and head, point-at as a
//! two-bone IK, clip crossfades, the event-to-clip table, what each hardware tier runs, walking
//! and height scaling. f64 throughout; no glTF or GPU here.
//!
//! [`persona`] is the persona file of "Bring your own Sancho" (doc 07 section 7.3.1; campaign
//! mk-core-15): the TOML subset it uses read and written back, errors with line numbers, Mixamo
//! names and a built-in Rigify-to-Mixamo table, and an imported model checked against facts the
//! app measured (bones, influences, clips, rest height, triangles).

//!
//! [`clips`] is Sancho's animation clips as data and the math that plays them (doc 07 section
//! 7.3.1; campaign mk-core-22): one rotation track per bone as unit quaternions, sampling with
//! normalized lerp the shorter way round, looping or holding the last key, blending two poses by
//! a crossfade weight (from `rig::fade_weight`), validation with exact errors, and baking to
//! whole frames for the potato tier.

pub mod clips;
pub mod figure;
mod glb;
pub mod persona;
pub mod rig;
mod shapes;

pub use glb::write_glb;

/// The placeholder figure (decision D26): BodyKun by vinchau (CC0), rebuilt by
/// `tools/blender/export_bodykun.py`. 71 deform bones, one mesh, 1.68 m, Y up, facing +Z.
pub const PLACEHOLDER_GLB: &[u8] = include_bytes!("../assets/sancho-bodykun.glb");

/// One joint of the rig.
#[derive(Clone, Debug, PartialEq)]
pub struct Joint {
    /// A stable name (`upper_arm_l`, `index_l_2`...). Never change one: animations use them.
    pub name: String,
    /// The parent joint's index; `None` for the root (`hips`).
    pub parent: Option<usize>,
    /// Position in the rest pose, in metres, in the figure's space.
    pub rest: [f32; 3],
}

/// The fingers, thumb first, with the lengths of their three bones as a fraction of the
/// figure's height and the offset of the finger's base across the palm (along Z).
const FINGERS: [(&str, [f32; 3], f32); 5] = [
    ("thumb", [0.021, 0.018, 0.015], 0.0),
    ("index", [0.024, 0.015, 0.012], 0.018),
    ("middle", [0.027, 0.018, 0.012], 0.006),
    ("ring", [0.024, 0.017, 0.012], -0.006),
    ("pinky", [0.019, 0.012, 0.011], -0.018),
];

/// Sancho's default height: 168 cm, as on the opening screen.
pub const DEFAULT_HEIGHT_M: f32 = 1.68;

/// The rig for a figure `height_m` tall, root first; every parent comes before its children.
pub fn skeleton(height_m: f32) -> Vec<Joint> {
    let h = if height_m.is_finite() && height_m > 0.1 {
        height_m
    } else {
        DEFAULT_HEIGHT_M
    };
    // Proportions as fractions of the height (an adult mannequin, about 7.5 heads tall).
    let at = |x: f32, y: f32, z: f32| [x * h, y * h, z * h];
    let mut joints: Vec<Joint> = Vec::new();
    let add = |joints: &mut Vec<Joint>, name: String, parent: Option<usize>, rest: [f32; 3]| {
        joints.push(Joint { name, parent, rest });
        joints.len() - 1
    };
    let hips = add(&mut joints, "hips".into(), None, at(0.0, 0.545, 0.0));
    let spine = add(&mut joints, "spine".into(), Some(hips), at(0.0, 0.625, 0.0));
    let chest = add(
        &mut joints,
        "chest".into(),
        Some(spine),
        at(0.0, 0.745, 0.0),
    );
    let neck = add(&mut joints, "neck".into(), Some(chest), at(0.0, 0.855, 0.0));
    add(&mut joints, "head".into(), Some(neck), at(0.0, 0.895, 0.0));
    for (side, sx) in [("l", 1.0_f32), ("r", -1.0_f32)] {
        let shoulder = add(
            &mut joints,
            format!("shoulder_{side}"),
            Some(chest),
            at(sx * 0.05, 0.825, 0.0),
        );
        let upper = add(
            &mut joints,
            format!("upper_arm_{side}"),
            Some(shoulder),
            at(sx * 0.115, 0.825, 0.0),
        );
        let fore = add(
            &mut joints,
            format!("forearm_{side}"),
            Some(upper),
            at(sx * 0.28, 0.825, 0.0),
        );
        let hand = add(
            &mut joints,
            format!("hand_{side}"),
            Some(fore),
            at(sx * 0.43, 0.825, 0.0),
        );
        for (finger, lengths, across) in FINGERS {
            let (mut pos, dir) = if finger == "thumb" {
                (
                    at(sx * 0.452, 0.82, 0.024),
                    normalize([sx * 0.6, -0.15, 0.8]),
                )
            } else {
                (at(sx * 0.488, 0.825, across), [sx, 0.0, 0.0])
            };
            let mut parent = hand;
            for (bone, len) in lengths.iter().enumerate() {
                parent = add(
                    &mut joints,
                    format!("{finger}_{side}_{}", bone + 1),
                    Some(parent),
                    pos,
                );
                pos = add3(pos, scale(dir, len * h));
            }
        }
    }
    for (side, sx) in [("l", 1.0_f32), ("r", -1.0_f32)] {
        let thigh = add(
            &mut joints,
            format!("thigh_{side}"),
            Some(hips),
            at(sx * 0.055, 0.525, 0.0),
        );
        let shin = add(
            &mut joints,
            format!("shin_{side}"),
            Some(thigh),
            at(sx * 0.055, 0.29, 0.0),
        );
        let foot = add(
            &mut joints,
            format!("foot_{side}"),
            Some(shin),
            at(sx * 0.055, 0.045, 0.0),
        );
        add(
            &mut joints,
            format!("toe_{side}"),
            Some(foot),
            at(sx * 0.058, 0.016, 0.075),
        );
    }
    joints
}

/// The index of the joint called `name`.
pub fn joint_index(joints: &[Joint], name: &str) -> Option<usize> {
    joints.iter().position(|j| j.name == name)
}

/// The tip of a finger's last bone, which is not a joint: the last joint plus the finger's
/// direction times its last bone's length.
fn finger_tip(joints: &[Joint], h: f32, finger: &str, side: &str) -> Option<[f32; 3]> {
    let (_, lengths, _) = FINGERS.iter().find(|(f, _, _)| *f == finger)?;
    let last = joints.get(joint_index(joints, &format!("{finger}_{side}_3"))?)?;
    let prev = joints.get(joint_index(joints, &format!("{finger}_{side}_2"))?)?;
    let dir = normalize(sub(last.rest, prev.rest));
    Some(add3(last.rest, scale(dir, lengths[2] * h)))
}

/// A triangle mesh skinned to one joint per vertex.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Part {
    /// Vertex positions, metres.
    pub positions: Vec<[f32; 3]>,
    /// Unit normals.
    pub normals: Vec<[f32; 3]>,
    /// The joint each vertex follows (weight 1).
    pub joints: Vec<u8>,
    /// Triangles, three vertex indices each, counter-clockwise from outside.
    pub indices: Vec<u32>,
}

/// The figure as two parts: the wooden segments and the darker ball joints.
pub fn body(height_m: f32) -> (Part, Part) {
    let joints = skeleton(height_m);
    let h = if height_m.is_finite() && height_m > 0.1 {
        height_m
    } else {
        DEFAULT_HEIGHT_M
    };
    let j = |name: &str| joint_index(&joints, name).unwrap_or(0);
    let p = |name: &str| joints.get(j(name)).map(|x| x.rest).unwrap_or([0.0; 3]);
    let ji = |name: &str| u8::try_from(j(name)).unwrap_or(0);
    let mut wood = Part::default();
    let mut balls = Part::default();
    let r = |f: f32| f * h;

    // Head, neck and torso.
    shapes::ellipsoid(
        &mut wood,
        add3(p("head"), [0.0, r(0.045), r(0.004)]),
        [r(0.053), r(0.066), r(0.058)],
        ji("head"),
    );
    shapes::capsule(&mut wood, p("neck"), p("head"), r(0.022), ji("neck"));
    shapes::ellipsoid(
        &mut wood,
        [0.0, r(0.77), 0.0],
        [r(0.105), r(0.085), r(0.062)],
        ji("chest"),
    );
    shapes::ellipsoid(
        &mut wood,
        [0.0, r(0.655), 0.0],
        [r(0.068), r(0.055), r(0.05)],
        ji("spine"),
    );
    shapes::ellipsoid(
        &mut wood,
        [0.0, r(0.55), 0.0],
        [r(0.092), r(0.06), r(0.062)],
        ji("hips"),
    );
    shapes::sphere(&mut balls, p("neck"), r(0.027), ji("neck"));
    shapes::sphere(&mut balls, [0.0, r(0.705), 0.0], r(0.045), ji("chest"));

    for side in ["l", "r"] {
        let s = |part: &str| format!("{part}_{side}");
        // Arm.
        shapes::capsule(
            &mut wood,
            p(&s("shoulder")),
            p(&s("upper_arm")),
            r(0.03),
            ji(&s("shoulder")),
        );
        shapes::sphere(
            &mut balls,
            p(&s("upper_arm")),
            r(0.032),
            ji(&s("upper_arm")),
        );
        shapes::capsule(
            &mut wood,
            p(&s("upper_arm")),
            p(&s("forearm")),
            r(0.025),
            ji(&s("upper_arm")),
        );
        shapes::sphere(&mut balls, p(&s("forearm")), r(0.024), ji(&s("forearm")));
        shapes::capsule(
            &mut wood,
            p(&s("forearm")),
            p(&s("hand")),
            r(0.02),
            ji(&s("forearm")),
        );
        shapes::sphere(&mut balls, p(&s("hand")), r(0.017), ji(&s("hand")));
        // Palm: a flat oval reaching to the finger bases.
        let sx = if side == "l" { 1.0 } else { -1.0 };
        shapes::ellipsoid(
            &mut wood,
            add3(p(&s("hand")), [sx * r(0.03), 0.0, 0.0]),
            [r(0.032), r(0.011), r(0.028)],
            ji(&s("hand")),
        );
        // Fingers: one capsule per bone, each following its own joint.
        for (finger, _, _) in FINGERS {
            for bone in 1..=3 {
                let name = format!("{finger}_{side}_{bone}");
                let start = p(&name);
                let end = if bone < 3 {
                    p(&format!("{finger}_{side}_{}", bone + 1))
                } else {
                    finger_tip(&joints, h, finger, side).unwrap_or(start)
                };
                let radius = if finger == "thumb" {
                    r(0.0062)
                } else {
                    r(0.0052)
                };
                shapes::capsule(&mut wood, start, end, radius, ji(&name));
            }
        }
        // Leg.
        shapes::sphere(&mut balls, p(&s("thigh")), r(0.042), ji(&s("thigh")));
        shapes::capsule(
            &mut wood,
            p(&s("thigh")),
            p(&s("shin")),
            r(0.038),
            ji(&s("thigh")),
        );
        shapes::sphere(&mut balls, p(&s("shin")), r(0.032), ji(&s("shin")));
        shapes::capsule(
            &mut wood,
            p(&s("shin")),
            p(&s("foot")),
            r(0.029),
            ji(&s("shin")),
        );
        shapes::sphere(&mut balls, p(&s("foot")), r(0.022), ji(&s("foot")));
        shapes::capsule(
            &mut wood,
            p(&s("foot")),
            p(&s("toe")),
            r(0.02),
            ji(&s("foot")),
        );
        let toe_end = add3(p(&s("toe")), [0.0, 0.0, r(0.03)]);
        shapes::capsule(&mut wood, p(&s("toe")), toe_end, r(0.017), ji(&s("toe")));
    }
    (wood, balls)
}

/// A keyframed rotation of one joint.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// The joint that turns.
    pub joint: usize,
    /// Key times, seconds.
    pub times: Vec<f32>,
    /// Rotations at those times, as unit quaternions `[x, y, z, w]`.
    pub rotations: Vec<[f32; 4]>,
}

/// A named animation.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    /// `Wave` or `Grip`.
    pub name: String,
    /// One track per moving joint.
    pub tracks: Vec<Track>,
}

/// The rotation by `angle` radians about the unit `axis`.
pub fn quat(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let a = normalize(axis);
    let (s, c) = (angle * 0.5).sin_cos();
    [a[0] * s, a[1] * s, a[2] * s, c]
}

/// The two animations: `Wave` (right arm up, forearm waving, 2 s loop) and `Grip` (every
/// finger of both hands curls and opens again, 2 s loop).
pub fn animations(joints: &[Joint]) -> Vec<Animation> {
    let z = [0.0, 0.0, 1.0];
    let mut out = Vec::new();
    let mut wave = Vec::new();
    if let (Some(upper), Some(fore)) = (
        joint_index(joints, "upper_arm_r"),
        joint_index(joints, "forearm_r"),
    ) {
        let times = vec![0.0, 0.5, 1.0, 1.5, 2.0];
        wave.push(Track {
            joint: upper,
            times: times.clone(),
            rotations: vec![quat(z, -1.15); 5],
        });
        let swing = [-0.95, -0.25, -0.95, -0.25, -0.95];
        wave.push(Track {
            joint: fore,
            times,
            rotations: swing.iter().map(|a| quat(z, *a)).collect(),
        });
    }
    out.push(Animation {
        name: "Wave".into(),
        tracks: wave,
    });

    let mut grip = Vec::new();
    for (side, sign) in [("l", -1.0_f32), ("r", 1.0_f32)] {
        for (finger, _, _) in FINGERS {
            for (bone, curl) in [(1, 1.0_f32), (2, 1.25), (3, 0.9)] {
                let Some(joint) = joint_index(joints, &format!("{finger}_{side}_{bone}")) else {
                    continue;
                };
                let closed = if finger == "thumb" {
                    quat([0.0, 1.0, 0.0], -sign * 0.45 * curl)
                } else {
                    quat(z, sign * curl)
                };
                let open = [0.0, 0.0, 0.0, 1.0];
                grip.push(Track {
                    joint,
                    times: vec![0.0, 1.0, 2.0],
                    rotations: vec![open, closed, open],
                });
            }
        }
    }
    out.push(Animation {
        name: "Grip".into(),
        tracks: grip,
    });
    out
}

pub(crate) fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(crate) fn normalize(a: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    if len > 1e-12 {
        scale(a, 1.0 / len)
    } else {
        [0.0, 1.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rig_has_51_joints_and_15_finger_bones_per_hand() {
        let s = skeleton(DEFAULT_HEIGHT_M);
        assert_eq!(s.len(), 51);
        for side in ["l", "r"] {
            let fingers = s.iter().filter(|j| {
                FINGERS
                    .iter()
                    .any(|(f, _, _)| j.name.starts_with(&format!("{f}_{side}_")))
            });
            assert_eq!(fingers.count(), 15, "{side}");
        }
    }

    #[test]
    fn parents_come_first_and_names_are_unique() {
        let s = skeleton(DEFAULT_HEIGHT_M);
        assert_eq!(s.first().map(|j| j.name.as_str()), Some("hips"));
        for (i, j) in s.iter().enumerate() {
            if let Some(p) = j.parent {
                assert!(p < i, "{} comes before its parent", j.name);
            } else {
                assert_eq!(i, 0);
            }
            assert_eq!(
                s.iter().filter(|o| o.name == j.name).count(),
                1,
                "{}",
                j.name
            );
        }
    }

    #[test]
    fn left_mirrors_right() {
        let s = skeleton(DEFAULT_HEIGHT_M);
        for j in s.iter().filter(|j| j.name.contains("_l")) {
            let twin = j.name.replace("_l", "_r");
            let other = s.iter().find(|o| o.name == twin).expect("twin");
            assert!((j.rest[0] + other.rest[0]).abs() < 1e-6, "{}", j.name);
            assert!((j.rest[1] - other.rest[1]).abs() < 1e-6, "{}", j.name);
            assert!((j.rest[2] - other.rest[2]).abs() < 1e-6, "{}", j.name);
        }
    }

    #[test]
    fn the_figure_is_as_tall_as_asked_and_stands_on_the_floor() {
        for h in [1.2, 1.68, 1.9] {
            let (wood, balls) = body(h);
            let ys = wood.positions.iter().chain(&balls.positions).map(|p| p[1]);
            let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(lo, hi), y| (lo.min(y), hi.max(y)));
            assert!(lo > -0.01 * h && lo < 0.02 * h, "feet at {lo} for {h}");
            assert!((hi - h).abs() < 0.03 * h, "top at {hi} for {h}");
        }
    }

    #[test]
    fn every_vertex_follows_a_real_joint_and_every_triangle_is_in_range() {
        let s = skeleton(DEFAULT_HEIGHT_M);
        let (wood, balls) = body(DEFAULT_HEIGHT_M);
        for part in [&wood, &balls] {
            assert_eq!(part.positions.len(), part.normals.len());
            assert_eq!(part.positions.len(), part.joints.len());
            assert!(part.joints.iter().all(|j| usize::from(*j) < s.len()));
            assert_eq!(part.indices.len() % 3, 0);
            assert!(part
                .indices
                .iter()
                .all(|i| (*i as usize) < part.positions.len()));
            for n in &part.normals {
                assert!((dot(*n, *n) - 1.0).abs() < 1e-3, "{n:?}");
            }
        }
    }

    #[test]
    fn each_finger_bone_has_its_own_segment() {
        let s = skeleton(DEFAULT_HEIGHT_M);
        let (wood, _) = body(DEFAULT_HEIGHT_M);
        for name in ["index_l_3", "thumb_r_1", "pinky_r_2"] {
            let j = joint_index(&s, name).expect("joint") as u8;
            assert!(wood.joints.contains(&j), "{name} moves no wood");
        }
    }

    #[test]
    fn the_animations_turn_real_joints_with_unit_quaternions() {
        let s = skeleton(DEFAULT_HEIGHT_M);
        let anims = animations(&s);
        assert_eq!(
            anims.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            ["Wave", "Grip"]
        );
        let grip = &anims[1];
        assert_eq!(
            grip.tracks.len(),
            30,
            "every finger bone of both hands curls"
        );
        for a in &anims {
            for t in &a.tracks {
                assert!(t.joint < s.len());
                assert_eq!(t.times.len(), t.rotations.len());
                assert!(t.times.windows(2).all(|w| w[0] < w[1]));
                for q in &t.rotations {
                    let len = q.iter().map(|c| c * c).sum::<f32>().sqrt();
                    assert!((len - 1.0).abs() < 1e-5);
                }
                assert_eq!(
                    t.rotations.first(),
                    t.rotations.last(),
                    "a loop ends where it starts"
                );
            }
        }
    }

    #[test]
    fn a_silly_height_falls_back_to_the_default() {
        assert_eq!(skeleton(f32::NAN), skeleton(DEFAULT_HEIGHT_M));
        assert_eq!(skeleton(-3.0), skeleton(DEFAULT_HEIGHT_M));
    }
}
