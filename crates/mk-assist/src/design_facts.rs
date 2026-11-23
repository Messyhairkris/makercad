//! The structure pack's facts, measured from boxes (doc 07 section 7.5).
//!
//! Written by Hoss (the Crew, campaign mk-core-34, 2026-11-22) from the contract
//! crew/contracts/mk_design_facts.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).
use crate::rules_structure::{Contact, Design, Fastener, Part, Shelf, Slot};
use std::collections::HashSet;

/// Contacts: two boxes count as touching when they are this close, in millimetres.
pub const CONTACT_TOLERANCE_MM: f64 = 1.0; // (v1 starting value, to tune)
/// A shelf must span at least this far between its supports, in millimetres.
pub const MIN_SHELF_SPAN_MM: f64 = 300.0; // (v1 starting value, to tune)
/// The id of the synthetic part that stands for the floor.
pub const FLOOR_ID: &str = "floor";

/// One part as the window measured it: an axis-aligned box in millimetres, Z up.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxPart {
    pub id: String,
    pub material: String,
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// The side lengths along x, y and z in millimetres; None when the part is unusable.
pub fn size(part: &BoxPart) -> Option<[f64; 3]> {
    if !is_usable(part, &mut HashSet::new()) {
        return None;
    }
    Some([
        part.max[0] - part.min[0],
        part.max[1] - part.min[1],
        part.max[2] - part.min[2],
    ])
}

/// The smallest side in millimetres; None when the part is unusable.
pub fn thickness_mm(part: &BoxPart) -> Option<f64> {
    let dims = size(part)?;
    Some(dims.iter().cloned().fold(f64::INFINITY, f64::min))
}

/// True when the two boxes touch after both grow by half the tolerance on every side.
pub fn touching(a: &BoxPart, b: &BoxPart) -> bool {
    if a == b {
        return false;
    }
    if !is_usable(a, &mut HashSet::new()) || !is_usable(b, &mut HashSet::new()) {
        return false;
    }
    let half = CONTACT_TOLERANCE_MM / 2.0;
    for i in 0..3 {
        if a.min[i] - half > b.max[i] + half {
            return false;
        }
        if b.min[i] - half > a.max[i] + half {
            return false;
        }
    }
    true
}

/// The usable parts in input order, each with its id, material and thickness.
pub fn parts_of(parts: &[BoxPart]) -> Vec<Part> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in parts {
        if !is_usable(p, &mut seen) {
            continue;
        }
        let thickness = thickness_mm(p).unwrap(); // safe: usable guarantees Some
        out.push(Part {
            id: p.id.clone(),
            material: p.material.clone(),
            thickness_mm: thickness,
        });
    }
    out
}

/// Every touching pair of usable parts once, earlier part as `a`, ordered by (a index, b index).
pub fn contacts(parts: &[BoxPart]) -> Vec<Contact> {
    let mut seen = HashSet::new();
    let mut usable: Vec<&BoxPart> = Vec::new();
    for p in parts {
        if is_usable(p, &mut seen) {
            usable.push(p);
        }
    }
    let mut out = Vec::new();
    for (i, a) in usable.iter().enumerate() {
        for b in usable.iter().skip(i + 1) {
            if touching(a, b) {
                out.push(Contact {
                    a: a.id.clone(),
                    b: b.id.clone(),
                });
            }
        }
    }
    out
}

/// The parts that really span two supports, loaded evenly along the span.
pub fn shelves(parts: &[BoxPart], load_kg_per_m: f64) -> Vec<Shelf> {
    if !load_kg_per_m.is_finite() || load_kg_per_m < 0.0 {
        return Vec::new();
    }
    let mut seen = HashSet::new();
    let mut usable: Vec<&BoxPart> = Vec::new();
    for p in parts {
        if is_usable(p, &mut seen) {
            usable.push(p);
        }
    }

    let mut out = Vec::new();

    for shelf_part in usable.iter() {
        // dimensions
        let dx = shelf_part.max[0] - shelf_part.min[0];
        let dy = shelf_part.max[1] - shelf_part.min[1];
        let dz = shelf_part.max[2] - shelf_part.min[2];

        // (a) smallest side must be Z and strictly smaller than both X and Y
        if !(dz < dx && dz < dy) {
            continue;
        }

        // Determine span axis and lengths
        let (span_axis, span_len, depth_len) = if dx >= dy {
            (0usize, dx, dy) // X is span
        } else {
            (1usize, dy, dx) // Y is span
        };

        // (b) span length requirement
        if span_len < MIN_SHELF_SPAN_MM {
            continue;
        }

        // (c) held at both ends
        let first_quarter = shelf_part.min[span_axis] + span_len / 4.0;
        let last_quarter = shelf_part.max[span_axis] - span_len / 4.0;

        let mut has_first = false;
        let mut has_last = false;

        for other in usable.iter() {
            if other.id == shelf_part.id {
                continue;
            }
            if !touching(shelf_part, other) {
                continue;
            }
            let centre = (other.min[span_axis] + other.max[span_axis]) / 2.0;
            if centre <= first_quarter {
                has_first = true;
            }
            if centre >= last_quarter {
                has_last = true;
            }
            if has_first && has_last {
                break;
            }
        }

        if !(has_first && has_last) {
            continue;
        }

        out.push(Shelf {
            part: shelf_part.id.clone(),
            span_mm: span_len,
            depth_mm: depth_len,
            load_kg_per_m,
        });
    }

    out
}

/// The whole design as the pack reads it.
pub fn design(
    parts: &[BoxPart],
    floor_z_mm: Option<f64>,
    load_kg_per_m: f64,
    fasteners: Vec<Fastener>,
    slots: Vec<Slot>,
) -> Design {
    let mut design = Design {
        parts: parts_of(parts),
        contacts: contacts(parts),
        shelves: shelves(parts, load_kg_per_m),
        fasteners,
        slots,
    };

    // Floor handling (A10)
    if let Some(z) = floor_z_mm {
        if z.is_finite() {
            // Append floor part
            design.parts.push(Part {
                id: FLOOR_ID.to_string(),
                material: "floor".to_string(),
                thickness_mm: f64::NAN,
            });

            // Determine contacts with floor
            let mut seen = HashSet::new();
            for p in parts {
                if !is_usable(p, &mut seen) {
                    continue;
                }
                let min_z = p.min[2];
                if (z - CONTACT_TOLERANCE_MM) <= min_z && min_z <= (z + CONTACT_TOLERANCE_MM) {
                    design.contacts.push(Contact {
                        a: p.id.clone(),
                        b: FLOOR_ID.to_string(),
                    });
                }
            }
        }
    }

    design
}

/// Helper: determines if a BoxPart is usable according to A1.
/// `seen` is mutated to track duplicate ids; the caller must provide a fresh set per overall pass.
fn is_usable(part: &BoxPart, seen: &mut HashSet<String>) -> bool {
    if part.id.is_empty() {
        return false;
    }
    if part.id == FLOOR_ID {
        return false;
    }
    for i in 0..3 {
        if !part.min[i].is_finite() || !part.max[i].is_finite() {
            return false;
        }
        // Both are finite here, so this is exactly "not max > min".
        if part.max[i] <= part.min[i] {
            return false;
        }
    }
    if !seen.insert(part.id.clone()) {
        // duplicate
        return false;
    }
    true
}
