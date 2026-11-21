//! The placeholder figure (decision D26): `assets/sancho-bodykun.glb`, BodyKun (CC0) rebuilt by
//! `tools/blender/export_bodykun.py`. These checks keep a re-export honest without a JSON parser.

use std::path::Path;

fn glb() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sancho-bodykun.glb");
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// The GLB's JSON chunk, as text.
fn json(b: &[u8]) -> String {
    let len = u32_at(b, 12) as usize;
    assert_eq!(&b[16..20], b"JSON", "the first chunk is the JSON chunk");
    String::from_utf8(b[20..20 + len].to_vec()).expect("the JSON chunk is UTF-8")
}

#[test]
fn it_is_a_whole_gltf_2_binary() {
    let b = glb();
    assert_eq!(&b[0..4], b"glTF");
    assert_eq!(u32_at(&b, 4), 2, "glTF version 2");
    assert_eq!(
        u32_at(&b, 8) as usize,
        b.len(),
        "the header's length is the file's"
    );
    assert!(json(&b).contains("\"version\":\"2.0\""));
}

#[test]
fn one_skin_of_71_deform_bones() {
    let j = json(&glb());
    assert_eq!(j.matches("\"skins\"").count(), 1);
    let start = j.find("\"joints\":[").expect("a skin with joints") + "\"joints\":[".len();
    let end = start + j[start..].find(']').expect("the joints list ends");
    let joints = j[start..end].split(',').count();
    assert_eq!(joints, 71, "BodyKun's DEF bones only");
    for bone in [
        "DEF-spine",
        "DEF-hand.R",
        "DEF-f_index.03.L",
        "DEF-thumb.01.R",
        "DEF-toe.L",
    ] {
        assert!(
            j.contains(&format!("\"name\":\"{bone}\"")),
            "{bone} is a node"
        );
    }
    assert!(!j.contains("\"name\":\"ORG-"), "no Rigify helper bones");
    assert!(!j.contains("\"name\":\"MCH-"), "no Rigify mechanism bones");
}

#[test]
fn it_has_the_two_clips_and_our_wood() {
    let j = json(&glb());
    assert!(j.contains("\"name\":\"Wave\""));
    assert!(j.contains("\"name\":\"Grip\""));
    assert!(j.contains("\"name\":\"Sancho wood\""));
    assert_eq!(j.matches("\"materials\"").count(), 1, "one material list");
}

#[test]
fn it_stands_on_the_floor_at_1_68_m() {
    // The body mesh's POSITION accessor: min y 0, max y 1.68 (glTF is Y-up, metres).
    let j = json(&glb());
    let max = j.find("\"max\":[").expect("an accessor with bounds");
    let tail = &j[max + 7..];
    let nums: Vec<f64> = tail[..tail.find(']').unwrap_or(0)]
        .split(',')
        .filter_map(|n| n.parse().ok())
        .collect();
    assert_eq!(nums.len(), 3, "the first bounded accessor is a position");
    assert!((nums[1] - 1.68).abs() < 1e-4, "height {} m", nums[1]);
}

#[test]
fn the_attribution_sidecar_names_the_source_and_licence() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sancho-bodykun.glb.attribution");
    let text = std::fs::read_to_string(path).expect("the sidecar is next to the asset");
    assert!(text.contains("vinchau"));
    assert!(text.contains("CC0"));
    assert!(text.contains("blendswap.com/blend/23521"));
}
