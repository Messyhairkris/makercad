//! A skinned figure read from a binary glTF and posed on the CPU (decision D27: Sancho is a still,
//! posable scale figure that stands beside the design).
//!
//! It reads what a humanoid glTF holds for Sancho: the skinned mesh's triangles (`POSITION`,
//! `NORMAL`, `JOINTS_0`, `WEIGHTS_0`, indices), its skin, and the node tree's rest transforms. A
//! [`Pose`] is a short list of turns and aims, applied from the root out. [`Figure::posed`] skins
//! the mesh to that pose by linear blend skinning with the skin's inverse bind matrices, once per
//! pose, not per frame. Coordinates are glTF's: metres, +Y up, the figure faces +Z, its left is +X.
//!
//! A file is data from outside, so nothing here panics on a bad one. Every read is checked, and a
//! broken file is a [`FigureError`].

use std::fmt;

use serde_json::Value;

type V3 = [f32; 3];
/// A rotation as a unit quaternion, glTF's order: x, y, z, w.
type Q = [f32; 4];
/// A 4×4 matrix, column-major as glTF stores it.
type M4 = [f32; 16];

const IDENTITY: M4 = [
    1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
];
const NO_TURN: Q = [0., 0., 0., 1.];

/// Why a file is not a figure this module can pose.
#[derive(Clone, Debug, PartialEq)]
pub enum FigureError {
    /// Not a binary glTF 2.0 file, or its chunks are cut short.
    NotGlb,
    /// The JSON chunk does not parse.
    Json(String),
    /// Something the figure needs is missing or malformed (named).
    Missing(&'static str),
    /// This accessor reaches outside the binary chunk, or has a layout this reader does not take
    /// (sparse, another buffer, an unknown component type).
    Accessor(usize),
}

impl fmt::Display for FigureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FigureError::NotGlb => write!(f, "not a whole binary glTF 2.0 file"),
            FigureError::Json(e) => write!(f, "the glTF's JSON does not parse: {e}"),
            FigureError::Missing(what) => write!(f, "the figure has no usable {what}"),
            FigureError::Accessor(i) => write!(f, "glTF accessor {i} cannot be read"),
        }
    }
}

impl std::error::Error for FigureError {}

/// One node of the glTF's tree.
#[derive(Clone, Debug)]
struct Node {
    name: String,
    parent: Option<usize>,
    local: M4,
}

/// A figure: one skinned mesh and the skeleton that moves it.
#[derive(Clone, Debug)]
pub struct Figure {
    positions: Vec<V3>,
    normals: Vec<V3>,
    triangles: Vec<[u32; 3]>,
    /// Per vertex: up to four skin joints (indices into `skin`) and their weights. Empty for a
    /// mesh without a skin, which then never moves.
    joints: Vec<[u16; 4]>,
    weights: Vec<[f32; 4]>,
    nodes: Vec<Node>,
    /// Node indices, every parent before its children.
    order: Vec<usize>,
    /// The skin's joints: (node index, inverse bind matrix).
    skin: Vec<(usize, M4)>,
}

/// A pose: turns and aims applied in order. List them from the root out (upper arm before
/// forearm), because a step moves everything below its bone. A step naming a bone the figure
/// does not have is skipped, so one pose can serve several figures.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pose {
    steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Turn {
        bone: String,
        axis: V3,
        degrees: f32,
    },
    Aim {
        bone: String,
        tip: String,
        toward: V3,
    },
}

impl Pose {
    /// The figure as it was modelled (its bind pose).
    pub fn rest() -> Pose {
        Pose::default()
    }

    /// Turn `bone` (and everything below it) by `degrees` about `axis`, in the figure's space,
    /// around the bone's head.
    pub fn turn(mut self, bone: &str, axis: [f32; 3], degrees: f32) -> Pose {
        self.steps.push(Step::Turn {
            bone: bone.into(),
            axis,
            degrees,
        });
        self
    }

    /// Point `bone` so that the head of `tip` (a bone further down its chain) lies in the
    /// direction `toward` from the bone's head, turning it the shortest way.
    pub fn aim(mut self, bone: &str, tip: &str, toward: [f32; 3]) -> Pose {
        self.steps.push(Step::Aim {
            bone: bone.into(),
            tip: tip.into(),
            toward,
        });
        self
    }
}

