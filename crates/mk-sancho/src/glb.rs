//! Writes Sancho as a binary glTF 2.0 file (`.glb`): one skinned mesh with two materials
//! (wood and the darker ball joints), the 51-joint skin, and the `Wave` and `Grip` animations.

use crate::{animations, body, skeleton, sub, Part};

const FLOAT: u32 = 5126;
const UNSIGNED_BYTE: u32 = 5121;
const UNSIGNED_INT: u32 = 5125;
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;

/// Wood (sRGB #D4A373) and darker joint wood (#A8784C), as linear base colours. The palette's
/// #E8C9A8 is a flat icon fill; lit in 3D it washed out to bone white, so the model is a
/// shade deeper.
const WOOD: [f32; 4] = [0.658, 0.366, 0.171, 1.0];
const JOINT_WOOD: [f32; 4] = [0.392, 0.188, 0.072, 1.0];

#[derive(Default)]
struct Builder {
    bin: Vec<u8>,
    views: Vec<String>,
    accessors: Vec<String>,
}

impl Builder {
    /// Adds raw bytes as a buffer view; returns the view's index.
    fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        let offset = self.bin.len();
        self.bin.extend_from_slice(bytes);
        let target = target
            .map(|t| format!(",\"target\":{t}"))
            .unwrap_or_default();
        self.views.push(format!(
            "{{\"buffer\":0,\"byteOffset\":{offset},\"byteLength\":{}{target}}}",
            bytes.len()
        ));
        self.views.len() - 1
    }

    /// Adds an accessor over a new view; returns the accessor's index.
    fn accessor(
        &mut self,
        bytes: &[u8],
        target: Option<u32>,
        component: u32,
        kind: &str,
        count: usize,
        bounds: &str,
    ) -> usize {
        let view = self.view(bytes, target);
        self.accessors.push(format!(
            "{{\"bufferView\":{view},\"componentType\":{component},\"count\":{count},\"type\":\"{kind}\"{bounds}}}"
        ));
        self.accessors.len() - 1
    }
}

fn num(x: f32) -> String {
    if x.is_finite() {
        format!("{x}")
    } else {
        "0".to_string()
    }
}

fn list(xs: &[f32]) -> String {
    xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(",")
}

fn floats(xs: impl IntoIterator<Item = f32>) -> Vec<u8> {
    xs.into_iter().flat_map(f32::to_le_bytes).collect()
}

