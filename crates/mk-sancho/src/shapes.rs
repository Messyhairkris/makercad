//! Smooth primitives for the mannequin: capsules, spheres and ellipsoids, each added to a
//! [`Part`] and skinned rigidly to one joint.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::{add3, cross, dot, normalize, scale, sub, Part};

/// Points around each ring.
const AROUND: u32 = 14;
/// Rings in each half sphere.
const HALF: u32 = 5;

/// A capsule from `a` to `b` with radius `r`.
pub fn capsule(part: &mut Part, a: [f32; 3], b: [f32; 3], r: f32, joint: u8) {
    let axis = sub(b, a);
    let dir = if dot(axis, axis) > 1e-12 {
        normalize(axis)
    } else {
        [0.0, 1.0, 0.0]
    };
    let helper = if dir[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let u = normalize(cross(dir, helper));
    let v = cross(dir, u);
    let base = part.positions.len() as u32;
    let rings = 2 * (HALF + 1);
    for ring in 0..rings {
        let (center, phi) = if ring <= HALF {
            (a, -FRAC_PI_2 + FRAC_PI_2 * ring as f32 / HALF as f32)
        } else {
            (b, FRAC_PI_2 * (ring - HALF - 1) as f32 / HALF as f32)
        };
        let (sp, cp) = phi.sin_cos();
        for k in 0..AROUND {
            let (st, ct) = (TAU * k as f32 / AROUND as f32).sin_cos();
            let radial = add3(scale(u, ct), scale(v, st));
            let normal = normalize(add3(scale(dir, sp), scale(radial, cp)));
            part.positions.push(add3(center, scale(normal, r)));
            part.normals.push(normal);
            part.joints.push(joint);
        }
    }
    stitch(part, base, rings);
}

/// A sphere of radius `r` at `c`.
pub fn sphere(part: &mut Part, c: [f32; 3], r: f32, joint: u8) {
    ellipsoid(part, c, [r, r, r], joint);
}

/// An ellipsoid at `c` with radii `radii` along x, y and z.
pub fn ellipsoid(part: &mut Part, c: [f32; 3], radii: [f32; 3], joint: u8) {
    let base = part.positions.len() as u32;
    let rings = 2 * HALF + 1;
    for ring in 0..rings {
        let phi = -FRAC_PI_2 + PI * ring as f32 / (rings - 1) as f32;
        let (sp, cp) = phi.sin_cos();
        for k in 0..AROUND {
            let (st, ct) = (TAU * k as f32 / AROUND as f32).sin_cos();
            // Going round clockwise seen from +Y, so (around, up) is right-handed about the
            // outward normal, the same as the capsule's rings.
            let unit = [cp * ct, sp, -cp * st];
            let pos = [
                c[0] + radii[0] * unit[0],
                c[1] + radii[1] * unit[1],
                c[2] + radii[2] * unit[2],
            ];
            let normal = normalize([
                unit[0] / radii[0].max(1e-6),
                unit[1] / radii[1].max(1e-6),
                unit[2] / radii[2].max(1e-6),
            ]);
            part.positions.push(pos);
            part.normals.push(normal);
            part.joints.push(joint);
        }
    }
    stitch(part, base, rings);
}

/// Triangles between consecutive rings of `AROUND` points, outward facing.
fn stitch(part: &mut Part, base: u32, rings: u32) {
    for ring in 0..rings - 1 {
        for k in 0..AROUND {
            let a = base + ring * AROUND + k;
            let b = base + ring * AROUND + (k + 1) % AROUND;
            let c = a + AROUND;
            let d = b + AROUND;
            // With (around, up) right-handed about the outward normal, this order is
            // counter-clockwise seen from outside.
            part.indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_normal(p: &Part, t: usize) -> ([f32; 3], [f32; 3]) {
        let i = |n: usize| p.indices[t * 3 + n] as usize;
        let (a, b, c) = (p.positions[i(0)], p.positions[i(1)], p.positions[i(2)]);
        let face = cross(sub(b, a), sub(c, a));
        let centre = scale(add3(add3(a, b), c), 1.0 / 3.0);
        (face, centre)
    }

    #[test]
    fn triangles_face_outward() {
        let mut p = Part::default();
        capsule(&mut p, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.2, 0);
        sphere(&mut p, [3.0, 0.0, 0.0], 0.5, 0);
        let mut outward = 0;
        let mut checked = 0;
        for t in 0..p.indices.len() / 3 {
            let (face, centre) = triangle_normal(&p, t);
            if dot(face, face) < 1e-12 {
                continue; // a degenerate triangle at a pole
            }
            let inside = if centre[0] > 1.5 {
                [3.0, 0.0, 0.0]
            } else {
                [0.0, centre[1].clamp(0.0, 1.0), 0.0]
            };
            checked += 1;
            if dot(face, sub(centre, inside)) > 0.0 {
                outward += 1;
            }
        }
        assert_eq!(outward, checked);
    }

    #[test]
    fn a_capsule_reaches_its_ends_plus_its_radius() {
        let mut p = Part::default();
        capsule(&mut p, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.25, 3);
        let ys: Vec<f32> = p.positions.iter().map(|v| v[1]).collect();
        let lo = ys.iter().cloned().fold(f32::MAX, f32::min);
        let hi = ys.iter().cloned().fold(f32::MIN, f32::max);
        assert!(
            (lo + 0.25).abs() < 1e-5 && (hi - 1.25).abs() < 1e-5,
            "{lo} {hi}"
        );
        assert!(p.joints.iter().all(|j| *j == 3));
    }
}