/// Standing at ease beside the work: arms hanging by the sides, forearms a little forward. It is
/// written for the placeholder's bone names (BodyKun's Rigify `DEF-` bones, decision D26).
pub fn standing() -> Pose {
    Pose::rest()
        .aim("DEF-upper_arm.L", "DEF-forearm.L", [0.17, -1.0, -0.02])
        .aim("DEF-forearm.L", "DEF-hand.L", [0.05, -1.0, 0.2])
        .aim("DEF-upper_arm.R", "DEF-forearm.R", [-0.17, -1.0, -0.02])
        .aim("DEF-forearm.R", "DEF-hand.R", [-0.05, -1.0, 0.2])
}

/// A figure skinned to a pose.
#[derive(Clone, Debug)]
pub struct Posed {
    /// One per vertex, in metres.
    pub positions: Vec<[f32; 3]>,
    /// One per vertex, unit length.
    pub normals: Vec<[f32; 3]>,
    heads: Vec<(String, V3)>,
}

impl Posed {
    /// Where a bone's head is in this pose.
    pub fn head(&self, bone: &str) -> Option<[f32; 3]> {
        self.heads
            .iter()
            .find(|(name, _)| name == bone)
            .map(|(_, p)| *p)
    }

    /// The box around the posed mesh, (min, max); `None` for a mesh without vertices.
    pub fn bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        let first = *self.positions.first()?;
        Some(self.positions.iter().fold((first, first), |(lo, hi), p| {
            (
                [lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])],
                [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])],
            )
        }))
    }
}

impl Figure {
    /// Reads a figure from a binary glTF (`.glb`): the first mesh with a skin, or the first mesh.
    pub fn from_glb(bytes: &[u8]) -> Result<Figure, FigureError> {
        let (json, bin) = chunks(bytes)?;
        let doc: Value =
            serde_json::from_slice(json).map_err(|e| FigureError::Json(e.to_string()))?;
        let nodes = read_nodes(&doc)?;
        let order = parents_first(&nodes)?;

        // The mesh node: the first that has both a mesh and a skin, else the first with a mesh.
        let node_list = doc["nodes"]
            .as_array()
            .ok_or(FigureError::Missing("nodes"))?;
        let skinned = node_list
            .iter()
            .find(|n| n["mesh"].is_u64() && n["skin"].is_u64());
        let mesh_node = skinned
            .or_else(|| node_list.iter().find(|n| n["mesh"].is_u64()))
            .ok_or(FigureError::Missing("mesh"))?;
        let mesh =
            &doc["meshes"][as_index(&mesh_node["mesh"]).ok_or(FigureError::Missing("mesh"))?];
        let skin = match mesh_node["skin"].as_u64() {
            Some(s) => read_skin(
                &doc,
                bin,
                usize::try_from(s).map_err(|_| FigureError::Missing("skin"))?,
                nodes.len(),
            )?,
            None => Vec::new(),
        };

        let mut figure = Figure {
            positions: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::new(),
            joints: Vec::new(),
            weights: Vec::new(),
            nodes,
            order,
            skin,
        };
        let primitives = mesh["primitives"]
            .as_array()
            .ok_or(FigureError::Missing("mesh primitives"))?;
        for primitive in primitives {
            figure.add_primitive(&doc, bin, primitive)?;
        }
        if figure.positions.is_empty() || figure.triangles.is_empty() {
            return Err(FigureError::Missing("triangles"));
        }
        if !figure.joints.is_empty() && figure.joints.len() != figure.positions.len() {
            // Some primitives skinned and some not: the unskinned ones would fly apart.
            return Err(FigureError::Missing("skin weights on every primitive"));
        }
        Ok(figure)
    }

    /// How many vertices and triangles the mesh has.
    pub fn size(&self) -> (usize, usize) {
        (self.positions.len(), self.triangles.len())
    }

    /// The mesh's triangles (counter-clockwise seen from outside), indexing the posed vertices.
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    /// The skin's bone names, in the skin's order.
    pub fn bone_names(&self) -> Vec<&str> {
        self.skin
            .iter()
            .filter_map(|(node, _)| self.nodes.get(*node))
            .map(|n| n.name.as_str())
            .collect()
    }