/// The accessors of one primitive, as its JSON.
fn primitive(b: &mut Builder, part: &Part, material: usize) -> String {
    let count = part.positions.len();
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in &part.positions {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    if count == 0 {
        lo = [0.0; 3];
        hi = [0.0; 3];
    }
    let bounds = format!(",\"min\":[{}],\"max\":[{}]", list(&lo), list(&hi));
    let pos = b.accessor(
        &floats(part.positions.iter().flatten().copied()),
        Some(ARRAY_BUFFER),
        FLOAT,
        "VEC3",
        count,
        &bounds,
    );
    let nor = b.accessor(
        &floats(part.normals.iter().flatten().copied()),
        Some(ARRAY_BUFFER),
        FLOAT,
        "VEC3",
        count,
        "",
    );
    let joints: Vec<u8> = part.joints.iter().flat_map(|j| [*j, 0, 0, 0]).collect();
    let jnt = b.accessor(
        &joints,
        Some(ARRAY_BUFFER),
        UNSIGNED_BYTE,
        "VEC4",
        count,
        "",
    );
    let wgt = b.accessor(
        &floats(part.joints.iter().flat_map(|_| [1.0, 0.0, 0.0, 0.0])),
        Some(ARRAY_BUFFER),
        FLOAT,
        "VEC4",
        count,
        "",
    );
    let idx_bytes: Vec<u8> = part.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    let idx = b.accessor(
        &idx_bytes,
        Some(ELEMENT_ARRAY_BUFFER),
        UNSIGNED_INT,
        "SCALAR",
        part.indices.len(),
        "",
    );
    format!(
        "{{\"attributes\":{{\"POSITION\":{pos},\"NORMAL\":{nor},\"JOINTS_0\":{jnt},\"WEIGHTS_0\":{wgt}}},\"indices\":{idx},\"material\":{material}}}"
    )
}

/// Sancho at `height_m` metres as `.glb` bytes.
pub fn write_glb(height_m: f32) -> Vec<u8> {
    let joints = skeleton(height_m);
    let (wood, balls) = body(height_m);
    let anims = animations(&joints);
    let mut b = Builder::default();

    let prims = [primitive(&mut b, &wood, 0), primitive(&mut b, &balls, 1)];

    // Joint nodes (node i is joint i), then the mesh node.
    let mut nodes = Vec::new();
    for (i, j) in joints.iter().enumerate() {
        let local = match j.parent.and_then(|p| joints.get(p)) {
            Some(parent) => sub(j.rest, parent.rest),
            None => j.rest,
        };
        let children: Vec<String> = joints
            .iter()
            .enumerate()
            .filter(|(_, c)| c.parent == Some(i))
            .map(|(ci, _)| ci.to_string())
            .collect();
        let children = if children.is_empty() {
            String::new()
        } else {
            format!(",\"children\":[{}]", children.join(","))
        };
        nodes.push(format!(
            "{{\"name\":\"{}\",\"translation\":[{}]{children}}}",
            j.name,
            list(&local)
        ));
    }
    let mesh_node = nodes.len();
    nodes.push("{\"name\":\"Sancho\",\"mesh\":0,\"skin\":0}".to_string());

    // Inverse bind matrices: every rest pose is a pure translation.
    let ibm: Vec<f32> = joints
        .iter()
        .flat_map(|j| {
            let [x, y, z] = j.rest;
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -x, -y, -z, 1.0,
            ]
        })
        .collect();
    let ibm_acc = b.accessor(&floats(ibm), None, FLOAT, "MAT4", joints.len(), "");
    let joint_list = (0..joints.len())
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let mut anim_json = Vec::new();
    for a in &anims {
        let mut channels = Vec::new();
        let mut samplers = Vec::new();
        for t in &a.tracks {
            let (first, last) = (
                t.times.first().copied().unwrap_or(0.0),
                t.times.last().copied().unwrap_or(0.0),
            );
            let input = b.accessor(
                &floats(t.times.iter().copied()),
                None,
                FLOAT,
                "SCALAR",
                t.times.len(),
                &format!(",\"min\":[{}],\"max\":[{}]", num(first), num(last)),
            );
            let output = b.accessor(
                &floats(t.rotations.iter().flatten().copied()),
                None,
                FLOAT,
                "VEC4",
                t.rotations.len(),
                "",
            );
            let s = samplers.len();
            samplers.push(format!(
                "{{\"input\":{input},\"output\":{output},\"interpolation\":\"LINEAR\"}}"
            ));
            channels.push(format!(
                "{{\"sampler\":{s},\"target\":{{\"node\":{},\"path\":\"rotation\"}}}}",
                t.joint
            ));
        }
        anim_json.push(format!(
            "{{\"name\":\"{}\",\"channels\":[{}],\"samplers\":[{}]}}",
            a.name,
            channels.join(","),
            samplers.join(",")
        ));
    }

    while !b.bin.len().is_multiple_of(4) {
        b.bin.push(0);
    }
    let json = format!(
        concat!(
            "{{\"asset\":{{\"version\":\"2.0\",\"generator\":\"MakerCAD mk-sancho {ver}\"}},",
            "\"scene\":0,\"scenes\":[{{\"name\":\"Sancho\",\"nodes\":[0,{mesh_node}]}}],",
            "\"nodes\":[{nodes}],",
            "\"meshes\":[{{\"name\":\"Sancho\",\"primitives\":[{prims}]}}],",
            "\"materials\":[",
            "{{\"name\":\"wood\",\"pbrMetallicRoughness\":{{\"baseColorFactor\":[{wood}],\"metallicFactor\":0,\"roughnessFactor\":0.7}}}},",
            "{{\"name\":\"joint wood\",\"pbrMetallicRoughness\":{{\"baseColorFactor\":[{joint}],\"metallicFactor\":0,\"roughnessFactor\":0.55}}}}],",
            "\"skins\":[{{\"name\":\"Sancho rig\",\"skeleton\":0,\"joints\":[{joints}],\"inverseBindMatrices\":{ibm}}}],",
            "\"animations\":[{anims}],",
            "\"accessors\":[{accessors}],",
            "\"bufferViews\":[{views}],",
            "\"buffers\":[{{\"byteLength\":{len}}}]}}"
        ),
        ver = env!("CARGO_PKG_VERSION"),
        mesh_node = mesh_node,
        nodes = nodes.join(","),
        prims = prims.join(","),
        wood = list(&WOOD),
        joint = list(&JOINT_WOOD),
        joints = joint_list,
        ibm = ibm_acc,
        anims = anim_json.join(","),
        accessors = b.accessors.join(","),
        views = b.views.join(","),
        len = b.bin.len(),
    );
    let mut json = json.into_bytes();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let total = 12 + 8 + json.len() + 8 + b.bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json);
    out.extend_from_slice(&(b.bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&b.bin);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(b: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }

    #[test]
    fn the_container_is_a_valid_glb() {
        let glb = write_glb(1.68);
        assert_eq!(&glb[0..4], b"glTF");
        assert_eq!(u32_at(&glb, 4), 2);
        assert_eq!(u32_at(&glb, 8) as usize, glb.len());
        let json_len = u32_at(&glb, 12) as usize;
        assert_eq!(&glb[16..20], b"JSON");
        assert_eq!(json_len % 4, 0);
        let bin_at = 20 + json_len;
        let bin_len = u32_at(&glb, bin_at) as usize;
        assert_eq!(&glb[bin_at + 4..bin_at + 8], b"BIN\0");
        assert_eq!(bin_at + 8 + bin_len, glb.len());
        assert_eq!(bin_len % 4, 0);
    }

    #[test]
    fn the_json_names_the_rig_and_both_animations_and_has_no_nan() {
        let glb = write_glb(1.68);
        let json_len = u32_at(&glb, 12) as usize;
        let json = String::from_utf8(glb[20..20 + json_len].to_vec()).expect("utf-8");
        for want in [
            "\"skins\"",
            "\"Wave\"",
            "\"Grip\"",
            "\"index_l_3\"",
            "\"JOINTS_0\"",
            "\"inverseBindMatrices\"",
        ] {
            assert!(json.contains(want), "{want}");
        }
        assert!(!json.contains("NaN") && !json.contains("inf"));
        let opens = json.matches('{').count() + json.matches('[').count();
        let closes = json.matches('}').count() + json.matches(']').count();
        assert_eq!(opens, closes);
    }

    #[test]
    fn the_buffer_length_in_the_json_matches_the_bin_chunk() {
        let glb = write_glb(1.68);
        let json_len = u32_at(&glb, 12) as usize;
        let json = String::from_utf8(glb[20..20 + json_len].to_vec()).expect("utf-8");
        let bin_len = u32_at(&glb, 20 + json_len);
        assert!(
            json.contains(&format!("\"buffers\":[{{\"byteLength\":{bin_len}}}]")),
            "{bin_len}"
        );
    }
}
