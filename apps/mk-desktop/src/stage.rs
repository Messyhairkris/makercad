//! The stage a design is shown on, with Sancho standing on it (decision D27; doc 05 §5.5 as Kris
//! amended it on 2026-11-21).
//!
//! A small part (its largest side up to [`BENCH_LIMIT_MM`]) sits on a **workbench** at standing
//! height: the design's floor plane becomes the bench top, a cutting mat with a 10 mm grid lies
//! under the part, and Sancho stands at true size on the room floor beside the bench, his feet a
//! bench height below the part. Anything bigger stands on the floor, and so does Sancho, beside
//! it. The stage is scenery: added to the picture only, never to the design.

use std::sync::Arc;

use solidcore_record::Vec3;
use solidcore_tess::Mesh;
use solidcraft_viewport::vec::Bounds;
use solidcraft_viewport::{Rgb, SceneLine, SceneMesh};

/// A part whose largest side is at most this (mm) is shown on the workbench (doc 05's `bench`
/// class, 10–500 mm, and smaller).
pub const BENCH_LIMIT_MM: f64 = 500.0;
/// Height of the bench top above the floor (mm): a standing workbench.
pub const BENCH_HEIGHT_MM: f64 = 900.0;
/// Thickness of the bench top (mm).
const TOP_MM: f64 = 40.0;
/// Thickness of the cutting mat (mm).
const MAT_MM: f64 = 2.0;
/// Side of a square leg (mm).
const LEG_MM: f64 = 60.0;
/// The gap between the bench (or the design) and Sancho (mm).
const GAP_MM: f64 = 250.0;

const BENCH_WOOD: Rgb = Rgb(214, 178, 132);
const LEG_WOOD: Rgb = Rgb(176, 136, 96);
const MAT_GREEN: Rgb = Rgb(118, 164, 136);
const MAT_LINE: Rgb = Rgb(236, 244, 238);
const FLOOR: Rgb = Rgb(228, 222, 210);

/// Where everything stands for one design.
#[derive(Clone, Debug, PartialEq)]
pub struct Stage {
    /// The bench top's box, when the design is small enough for the bench.
    pub bench: Option<(Vec3, Vec3)>,
    /// The height of the floor Sancho stands on (mm).
    pub floor_z: f64,
    /// Where Sancho's feet are (the middle of them).
    pub sancho: Vec3,
    /// Whether SolidCraft's ground grid is shown (on the floor stage; the bench has its mat).
    pub grid: bool,
}

/// The stage for a design whose box is `design` (min, max), or for an empty design, with Sancho
/// `sancho_half_width` (mm) wide on each side of his middle.
pub fn plan(design: Option<(Vec3, Vec3)>, sancho_half_width: f64) -> Stage {
    let (min, max) = design.unwrap_or((Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)));
    let largest = (max.x - min.x)
        .max(max.y - min.y)
        .max(max.z - min.z)
        .max(0.0);
    let ground = min.z.min(0.0);
    let middle_y = (min.y + max.y) * 0.5;
    if design.is_some() && largest <= BENCH_LIMIT_MM {
        // The top: the part's footprint with room around it, at least the doc's 600 × 450 mm.
        let w = (max.x - min.x + 300.0).max(600.0);
        let d = (max.y - min.y + 250.0).max(450.0);
        let cx = (min.x + max.x) * 0.5;
        let top = (
            Vec3::new(cx - w * 0.5, middle_y - d * 0.5, ground - MAT_MM - TOP_MM),
            Vec3::new(cx + w * 0.5, middle_y + d * 0.5, ground),
        );
        let floor_z = ground - BENCH_HEIGHT_MM;
        let sancho = Vec3::new(top.1.x + GAP_MM + sancho_half_width, middle_y, floor_z);
        Stage {
            bench: Some(top),
            floor_z,
            sancho,
            grid: false,
        }
    } else {
        let gap = 300.0 + 0.1 * largest;
        let sancho = match design {
            Some(_) => Vec3::new(max.x + gap + sancho_half_width, middle_y, ground),
            None => Vec3::new(300.0 + sancho_half_width, 0.0, 0.0),
        };
        Stage {
            bench: None,
            floor_z: ground,
            sancho,
            grid: true,
        }
    }
}