    /// The figure skinned to `pose`.
    pub fn posed(&self, pose: &Pose) -> Posed {
        let mut turns: Vec<Q> = vec![NO_TURN; self.nodes.len()];
        for step in &pose.steps {
            match step {
                Step::Turn {
                    bone,
                    axis,
                    degrees,
                } => {
                    if let (Some(n), Some(q)) =
                        (self.node(bone), axis_angle(*axis, degrees.to_radians()))
                    {
                        if let Some(t) = turns.get_mut(n) {
                            *t = q_mul(q, *t);
                        }
                    }
                }
                Step::Aim { bone, tip, toward } => {
                    let (Some(n), Some(t)) = (self.node(bone), self.node(tip)) else {
                        continue;
                    };
                    let g = self.globals(&turns);
                    let (Some(from), Some(to)) =
                        (g.get(n).map(translation), g.get(t).map(translation))
                    else {
                        continue;
                    };
                    if let (Some(q), Some(turn)) =
                        (from_to(sub(to, from), *toward), turns.get_mut(n))
                    {
                        *turn = q_mul(q, *turn);
                    }
                }
            }
        }
        let g = self.globals(&turns);
        let skin: Vec<M4> = self
            .skin
            .iter()
            .map(|(node, ibm)| m_mul(g.get(*node).unwrap_or(&IDENTITY), ibm))
            .collect();
        let mut positions = Vec::with_capacity(self.positions.len());
        let mut normals = Vec::with_capacity(self.positions.len());
        for (v, (&p, &n)) in self.positions.iter().zip(&self.normals).enumerate() {
            let (Some(js), Some(ws)) = (self.joints.get(v), self.weights.get(v)) else {
                positions.push(p);
                normals.push(n);
                continue;
            };
            let (mut sp, mut sn, mut total) = ([0.0f32; 3], [0.0f32; 3], 0.0f32);
            for (&j, &w) in js.iter().zip(ws) {
                let Some(m) = skin.get(usize::from(j)) else {
                    continue;
                };
                if w <= 0.0 {
                    continue;
                }
                sp = add(sp, scale(point(m, p), w));
                sn = add(sn, scale(direction(m, n), w));
                total += w;
            }
            if total > 0.0 {
                positions.push(scale(sp, 1.0 / total));
                normals.push(normalize(sn).unwrap_or(n));
            } else {
                positions.push(p);
                normals.push(n);
            }
        }
        let heads = self
            .skin
            .iter()
            .filter_map(|(node, _)| {
                Some((
                    self.nodes.get(*node)?.name.clone(),
                    translation(g.get(*node)?),
                ))
            })
            .collect();
        Posed {
            positions,
            normals,
            heads,
        }
    }

