//! Sancho as the scale figure (decision D27): a still figure at true size standing beside the
//! design, as SketchUp puts a person in each new model, so the maker reads the design's size
//! against a body at a glance. He is not part of the design: never saved, never picked, not in
//! the parts list, and the top bar hides him.
//!
//! The figure is the placeholder (D26, BodyKun), posed once by `mk_sancho::figure::standing`
//! and turned into the viewport's own mesh type: millimetres, Z up, facing the front view.

use std::sync::Arc;

use mk_sancho::figure::{standing, Figure};
use solidcore_record::Vec3;
use solidcore_tess::Mesh;
use solidcraft_viewport::vec::Bounds;
use solidcraft_viewport::{Rgb, SceneMesh};

/// His colour: a light wood that reads as a figure, not as part of the design.
pub const WOOD: Rgb = Rgb(200, 160, 118);

/// How far he turns from facing the front toward the design (on his right), in degrees.
const TURN_DEG: f64 = -20.0;

/// The figure, posed, ready to stand anywhere.
pub struct ScaleFigure {
    /// Posed, in mm, Z up, feet on z = 0, centred on x = y = 0.
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
    /// His height in mm.
    pub height_mm: f64,
    /// Half the larger of his width and depth, in mm.
    half_width_mm: f64,
    /// The last placed mesh and where it stands, so an unchanged spot reuses it.
    placed: Option<(Vec3, Arc<Mesh>)>,
}

impl ScaleFigure {
    /// The placeholder figure standing at ease, `height_mm` tall.
    pub fn placeholder(height_mm: f64) -> Result<ScaleFigure, String> {
        let figure = Figure::from_glb(mk_sancho::PLACEHOLDER_GLB).map_err(|e| e.to_string())?;
        Self::from_figure(&figure, height_mm)
    }

    fn from_figure(figure: &Figure, height_mm: f64) -> Result<ScaleFigure, String> {
        let posed = figure.posed(&standing());
        let (lo, hi) = posed.bounds().ok_or("the figure has no vertices")?;
        let model_height = f64::from(hi[1] - lo[1]);
        if model_height.is_nan() || model_height <= 0.0 || height_mm.is_nan() || height_mm <= 0.0 {
            return Err("the figure has no height".into());
        }
        let k = height_mm / model_height;
        let (s, c) = TURN_DEG.to_radians().sin_cos();
        let (cx, cz) = (
            f64::from(lo[0] + hi[0]) * 0.5,
            f64::from(lo[2] + hi[2]) * 0.5,
        );
        // glTF's (x, y, z), Y up, facing +Z, becomes ours (x, -z, y): Z up, facing -Y (toward
        // the front view), his left still +X. Then the turn about Z.
        let turn = |x: f64, y: f64, z: f64| Vec3::new(x * c - y * s, x * s + y * c, z);
        let positions = posed
            .positions
            .iter()
            .map(|p| {
                turn(
                    (f64::from(p[0]) - cx) * k,
                    -(f64::from(p[2]) - cz) * k,
                    (f64::from(p[1]) - f64::from(lo[1])) * k,
                )
            })
            .collect();
        let normals = posed
            .normals
            .iter()
            .map(|n| turn(f64::from(n[0]), -f64::from(n[2]), f64::from(n[1])))
            .collect();
        let half_width_mm = f64::from((hi[0] - lo[0]).max(hi[2] - lo[2])) * 0.5 * k;
        Ok(ScaleFigure {
            positions,
            normals,
            triangles: figure.triangles().to_vec(),
            height_mm,
            half_width_mm,
            placed: None,
        })
    }

    /// Where he stands beside a design whose box is `design` (min, max), or beside the origin
    /// for an empty design: on its right, a step away, on the ground (z = 0) or at the design's
    /// lowest point if that is lower.
    pub fn spot(&self, design: Option<(Vec3, Vec3)>) -> Vec3 {
        match design {
            Some((min, max)) => {
                let span = (max.x - min.x).max(max.y - min.y).max(0.0);
                let gap = 300.0 + 0.1 * span;
                Vec3::new(
                    max.x + gap + self.half_width_mm,
                    (min.y + max.y) * 0.5,
                    min.z.min(0.0),
                )
            }
            None => Vec3::new(300.0 + self.half_width_mm, 0.0, 0.0),
        }
    }

