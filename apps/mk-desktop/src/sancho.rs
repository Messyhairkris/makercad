//! Sancho on the opening screen: a jointed wooden mannequin with fingers, waving.
//!
//! Drawn from code (no image file, no licence questions). Every part is a capsule between two
//! joints, so the same joint list can later drive the rigged 3D Sancho (doc 06): the 2D figure
//! is that skeleton seen from the front.

use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke, Vec2};

use crate::theme::{SANCHO_INK, WOOD};

/// The figure's design height in points; [`paint`] scales it to the rect.
const DESIGN_HEIGHT: f32 = 318.0;
/// Room above the head for the waving hand's fingers.
const TOP: f32 = 18.0;

/// One body part: a capsule from `a` to `b` with radius `r`, in design points.
#[derive(Clone, Copy, Debug)]
struct Part {
    a: Vec2,
    b: Vec2,
    r: f32,
}

const fn p(ax: f32, ay: f32, bx: f32, by: f32, r: f32) -> Part {
    Part {
        a: Vec2 { x: ax, y: ay },
        b: Vec2 { x: bx, y: by },
        r,
    }
}

/// The skeleton, x from the centre line, y down from the top of the head.
const BODY: &[Part] = &[
    // head and neck
    p(0.0, 26.0, 0.0, 34.0, 22.0),
    p(0.0, 50.0, 0.0, 62.0, 7.0),
    // chest, waist, hips
    p(-14.0, 86.0, 14.0, 86.0, 26.0),
    p(0.0, 112.0, 0.0, 126.0, 15.0),
    p(-10.0, 146.0, 10.0, 146.0, 22.0),
    // left arm, resting
    p(-38.0, 76.0, -50.0, 124.0, 9.0),
    p(-50.0, 124.0, -56.0, 168.0, 8.0),
    // right arm, waving
    p(38.0, 76.0, 70.0, 58.0, 9.0),
    p(70.0, 58.0, 80.0, 18.0, 8.0),
    // legs
    p(-16.0, 162.0, -18.0, 224.0, 12.0),
    p(-18.0, 224.0, -18.0, 282.0, 10.0),
    p(16.0, 162.0, 18.0, 224.0, 12.0),
    p(18.0, 224.0, 18.0, 282.0, 10.0),
    // feet
    p(-18.0, 288.0, -34.0, 290.0, 7.0),
    p(18.0, 288.0, 34.0, 290.0, 7.0),
];

/// The joints drawn as rings, like a wooden mannequin's.
const JOINTS: &[(f32, f32, f32)] = &[
    (-38.0, 76.0, 8.0),
    (38.0, 76.0, 8.0),
    (-50.0, 124.0, 6.5),
    (70.0, 58.0, 6.5),
    (-16.0, 162.0, 9.0),
    (16.0, 162.0, 9.0),
    (-18.0, 224.0, 7.5),
    (18.0, 224.0, 7.5),
];

/// The hands: wrist position and the direction the forearm points, in design points.
const HANDS: &[(Vec2, Vec2)] = &[
    (Vec2 { x: -56.0, y: 168.0 }, Vec2 { x: -6.0, y: 44.0 }),
    (Vec2 { x: 80.0, y: 18.0 }, Vec2 { x: 10.0, y: -40.0 }),
];

/// Paints Sancho to fill `rect` (keeping his proportions), outline first and wood on top, so
/// overlapping parts merge into one figure.
pub fn paint(painter: &Painter, rect: Rect) {
    let s = (rect.height() / DESIGN_HEIGHT).min(rect.width() / 200.0);
    let origin = pos2(rect.center().x, rect.center().y - DESIGN_HEIGHT * s / 2.0);
    let at = |v: Vec2| origin + (v + vec2(0.0, TOP)) * s;
    let outline = 2.0 * s.max(0.75);

    let mut parts: Vec<Part> = BODY.to_vec();
    for &(wrist, dir) in HANDS {
        parts.extend(hand(wrist, dir));
    }
    for part in &parts {
        capsule(
            painter,
            at(part.a),
            at(part.b),
            (part.r + outline / s) * s,
            SANCHO_INK,
        );
    }
    for part in &parts {
        capsule(painter, at(part.a), at(part.b), part.r * s, WOOD);
    }
    let ring = Stroke::new(outline * 0.75, SANCHO_INK.gamma_multiply(0.55));
    for &(x, y, r) in JOINTS {
        painter.circle_stroke(at(vec2(x, y)), r * s, ring);
    }
}

/// A palm and four fingers plus a thumb at `wrist`, pointing along `dir`.
fn hand(wrist: Vec2, dir: Vec2) -> Vec<Part> {
    let d = dir.normalized();
    let side = vec2(-d.y, d.x);
    let palm = wrist + d * 9.0;
    let mut parts = vec![Part {
        a: wrist + d * 3.0,
        b: palm,
        r: 7.5,
    }];
    for (i, spread) in [-0.42_f32, -0.14, 0.14, 0.42].iter().enumerate() {
        let f = rotate(d, *spread);
        let base = palm + side * ((i as f32 - 1.5) * 3.2) + d * 4.0;
        let len = if i == 0 || i == 3 { 10.0 } else { 12.5 };
        parts.push(Part {
            a: base,
            b: base + f * len,
            r: 2.3,
        });
    }
    let thumb = rotate(d, 1.25);
    parts.push(Part {
        a: palm - side * 4.0,
        b: palm - side * 4.0 + thumb * 9.0,
        r: 2.6,
    });
    parts
}

fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn capsule(painter: &Painter, a: Pos2, b: Pos2, r: f32, color: Color32) {
    painter.circle_filled(a, r, color);
    painter.circle_filled(b, r, color);
    if a.distance(b) > 0.01 {
        painter.line_segment([a, b], Stroke::new(r * 2.0, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_hand_has_a_palm_four_fingers_and_a_thumb() {
        for &(wrist, dir) in HANDS {
            assert_eq!(hand(wrist, dir).len(), 6);
        }
    }

    #[test]
    fn the_figure_and_its_fingers_stay_inside_the_design_box() {
        let mut parts: Vec<Part> = BODY.to_vec();
        for &(wrist, dir) in HANDS {
            parts.extend(hand(wrist, dir));
        }
        for part in parts {
            for v in [part.a, part.b] {
                assert!(v.x.abs() + part.r <= 100.0, "{part:?}");
                let y = v.y + TOP;
                assert!(y - part.r >= 0.0 && y + part.r <= DESIGN_HEIGHT, "{part:?}");
            }
        }
    }
}