    fn node(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.name == name)
    }

    /// Every node's matrix in the figure's space, with `turns` (per node, in the figure's space,
    /// about the node's own origin) applied from the root out.
    fn globals(&self, turns: &[Q]) -> Vec<M4> {
        let mut g = vec![IDENTITY; self.nodes.len()];
        for &n in &self.order {
            let Some(node) = self.nodes.get(n) else {
                continue;
            };
            let base = match node.parent.and_then(|p| g.get(p)) {
                Some(parent) => m_mul(parent, &node.local),
                None => node.local,
            };
            let placed = match turns.get(n) {
                Some(q) if *q != NO_TURN => m_mul(&about(translation(&base), *q), &base),
                _ => base,
            };
            if let Some(slot) = g.get_mut(n) {
                *slot = placed;
            }
        }
        g
    }

    fn add_primitive(
        &mut self,
        doc: &Value,
        bin: &[u8],
        primitive: &Value,
    ) -> Result<(), FigureError> {
        if primitive["mode"].as_u64().unwrap_or(4) != 4 {
            return Err(FigureError::Missing("triangle primitives"));
        }
        let attributes = &primitive["attributes"];
        let base = u32::try_from(self.positions.len())
            .map_err(|_| FigureError::Missing("a mesh small enough"))?;
        let positions = vec3s(
            doc,
            bin,
            as_index(&attributes["POSITION"]).ok_or(FigureError::Missing("POSITION"))?,
        )?;
        let count = positions.len();
        let triangles: Vec<[u32; 3]> = match as_index(&primitive["indices"]) {
            Some(i) => {
                let flat = Accessor::open(doc, bin, i, 1)?.uints(i)?;
                flat.as_chunks::<3>().0.to_vec()
            }
            None => {
                let n = u32::try_from(count / 3 * 3)
                    .map_err(|_| FigureError::Missing("a mesh small enough"))?;
                (0..n).step_by(3).map(|i| [i, i + 1, i + 2]).collect()
            }
        };
        if triangles
            .iter()
            .flatten()
            .any(|&i| usize::try_from(i).ok().is_none_or(|i| i >= count))
        {
            return Err(FigureError::Missing("triangle indices inside the mesh"));
        }
        let normals = match as_index(&attributes["NORMAL"]) {
            Some(i) => vec3s(doc, bin, i)?,
            None => face_normals(&positions, &triangles),
        };
        if normals.len() != count {
            return Err(FigureError::Missing("one NORMAL per POSITION"));
        }
        if let (Some(j), Some(w)) = (
            as_index(&attributes["JOINTS_0"]),
            as_index(&attributes["WEIGHTS_0"]),
        ) {
            let joints = Accessor::open(doc, bin, j, 4)?.uints(j)?;
            let weights = Accessor::open(doc, bin, w, 4)?.floats(w)?;
            if joints.len() != count * 4 || weights.len() != count * 4 {
                return Err(FigureError::Missing(
                    "one JOINTS_0 and WEIGHTS_0 per POSITION",
                ));
            }
            for (js, ws) in joints
                .as_chunks::<4>()
                .0
                .iter()
                .zip(weights.as_chunks::<4>().0)
            {
                // A joint index past u16 matches no skin joint, so that influence is skipped.
                self.joints
                    .push(js.map(|x| u16::try_from(x).unwrap_or(u16::MAX)));
                self.weights.push(*ws);
            }
        }
        self.triangles
            .extend(triangles.iter().map(|t| t.map(|i| i.saturating_add(base))));
        self.positions.extend(positions);
        self.normals.extend(normals);
        Ok(())
    }
}

/// The JSON and BIN chunks of a `.glb`.
fn chunks(b: &[u8]) -> Result<(&[u8], &[u8]), FigureError> {
    let u32_at = |at: usize| -> Option<usize> {
        let bytes: [u8; 4] = b.get(at..at.checked_add(4)?)?.try_into().ok()?;
        usize::try_from(u32::from_le_bytes(bytes)).ok()
    };
    if b.get(0..4) != Some(b"glTF") || u32_at(4) != Some(2) {
        return Err(FigureError::NotGlb);
    }
    let json_len = u32_at(12).ok_or(FigureError::NotGlb)?;
    if b.get(16..20) != Some(b"JSON") {
        return Err(FigureError::NotGlb);
    }
    let json_end = 20usize.checked_add(json_len).ok_or(FigureError::NotGlb)?;
    let json = b.get(20..json_end).ok_or(FigureError::NotGlb)?;
    // The BIN chunk is optional in glTF; a figure needs it.
    let bin_len = u32_at(json_end).ok_or(FigureError::NotGlb)?;
    let bin_start = json_end.checked_add(8).ok_or(FigureError::NotGlb)?;
    if b.get(json_end + 4..bin_start) != Some(b"BIN\0") {
        return Err(FigureError::NotGlb);
    }
    let bin = b
        .get(bin_start..bin_start.checked_add(bin_len).ok_or(FigureError::NotGlb)?)
        .ok_or(FigureError::NotGlb)?;
    Ok((json, bin))
}

fn as_index(v: &Value) -> Option<usize> {
    v.as_u64().and_then(|i| usize::try_from(i).ok())
}

fn floats_of(v: &Value, n: usize) -> Option<Vec<f32>> {
    let a = v.as_array()?;
    (a.len() == n).then(|| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect())
}