    /// The box he fills standing at `spot`.
    pub fn bounds_at(&self, spot: Vec3) -> Bounds {
        let w = self.half_width_mm;
        let mut b = Bounds::EMPTY;
        b.add(Vec3::new(spot.x - w, spot.y - w, spot.z));
        b.add(Vec3::new(spot.x + w, spot.y + w, spot.z + self.height_mm));
        b
    }

    /// His mesh standing at `spot`, for the scene (reused while the spot stays the same).
    pub fn mesh_at(&mut self, spot: Vec3) -> SceneMesh {
        let mesh = match &self.placed {
            Some((at, mesh)) if *at == spot => mesh.clone(),
            _ => {
                let mesh = Arc::new(Mesh {
                    positions: self
                        .positions
                        .iter()
                        .map(|p| Vec3::new(p.x + spot.x, p.y + spot.y, p.z + spot.z))
                        .collect(),
                    normals: self.normals.clone(),
                    triangles: self.triangles.clone(),
                    tri_face: vec![0; self.triangles.len()],
                    display_only_edges: true,
                    ..Mesh::default()
                });
                self.placed = Some((spot, mesh.clone()));
                mesh
            }
        };
        SceneMesh::solid(mesh, WOOD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sancho() -> ScaleFigure {
        ScaleFigure::placeholder(1680.0).expect("the placeholder poses")
    }

    #[test]
    fn he_stands_at_the_asked_height_on_the_ground() {
        for height in [1680.0, 1905.0, 1200.0] {
            let mut s = ScaleFigure::placeholder(height).expect("poses");
            let mesh = s.mesh_at(Vec3::new(0.0, 0.0, 0.0)).mesh;
            let lowest = mesh
                .positions
                .iter()
                .map(|p| p.z)
                .fold(f64::INFINITY, f64::min);
            let highest = mesh
                .positions
                .iter()
                .map(|p| p.z)
                .fold(f64::NEG_INFINITY, f64::max);
            assert!(lowest.abs() < 1.0, "feet at {lowest} mm");
            assert!(
                (highest - height).abs() < 1.0,
                "head at {highest} mm for {height}"
            );
        }
    }

    #[test]
    fn he_faces_the_front_view() {
        // The nose is the frontmost point of the head: toward -Y, turned a little toward the
        // design on his right (-X).
        let mut s = sancho();
        let mesh = s.mesh_at(Vec3::new(0.0, 0.0, 0.0)).mesh;
        let head: Vec<&Vec3> = mesh.positions.iter().filter(|p| p.z > 1500.0).collect();
        let front = head
            .iter()
            .min_by(|a, b| a.y.total_cmp(&b.y))
            .expect("a head");
        let back = head
            .iter()
            .max_by(|a, b| a.y.total_cmp(&b.y))
            .expect("a head");
        assert!(
            front.y < back.y - 150.0,
            "nose {front:?}, back of head {back:?}"
        );
        assert!(front.x < 0.0, "turned toward the design: {front:?}");
    }

    #[test]
    fn he_stands_beside_the_design_not_in_it() {
        let s = sancho();
        let design = (Vec3::new(-100.0, -50.0, 0.0), Vec3::new(100.0, 50.0, 20.0));
        let spot = s.spot(Some(design));
        let b = s.bounds_at(spot);
        assert!(b.min.x > design.1.x + 250.0, "a step to the right: {b:?}");
        assert_eq!(spot.z, 0.0);
        // A design below the ground: he stands at its lowest point.
        let sunk = s.spot(Some((
            Vec3::new(0.0, 0.0, -400.0),
            Vec3::new(10.0, 10.0, 0.0),
        )));
        assert_eq!(sunk.z, -400.0);
        // An empty design: beside the origin.
        assert!(s.spot(None).x > 300.0);
    }

    #[test]
    fn the_same_spot_reuses_the_mesh() {
        let mut s = sancho();
        let a = s.mesh_at(Vec3::new(500.0, 0.0, 0.0)).mesh;
        let b = s.mesh_at(Vec3::new(500.0, 0.0, 0.0)).mesh;
        let c = s.mesh_at(Vec3::new(900.0, 0.0, 0.0)).mesh;
        assert!(Arc::ptr_eq(&a, &b));
        assert!(!Arc::ptr_eq(&a, &c));
    }
}
