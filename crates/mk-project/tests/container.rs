//! The `.maker` container: what goes in comes out, unsafe names never pass, and the limits hold.

use std::io::{Cursor, Write};

use mk_project::container::{content_hash, pack, unpack, ContainerError, Limits};
use mk_project::layout::NameProblem;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn sample() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("manifest.json", br#"{"format_version":1}"#.to_vec()),
        ("project/intent.json", br#"{"name":"Dresser"}"#.repeat(40)),
        ("designs/d1.sc", b"solidcraft design".to_vec()),
        ("log/journal.jsonl", b"{}\n{}\n".to_vec()),
        ("media/photo.png", vec![0x89, b'P', b'N', b'G', 0, 1, 2, 3]),
    ]
}

fn borrowed<'a>(entries: &'a [(&'static str, Vec<u8>)]) -> Vec<(&'static str, &'a [u8])> {
    entries.iter().map(|(n, b)| (*n, b.as_slice())).collect()
}

/// A zip written straight through the zip library, for files MakerCAD itself would refuse.
fn raw_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn what_goes_in_comes_out_in_order() {
    let entries = sample();
    let bytes = pack(&borrowed(&entries)).unwrap();
    let back = unpack(&bytes, Limits::default()).unwrap();
    let expected: Vec<(String, Vec<u8>)> = entries
        .into_iter()
        .map(|(n, b)| (n.to_string(), b))
        .collect();
    assert_eq!(back, expected);
}

#[test]
fn json_is_deflated_and_media_stored() {
    let entries = sample();
    let bytes = pack(&borrowed(&entries)).unwrap();
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let method = |archive: &mut ZipArchive<Cursor<Vec<u8>>>, i: usize| {
        archive.by_index(i).unwrap().compression()
    };
    assert_eq!(method(&mut archive, 0), CompressionMethod::Deflated);
    assert_eq!(method(&mut archive, 1), CompressionMethod::Deflated);
    assert_eq!(method(&mut archive, 4), CompressionMethod::Stored);
}

#[test]
fn the_same_project_packs_to_the_same_bytes() {
    let entries = sample();
    assert_eq!(
        pack(&borrowed(&entries)).unwrap(),
        pack(&borrowed(&entries)).unwrap()
    );
}

#[test]
fn unsafe_names_are_never_written() {
    for (name, problem) in [
        ("../escape.json", NameProblem::ParentSegment),
        ("/root.json", NameProblem::Absolute),
        ("a\\b.json", NameProblem::Backslash),
        ("", NameProblem::Empty),
        ("media/CON.png", NameProblem::Reserved),
    ] {
        let result = pack(&[("manifest.json", b"{}"), (name, b"x")]);
        assert_eq!(
            result,
            Err(ContainerError::BadName {
                name: name.to_string(),
                problem
            }),
            "{name:?}"
        );
    }
}

#[test]
fn names_that_differ_only_in_case_are_one_name() {
    let result = pack(&[("media/a.png", b"1"), ("media/A.PNG", b"2")]);
    assert_eq!(result, Err(ContainerError::Duplicate("media/A.PNG".into())));
}

#[test]
fn an_opened_file_with_an_unsafe_name_is_refused() {
    let bytes = raw_zip(&[("manifest.json", b"{}"), ("../../evil.txt", b"x")]);
    assert_eq!(
        unpack(&bytes, Limits::default()),
        Err(ContainerError::BadName {
            name: "../../evil.txt".into(),
            problem: NameProblem::ParentSegment
        })
    );
}

#[test]
fn an_opened_file_with_a_case_duplicate_is_refused() {
    let bytes = raw_zip(&[("log/journal.jsonl", b"1"), ("LOG/journal.jsonl", b"2")]);
    assert_eq!(
        unpack(&bytes, Limits::default()),
        Err(ContainerError::Duplicate("LOG/journal.jsonl".into()))
    );
}

#[test]
fn folder_entries_are_skipped() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_directory("media/", SimpleFileOptions::default())
        .unwrap();
    writer
        .start_file("media/a.png", SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"png").unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let back = unpack(&bytes, Limits::default()).unwrap();
    assert_eq!(back, vec![("media/a.png".to_string(), b"png".to_vec())]);
}

#[test]
fn a_small_file_that_inflates_past_the_limit_is_refused() {
    // 4 MB of zeros deflates to a few kilobytes.
    let zeros = vec![0u8; 4 << 20];
    let bytes = pack(&[("project/intent.json", zeros.as_slice())]).unwrap();
    assert!(bytes.len() < 64 << 10);
    let limits = Limits {
        max_entry_bytes: 1 << 20,
        ..Limits::default()
    };
    assert_eq!(
        unpack(&bytes, limits),
        Err(ContainerError::TooBig("project/intent.json".into()))
    );
}

#[test]
fn the_total_and_the_count_are_limited() {
    let entries = sample();
    let bytes = pack(&borrowed(&entries)).unwrap();
    let total = Limits {
        max_total_bytes: 100,
        ..Limits::default()
    };
    assert!(matches!(
        unpack(&bytes, total),
        Err(ContainerError::TooBig(_))
    ));
    let count = Limits {
        max_entries: 4,
        ..Limits::default()
    };
    assert_eq!(unpack(&bytes, count), Err(ContainerError::TooManyEntries));
}

#[test]
fn not_a_zip_is_a_zip_error_not_a_panic() {
    for junk in [&b""[..], b"PK", b"not a zip at all", &[0xff; 300]] {
        assert!(matches!(
            unpack(junk, Limits::default()),
            Err(ContainerError::Zip(_))
        ));
    }
}

#[test]
fn an_empty_project_packs_and_reads_back_empty() {
    let bytes = pack(&[]).unwrap();
    assert_eq!(unpack(&bytes, Limits::default()).unwrap(), vec![]);
}

#[test]
fn the_hash_is_sha256_in_lower_case_hex() {
    assert_eq!(
        content_hash(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        content_hash(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
