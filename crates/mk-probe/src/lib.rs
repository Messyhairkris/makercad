//! The capability probe (doc 12 section 12.2).
//!
//! `profile`: what the probe measured (backends tried in child processes, the viewport
//! benchmark, Sancho's cost, cores and memory) turned into the hardware profile, and the
//! profile's TOML text read and written. Running the probe itself comes next.

pub mod profile;
