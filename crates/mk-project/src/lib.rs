//! The `.maker` project file (doc 03 section 3.6): one zip-like container per project.
//!
//! `layout`: what each entry is, how it is compressed, whether its name is safe to extract,
//! and how the entries present compare with the manifest's hash index.
//!
//! `atomic`: saving so that a crash never leaves the maker with nothing (a temp file, one
//! recovery copy, a rename), through a `Disk` the app implements over the real folder.
//!
//! `project`: the manifest (format version, environments, hash index) and the intent (name,
//! goal, success criteria, deadline, budget, skill), with their rules and a new project's defaults.

pub mod atomic;
pub mod layout;
pub mod project;
