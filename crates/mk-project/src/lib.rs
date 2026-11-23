//! The `.maker` project file (doc 03 section 3.6): one zip-like container per project.
//!
//! `layout`: what each entry is, how it is compressed, whether its name is safe to extract,
//! and how the entries present compare with the manifest's hash index.
//!
//! `container`: the zip itself, entries in and out under those rules, and the content hash.
//!
//! `atomic`: saving so that a crash never leaves the maker with nothing (a temp file, one
//! recovery copy, a rename), through a `Disk` the app implements over the real folder.
//!
//! `maker`: saving and opening a whole project (manifest and intent as JSON, the index).
//!
//! `project`: the manifest (format version, environments, hash index) and the intent (name,
//! goal, success criteria, deadline, budget, skill), with their rules and a new project's defaults.
//!
//! `stage`: MakerCAD's scale classes stage table (doc 05 section 5.5).

pub mod atomic;
pub mod container;
pub mod layout;
pub mod maker;
pub mod project;
pub mod stage;