impl Stage {
    /// The bench, its mat and the floor under the bench and Sancho, for the scene (nothing on
    /// the floor stage: SolidCraft's grid is the floor there).
    pub fn meshes(&self, sancho_half_width: f64) -> Vec<SceneMesh> {
        let Some((top_min, top_max)) = self.bench else {
            return Vec::new();
        };
        let mat_bottom = top_max.z - MAT_MM;
        let mut wood = Mesh::default();
        add_block(
            &mut wood,
            Vec3::new(top_min.x, top_min.y, top_min.z),
            Vec3::new(top_max.x, top_max.y, mat_bottom),
        );
        let mut legs = Mesh::default();
        let inset = 40.0;
        for (x, y) in [
            (top_min.x + inset, top_min.y + inset),
            (top_max.x - inset - LEG_MM, top_min.y + inset),
            (top_min.x + inset, top_max.y - inset - LEG_MM),
            (top_max.x - inset - LEG_MM, top_max.y - inset - LEG_MM),
        ] {
            add_block(
                &mut legs,
                Vec3::new(x, y, self.floor_z),
                Vec3::new(x + LEG_MM, y + LEG_MM, top_min.z),
            );
        }
        let mut mat = Mesh::default();
        add_block(
            &mut mat,
            Vec3::new(top_min.x + 30.0, top_min.y + 30.0, mat_bottom),
            Vec3::new(top_max.x - 30.0, top_max.y - 30.0, top_max.z),
        );
        let mut floor = Mesh::default();
        let reach = sancho_half_width + 300.0;
        add_block(
            &mut floor,
            Vec3::new(
                top_min.x - 300.0,
                top_min.y.min(self.sancho.y - reach) - 300.0,
                self.floor_z - 20.0,
            ),
            Vec3::new(
                self.sancho.x + reach,
                top_max.y.max(self.sancho.y + reach) + 300.0,
                self.floor_z,
            ),
        );
        vec![
            solid(floor, FLOOR),
            solid(legs, LEG_WOOD),
            solid(wood, BENCH_WOOD),
            solid(mat, MAT_GREEN),
        ]
    }

    /// The cutting mat's grid: a line every 10 mm, a bolder one every 50 mm, just above the mat.
    pub fn lines(&self) -> Vec<SceneLine> {
        let Some((top_min, top_max)) = self.bench else {
            return Vec::new();
        };
        let (x0, x1, y0, y1) = (
            top_min.x + 40.0,
            top_max.x - 40.0,
            top_min.y + 40.0,
            top_max.y - 40.0,
        );
        let z = top_max.z + 0.2;
        let mut lines = Vec::new();
        let line = |a: Vec3, b: Vec3, major: bool| SceneLine {
            points: vec![a, b],
            color: MAT_LINE,
            alpha: if major { 0.85 } else { 0.4 },
            width: if major { 1.5 } else { 1.0 },
            on_top: false,
        };
        // From the mat's middle outward, so the 50 mm lines fall on round positions from there.
        let (cx, cy) = (((x0 + x1) * 0.5).round(), ((y0 + y1) * 0.5).round());
        let steps = |lo: f64, hi: f64, c: f64| {
            let below = ((c - lo) / 10.0).floor() as i64;
            let above = ((hi - c) / 10.0).floor() as i64;
            (-below..=above).map(move |k| (c + 10.0 * k as f64, k % 5 == 0))
        };
        for (x, major) in steps(x0, x1, cx) {
            lines.push(line(Vec3::new(x, y0, z), Vec3::new(x, y1, z), major));
        }
        for (y, major) in steps(y0, y1, cy) {
            lines.push(line(Vec3::new(x0, y, z), Vec3::new(x1, y, z), major));
        }
        lines
    }

    /// The box the stage's scenery fills (the bench, its legs and the floor patch).
    pub fn bounds(&self, sancho_half_width: f64) -> Bounds {
        let mut b = Bounds::EMPTY;
        for m in self.meshes(sancho_half_width) {
            for p in &m.mesh.positions {
                b.add(*p);
            }
        }
        b
    }
}

fn solid(mesh: Mesh, color: Rgb) -> SceneMesh {
    let n = mesh.triangles.len();
    SceneMesh::solid(
        Arc::new(Mesh {
            tri_face: vec![0; n],
            display_only_edges: true,
            ..mesh
        }),
        color,
    )
}