fn read_nodes(doc: &Value) -> Result<Vec<Node>, FigureError> {
    let list = doc["nodes"]
        .as_array()
        .ok_or(FigureError::Missing("nodes"))?;
    let mut nodes: Vec<Node> = list
        .iter()
        .map(|n| {
            let local = match floats_of(&n["matrix"], 16) {
                Some(m) => m.try_into().unwrap_or(IDENTITY),
                None => {
                    let t = floats_of(&n["translation"], 3).unwrap_or_else(|| vec![0.0; 3]);
                    let r = floats_of(&n["rotation"], 4).unwrap_or_else(|| NO_TURN.to_vec());
                    let s = floats_of(&n["scale"], 3).unwrap_or_else(|| vec![1.0; 3]);
                    trs(
                        [t[0], t[1], t[2]],
                        normalize4([r[0], r[1], r[2], r[3]]),
                        [s[0], s[1], s[2]],
                    )
                }
            };
            Node {
                name: n["name"].as_str().unwrap_or("").to_string(),
                parent: None,
                local,
            }
        })
        .collect();
    for (p, n) in list.iter().enumerate() {
        for child in n["children"].as_array().into_iter().flatten() {
            let c = as_index(child)
                .filter(|&c| c < nodes.len() && c != p)
                .ok_or(FigureError::Missing("a valid node tree"))?;
            if let Some(node) = nodes.get_mut(c) {
                node.parent = Some(p);
            }
        }
    }
    Ok(nodes)
}

/// Node indices with every parent before its children; a loop in the tree is an error.
fn parents_first(nodes: &[Node]) -> Result<Vec<usize>, FigureError> {
    let mut depth = vec![0usize; nodes.len()];
    for (i, d) in depth.iter_mut().enumerate() {
        let mut at = i;
        while let Some(p) = nodes.get(at).and_then(|n| n.parent) {
            *d += 1;
            if *d > nodes.len() {
                return Err(FigureError::Missing("a node tree without loops"));
            }
            at = p;
        }
    }
    let mut order: Vec<usize> = (0..nodes.len()).collect();
    order.sort_by_key(|&i| depth.get(i).copied().unwrap_or(0));
    Ok(order)
}

fn read_skin(
    doc: &Value,
    bin: &[u8],
    index: usize,
    node_count: usize,
) -> Result<Vec<(usize, M4)>, FigureError> {
    let skin = &doc["skins"][index];
    let joints: Vec<usize> = skin["joints"]
        .as_array()
        .ok_or(FigureError::Missing("skin joints"))?
        .iter()
        .map(|j| {
            as_index(j)
                .filter(|&j| j < node_count)
                .ok_or(FigureError::Missing("skin joints that are nodes"))
        })
        .collect::<Result<_, _>>()?;
    if joints.len() > usize::from(u16::MAX) {
        return Err(FigureError::Missing("a skin small enough"));
    }
    let inverse = match as_index(&skin["inverseBindMatrices"]) {
        Some(a) => Accessor::open(doc, bin, a, 16)?.floats(a)?,
        None => joints.iter().flat_map(|_| IDENTITY).collect(),
    };
    if inverse.len() != joints.len() * 16 {
        return Err(FigureError::Missing("one inverse bind matrix per joint"));
    }
    Ok(joints
        .into_iter()
        .zip(inverse.as_chunks::<16>().0)
        .map(|(j, m)| (j, *m))
        .collect())
}

fn vec3s(doc: &Value, bin: &[u8], index: usize) -> Result<Vec<V3>, FigureError> {
    let flat = Accessor::open(doc, bin, index, 3)?.floats(index)?;
    Ok(flat.as_chunks::<3>().0.to_vec())
}

/// Vertex normals from the triangles, for a mesh that has none.
fn face_normals(positions: &[V3], triangles: &[[u32; 3]]) -> Vec<V3> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for t in triangles {
        let p = |i: u32| {
            usize::try_from(i)
                .ok()
                .and_then(|i| positions.get(i))
                .copied()
                .unwrap_or([0.0; 3])
        };
        let n = cross(sub(p(t[1]), p(t[0])), sub(p(t[2]), p(t[0])));
        for &i in t {
            if let Some(slot) = usize::try_from(i).ok().and_then(|i| normals.get_mut(i)) {
                *slot = add(*slot, n);
            }
        }
    }
    normals
        .into_iter()
        .map(|n| normalize(n).unwrap_or([0.0, 1.0, 0.0]))
        .collect()
}

/// A checked view of one accessor's elements inside the binary chunk.
struct Accessor<'a> {
    bin: &'a [u8],
    start: usize,
    stride: usize,
    count: usize,
    comps: usize,
    kind: u64,
    normalized: bool,
}

