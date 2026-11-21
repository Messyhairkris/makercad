//! The maker's stock (MakerCAD docs 04 and 05). So far: quick-add, which reads a line like
//! "15 pine 2x4s, 3 foot" or "half sheet 3/4 ply" into an item. The library, kits and
//! salvage come later.
//!
//! `quickadd.rs` is the Crew's module (campaign mk-core-3), landed byte-for-byte; it was
//! written against `crate::units`, which this re-export provides.

mod units {
    pub use mk_units::{nominal_lumber, parse_length, Unit};
}

pub mod quickadd;

pub use quickadd::{quick_add, Item, Kind};