/// Adds a box from `a` to `b` (any two opposite corners) to `mesh`: four corners and a flat
/// normal per face, triangles counter-clockwise seen from outside.
pub fn add_block(mesh: &mut Mesh, a: Vec3, b: Vec3) {
    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
    let (z0, z1) = (a.z.min(b.z), a.z.max(b.z));
    let p = Vec3::new;
    // Each face: its normal and its corners in order, counter-clockwise seen along -normal.
    let faces: [(Vec3, [Vec3; 4]); 6] = [
        (
            p(1.0, 0.0, 0.0),
            [p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), p(x1, y0, z1)],
        ),
        (
            p(-1.0, 0.0, 0.0),
            [p(x0, y0, z0), p(x0, y0, z1), p(x0, y1, z1), p(x0, y1, z0)],
        ),
        (
            p(0.0, 1.0, 0.0),
            [p(x0, y1, z0), p(x0, y1, z1), p(x1, y1, z1), p(x1, y1, z0)],
        ),
        (
            p(0.0, -1.0, 0.0),
            [p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1)],
        ),
        (
            p(0.0, 0.0, 1.0),
            [p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)],
        ),
        (
            p(0.0, 0.0, -1.0),
            [p(x0, y0, z0), p(x0, y1, z0), p(x1, y1, z0), p(x1, y0, z0)],
        ),
    ];
    for (normal, corners) in faces {
        let Ok(base) = u32::try_from(mesh.positions.len()) else {
            return;
        };
        mesh.positions.extend(corners);
        mesh.normals.extend([normal; 4]);
        mesh.triangles.push([base, base + 1, base + 2]);
        mesh.triangles.push([base, base + 2, base + 3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF: f64 = 260.0;

    fn plate() -> (Vec3, Vec3) {
        (Vec3::new(-60.0, -40.0, 0.0), Vec3::new(60.0, 40.0, 12.0))
    }

    #[test]
    fn a_small_part_sits_on_the_bench_and_sancho_stands_on_the_floor() {
        let s = plan(Some(plate()), HALF);
        let (top_min, top_max) = s.bench.expect("a bench for a 120 mm plate");
        assert_eq!(top_max.z, 0.0, "the bench top is the part's floor plane");
        assert!(
            top_max.x - top_min.x >= 600.0 && top_max.y - top_min.y >= 450.0,
            "at least 600 × 450"
        );
        assert_eq!(
            s.floor_z, -BENCH_HEIGHT_MM,
            "his feet a bench height below the part"
        );
        assert_eq!(s.sancho.z, s.floor_z);
        assert!(s.sancho.x - HALF > top_max.x, "beside the bench, not in it");
        assert!(!s.grid, "the mat replaces the ground grid");
    }

    #[test]
    fn a_big_part_stands_on_the_floor_with_sancho() {
        let dresser = (Vec3::new(0.0, 0.0, 0.0), Vec3::new(812.0, 450.0, 1200.0));
        let s = plan(Some(dresser), HALF);
        assert_eq!(s.bench, None);
        assert_eq!((s.floor_z, s.sancho.z), (0.0, 0.0));
        assert!(s.sancho.x - HALF > 812.0);
        assert!(s.grid);
        assert!(s.meshes(HALF).is_empty());
        // The limit: 500 mm is still bench work, 501 is not.
        let at = |side: f64| {
            plan(
                Some((Vec3::new(0.0, 0.0, 0.0), Vec3::new(side, 10.0, 10.0))),
                HALF,
            )
            .bench
            .is_some()
        };
        assert!(at(500.0) && !at(501.0));
    }

    #[test]
    fn the_bench_has_a_top_four_legs_a_mat_and_a_floor() {
        let s = plan(Some(plate()), HALF);
        let meshes = s.meshes(HALF);
        let boxes: Vec<usize> = meshes.iter().map(|m| m.mesh.triangles.len() / 12).collect();
        assert_eq!(boxes, vec![1, 4, 1, 1], "floor, legs, top, mat");
        // The legs reach the floor, and the floor covers Sancho's feet.
        let lowest = meshes[1]
            .mesh
            .positions
            .iter()
            .map(|p| p.z)
            .fold(f64::INFINITY, f64::min);
        assert_eq!(lowest, s.floor_z);
        let b = s.bounds(HALF);
        assert!(b.min.x < s.sancho.x - HALF && b.max.x > s.sancho.x + HALF);
        // The mat's grid: lines every 10 mm, inside the top.
        let lines = s.lines();
        assert!(lines.len() > 80, "{} lines", lines.len());
        let (top_min, top_max) = s.bench.expect("bench");
        assert!(lines
            .iter()
            .flat_map(|l| &l.points)
            .all(|p| p.x > top_min.x && p.x < top_max.x && p.z > top_max.z));
    }

    #[test]
    fn block_faces_point_outward() {
        let mut m = Mesh::default();
        add_block(
            &mut m,
            Vec3::new(10.0, 20.0, 30.0),
            Vec3::new(-5.0, 0.0, 0.0),
        );
        assert_eq!((m.positions.len(), m.triangles.len()), (24, 12));
        let centre = Vec3::new(2.5, 10.0, 15.0);
        for t in &m.triangles {
            let [a, b, c] = t.map(|i| m.positions[i as usize]);
            let (u, v) = (b - a, c - a);
            let n = Vec3::new(
                u.y * v.z - u.z * v.y,
                u.z * v.x - u.x * v.z,
                u.x * v.y - u.y * v.x,
            );
            let out = (a + b + c) * (1.0 / 3.0) - centre;
            assert!(
                n.x * out.x + n.y * out.y + n.z * out.z > 0.0,
                "triangle {t:?} faces inward"
            );
            let normal = m.normals[t[0] as usize];
            assert!(
                normal.x * n.x + normal.y * n.y + normal.z * n.z > 0.0,
                "its normal agrees"
            );
        }
    }

    #[test]
    fn an_empty_design_stands_sancho_beside_the_origin() {
        let s = plan(None, HALF);
        assert_eq!(s.bench, None);
        assert!(s.sancho.x > 300.0);
    }
}