impl<'a> Accessor<'a> {
    fn open(
        doc: &Value,
        bin: &'a [u8],
        index: usize,
        comps: usize,
    ) -> Result<Accessor<'a>, FigureError> {
        let bad = || FigureError::Accessor(index);
        let a = &doc["accessors"][index];
        let want = match comps {
            1 => "SCALAR",
            3 => "VEC3",
            4 => "VEC4",
            16 => "MAT4",
            _ => return Err(bad()),
        };
        if a["type"].as_str() != Some(want) || a.get("sparse").is_some() {
            return Err(bad());
        }
        let kind = a["componentType"].as_u64().ok_or_else(bad)?;
        let size = component_size(kind).ok_or_else(bad)?;
        let count = as_index(&a["count"]).ok_or_else(bad)?;
        let view = &doc["bufferViews"][as_index(&a["bufferView"]).ok_or_else(bad)?];
        if view["buffer"].as_u64() != Some(0) {
            return Err(bad());
        }
        let view_start = as_index(&view["byteOffset"]).unwrap_or(0);
        let view_len = as_index(&view["byteLength"]).ok_or_else(bad)?;
        let element = comps.checked_mul(size).ok_or_else(bad)?;
        let stride = as_index(&view["byteStride"]).unwrap_or(element);
        if stride < element {
            return Err(bad());
        }
        let offset = as_index(&a["byteOffset"]).unwrap_or(0);
        let need = match count {
            0 => 0,
            n => (n - 1)
                .checked_mul(stride)
                .and_then(|x| x.checked_add(element))
                .ok_or_else(bad)?,
        };
        let view_end = view_start.checked_add(view_len).ok_or_else(bad)?;
        let start = view_start.checked_add(offset).ok_or_else(bad)?;
        let end = start.checked_add(need).ok_or_else(bad)?;
        if end > view_end || end > bin.len() {
            return Err(bad());
        }
        Ok(Accessor {
            bin,
            start,
            stride,
            count,
            comps,
            kind,
            normalized: a["normalized"].as_bool().unwrap_or(false),
        })
    }

    fn value(&self, element: usize, comp: usize) -> Option<f64> {
        let size = component_size(self.kind)?;
        let at = self
            .start
            .checked_add(element.checked_mul(self.stride)?)?
            .checked_add(comp.checked_mul(size)?)?;
        let bytes = self.bin.get(at..at.checked_add(size)?)?;
        let raw = match (self.kind, bytes) {
            (5120, [b]) => f64::from(i8::from_le_bytes([*b])),
            (5121, [b]) => f64::from(*b),
            (5122, [a, b]) => f64::from(i16::from_le_bytes([*a, *b])),
            (5123, [a, b]) => f64::from(u16::from_le_bytes([*a, *b])),
            (5125, [a, b, c, d]) => f64::from(u32::from_le_bytes([*a, *b, *c, *d])),
            (5126, [a, b, c, d]) => f64::from(f32::from_le_bytes([*a, *b, *c, *d])),
            _ => return None,
        };
        Some(match (self.normalized, self.kind) {
            (true, 5120) => (raw / 127.0).max(-1.0),
            (true, 5121) => raw / 255.0,
            (true, 5122) => (raw / 32767.0).max(-1.0),
            (true, 5123) => raw / 65535.0,
            _ => raw,
        })
    }

    fn floats(&self, index: usize) -> Result<Vec<f32>, FigureError> {
        let mut out = Vec::with_capacity(self.count.saturating_mul(self.comps));
        for e in 0..self.count {
            for c in 0..self.comps {
                let v = self.value(e, c).ok_or(FigureError::Accessor(index))?;
                if !v.is_finite() {
                    return Err(FigureError::Accessor(index));
                }
                out.push(v as f32);
            }
        }
        Ok(out)
    }

    fn uints(&self, index: usize) -> Result<Vec<u32>, FigureError> {
        if !matches!(self.kind, 5121 | 5123 | 5125) {
            return Err(FigureError::Accessor(index));
        }
        let mut out = Vec::with_capacity(self.count.saturating_mul(self.comps));
        for e in 0..self.count {
            for c in 0..self.comps {
                let v = self.value(e, c).ok_or(FigureError::Accessor(index))?;
                out.push(v as u32);
            }
        }
        Ok(out)
    }
}

fn component_size(kind: u64) -> Option<usize> {
    match kind {
        5120 | 5121 => Some(1),
        5122 | 5123 => Some(2),
        5125 | 5126 => Some(4),
        _ => None,
    }
}

// Small vector, quaternion and matrix helpers (glTF conventions).

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: V3) -> Option<V3> {
    let len = dot(a, a).sqrt();
    (len > 1e-12 && len.is_finite()).then(|| scale(a, 1.0 / len))
}

fn normalize4(q: Q) -> Q {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len > 1e-12 && len.is_finite() {
        [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
    } else {
        NO_TURN
    }
}

/// `a` after `b`: rotating by the product is rotating by `b`, then by `a`.
fn q_mul(a: Q, b: Q) -> Q {
    normalize4([
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ])
}

fn axis_angle(axis: V3, radians: f32) -> Option<Q> {
    let n = normalize(axis)?;
    let (s, c) = (radians * 0.5).sin_cos();
    Some([n[0] * s, n[1] * s, n[2] * s, c])
}

/// The shortest turn taking direction `a` to direction `b`.
fn from_to(a: V3, b: V3) -> Option<Q> {
    let (a, b) = (normalize(a)?, normalize(b)?);
    let d = dot(a, b);
    if d > 1.0 - 1e-6 {
        return Some(NO_TURN);
    }
    if d < -1.0 + 1e-6 {
        let other = if a[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let axis = normalize(cross(a, other))?;
        return Some([axis[0], axis[1], axis[2], 0.0]);
    }
    let c = cross(a, b);
    Some(normalize4([c[0], c[1], c[2], 1.0 + d]))
}

fn trs(t: V3, q: Q, s: V3) -> M4 {
    let [x, y, z, w] = q;
    [
        (1.0 - 2.0 * (y * y + z * z)) * s[0],
        (2.0 * (x * y + w * z)) * s[0],
        (2.0 * (x * z - w * y)) * s[0],
        0.0,
        (2.0 * (x * y - w * z)) * s[1],
        (1.0 - 2.0 * (x * x + z * z)) * s[1],
        (2.0 * (y * z + w * x)) * s[1],
        0.0,
        (2.0 * (x * z + w * y)) * s[2],
        (2.0 * (y * z - w * x)) * s[2],
        (1.0 - 2.0 * (x * x + y * y)) * s[2],
        0.0,
        t[0],
        t[1],
        t[2],
        1.0,
    ]
}

/// Turning by `q` about the point `pivot`.
fn about(pivot: V3, q: Q) -> M4 {
    let r = trs([0.0; 3], q, [1.0; 3]);
    let moved = direction(&r, pivot);
    let mut m = r;
    m[12] = pivot[0] - moved[0];
    m[13] = pivot[1] - moved[1];
    m[14] = pivot[2] - moved[2];
    m
}

fn m_mul(a: &M4, b: &M4) -> M4 {
    let mut r = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            r[col * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[col * 4 + k]).sum();
        }
    }
    r
}

fn point(m: &M4, p: V3) -> V3 {
    add(direction(m, p), [m[12], m[13], m[14]])
}

fn direction(m: &M4, v: V3) -> V3 {
    [
        m[0] * v[0] + m[4] * v[1] + m[8] * v[2],
        m[1] * v[0] + m[5] * v[1] + m[9] * v[2],
        m[2] * v[0] + m[6] * v[1] + m[10] * v[2],
    ]
}

fn translation(m: &M4) -> V3 {
    [m[12], m[13], m[14]]
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLACEHOLDER: &[u8] = include_bytes!("../assets/sancho-bodykun.glb");

    fn figure() -> Figure {
        Figure::from_glb(PLACEHOLDER).expect("the placeholder reads")
    }

    fn height(p: &Posed) -> (f32, f32) {
        let (lo, hi) = p.bounds().expect("vertices");
        (lo[1], hi[1])
    }

    #[test]
    fn it_reads_the_placeholder() {
        let f = figure();
        assert_eq!(
            f.size(),
            (23_164, 34_128),
            "glTF splits vertices at seams: Blender counts 17,230"
        );
        assert_eq!(f.bone_names().len(), 71);
        assert!(f.bone_names().contains(&"DEF-f_middle.03.L"));
    }

    #[test]
    fn the_rest_pose_is_the_modelled_figure() {
        // Skinning to the rest pose must give back the file's own vertices: this checks the node
        // tree, the inverse bind matrices and the blend all agree.
        let f = figure();
        let p = f.posed(&Pose::rest());
        let worst = f
            .positions
            .iter()
            .zip(&p.positions)
            .map(|(a, b)| dot(sub(*a, *b), sub(*a, *b)).sqrt())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-4, "a vertex moved {worst} m at rest");
        let (floor, top) = height(&p);
        assert!(
            floor.abs() < 1e-3 && (top - 1.68).abs() < 1e-3,
            "{floor}..{top}"
        );
    }

    #[test]
    fn standing_lowers_the_arms_and_keeps_the_height() {
        let f = figure();
        let rest = f.posed(&Pose::rest());
        let stand = f.posed(&standing());
        let (floor, top) = height(&stand);
        assert!(floor.abs() < 0.005, "feet at {floor}");
        assert!((top - height(&rest).1).abs() < 0.01, "head top {top}");
        for side in ["L", "R"] {
            let shoulder = stand
                .head(&format!("DEF-upper_arm.{side}"))
                .expect("upper arm");
            let hand = stand.head(&format!("DEF-hand.{side}")).expect("hand");
            assert!(
                hand[1] < shoulder[1] - 0.4,
                "{side} hand at {hand:?}, shoulder at {shoulder:?}"
            );
            assert!(hand[0].abs() < 0.32, "{side} hand by the side: {hand:?}");
            assert!(
                hand[0].signum() == shoulder[0].signum(),
                "{side} hand stays on its side"
            );
        }
        // The legs are not posed.
        assert_eq!(stand.head("DEF-foot.L"), rest.head("DEF-foot.L"));
    }

    #[test]
    fn a_turn_pivots_on_the_bone_head() {
        let f = figure();
        let rest = f.posed(&Pose::rest());
        let bent = f.posed(&Pose::rest().turn("DEF-forearm.L", [0.0, 0.0, 1.0], 90.0));
        let near = |a: [f32; 3], b: [f32; 3]| dot(sub(a, b), sub(a, b)).sqrt() < 1e-4;
        assert!(near(
            bent.head("DEF-forearm.L").expect("forearm"),
            rest.head("DEF-forearm.L").expect("forearm")
        ));
        assert!(!near(
            bent.head("DEF-hand.L").expect("hand"),
            rest.head("DEF-hand.L").expect("hand")
        ));
        // An unknown bone is skipped, not an error.
        let same = f.posed(&Pose::rest().turn("no-such-bone", [1.0, 0.0, 0.0], 45.0));
        assert_eq!(same.positions, rest.positions);
    }

    #[test]
    fn broken_files_are_errors_not_crashes() {
        assert_eq!(Figure::from_glb(&[]).err(), Some(FigureError::NotGlb));
        assert_eq!(
            Figure::from_glb(b"glTF\x02\0\0\0junk").err(),
            Some(FigureError::NotGlb)
        );
        for cut in [20, 1_000, PLACEHOLDER.len() / 2, PLACEHOLDER.len() - 1] {
            assert!(
                Figure::from_glb(&PLACEHOLDER[..cut]).is_err(),
                "cut at {cut}"
            );
        }
        // Scribble over the JSON: either it no longer parses or it no longer makes a figure.
        let mut bad = PLACEHOLDER.to_vec();
        for b in bad.iter_mut().skip(40).step_by(97).take(200) {
            *b = b'#';
        }
        assert!(Figure::from_glb(&bad).is_err());
        // An accessor claiming more elements than the file holds (same length, so the GLB's
        // chunk lengths still add up).
        let needle = b"\"count\":23164";
        let at = PLACEHOLDER
            .windows(needle.len())
            .position(|w| w == needle)
            .expect("the vertex count is in the JSON");
        let mut huge = PLACEHOLDER.to_vec();
        huge[at + 8] = b'9';
        assert_eq!(
            Figure::from_glb(&huge)
                .err()
                .map(|e| matches!(e, FigureError::Accessor(_))),
            Some(true)
        );
    }
}
