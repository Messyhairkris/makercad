//! The workshop library (doc 04 section 4.3): the maker's own tools and materials, "global per
//! user (`library.db`, SQLite) and snapshotted per project". A kit adds its tools at once (then
//! the maker removes what they don't have); a material is a quick-add line, read and kept with
//! what it was understood as.
//!
//! Desktop only: SQLite is C code (the `rusqlite` crate with SQLite bundled), so this module is
//! not built for the browser (wasm32); the rest of the crate is.
//!
//! The file carries its schema version (`PRAGMA user_version`). An empty file is set up at
//! version 1; a file from a newer MakerCAD is refused, never changed.

use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::quickadd::{quick_add, Item, Kind};
use crate::taxonomy;
use crate::taxonomy_data::Access;

/// The schema this code writes and reads.
pub const SCHEMA_VERSION: i64 = 1;
/// The library's file name, in the app's data folder.
pub const FILE_NAME: &str = "library.db";

const SCHEMA_V1: &str = "
CREATE TABLE tools (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    access TEXT NOT NULL,
    accuracy_mm REAL,
    notes TEXT NOT NULL DEFAULT ''
);
CREATE TABLE materials (
    id INTEGER PRIMARY KEY,
    said TEXT NOT NULL,
    qty INTEGER NOT NULL,
    kind TEXT NOT NULL,
    material TEXT NOT NULL,
    section_a_mm REAL,
    section_b_mm REAL,
    length_mm REAL,
    thickness_mm REAL,
    sheet_a_mm REAL,
    sheet_b_mm REAL,
    condition TEXT,
    notes TEXT NOT NULL DEFAULT '',
    on_hand INTEGER NOT NULL DEFAULT 1
);
";

/// Why the library could not do something.
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryError {
    /// SQLite's own message.
    Sqlite(String),
    /// The file was written by a newer MakerCAD (its schema version); it is left untouched.
    NewerSchema(i64),
    /// No kit has this id.
    UnknownKit(String),
    /// A material line with nothing in it.
    Empty,
    /// A row this code cannot read (what was wrong).
    Corrupt(String),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::Sqlite(m) => write!(f, "the library file: {m}"),
            LibraryError::NewerSchema(v) => write!(
                f,
                "the library was written by a newer MakerCAD (version {v}); it was not changed"
            ),
            LibraryError::UnknownKit(k) => write!(f, "there is no kit called {k}"),
            LibraryError::Empty => write!(f, "nothing was typed"),
            LibraryError::Corrupt(m) => write!(f, "a row in the library could not be read: {m}"),
        }
    }
}

impl std::error::Error for LibraryError {}

impl From<rusqlite::Error> for LibraryError {
    fn from(e: rusqlite::Error) -> Self {
        LibraryError::Sqlite(e.to_string())
    }
}

/// One of the maker's tools.
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    pub id: i64,
    /// A tool kind id from the taxonomy, or free text for one it doesn't know.
    pub kind: String,
    pub label: String,
    pub access: Access,
    pub accuracy_mm: Option<f64>,
    pub notes: String,
}

/// One material line, as quick-add understood it.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub id: i64,
    pub item: Item,
    pub on_hand: bool,
}

/// The access words doc 04 section 4.3.1 uses.
pub fn access_key(access: Access) -> &'static str {
    match access {
        Access::Owned => "owned",
        Access::Shared => "shared",
        Access::Makerspace => "makerspace",
        Access::Rentable => "rentable",
        Access::Wishlist => "wishlist",
    }
}

/// The access a word names.
pub fn access_from(key: &str) -> Option<Access> {
    [
        Access::Owned,
        Access::Shared,
        Access::Makerspace,
        Access::Rentable,
        Access::Wishlist,
    ]
    .into_iter()
    .find(|a| access_key(*a) == key)
}

fn kind_key(kind: Kind) -> &'static str {
    match kind {
        Kind::Board => "board",
        Kind::Sheet => "sheet",
        Kind::Other => "other",
    }
}

fn kind_from(key: &str) -> Option<Kind> {
    match key {
        "board" => Some(Kind::Board),
        "sheet" => Some(Kind::Sheet),
        "other" => Some(Kind::Other),
        _ => None,
    }
}

/// The maker's library: one SQLite connection.
pub struct Library {
    conn: Connection,
}

impl Library {
    /// Opens (or makes) the library file at `path`, setting up an empty one.
    pub fn open(path: &Path) -> Result<Library, LibraryError> {
        Library::ready(Connection::open(path)?)
    }

    /// A library that lives in memory only (tests, and a first run with no folder).
    pub fn in_memory() -> Result<Library, LibraryError> {
        Library::ready(Connection::open_in_memory()?)
    }

    fn ready(conn: Connection) -> Result<Library, LibraryError> {
        let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(LibraryError::NewerSchema(version));
        }
        if version < 1 {
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(SCHEMA_V1)?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            tx.commit()?;
        }
        Ok(Library { conn })
    }

    /// The schema version of the open file.
    pub fn schema_version(&self) -> Result<i64, LibraryError> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }

    /// Adds a tool and returns its id. An empty label takes the taxonomy's name for the kind
    /// (or the kind itself, for a kind the taxonomy doesn't know).
    pub fn add_tool(
        &self,
        kind: &str,
        label: &str,
        access: Access,
        accuracy_mm: Option<f64>,
    ) -> Result<i64, LibraryError> {
        let label = match label.trim() {
            "" => taxonomy::tool(kind).map_or(kind, |t| t.name).to_string(),
            given => given.to_string(),
        };
        let accuracy = accuracy_mm.filter(|a| a.is_finite() && *a > 0.0);
        self.conn.execute(
            "INSERT INTO tools (kind, label, access, accuracy_mm) VALUES (?1, ?2, ?3, ?4)",
            params![kind, label, access_key(access), accuracy],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Adds every tool of a kit whose kind the library doesn't have yet, with the kit's access;
    /// returns how many were added.
    pub fn add_kit(&self, kit_id: &str) -> Result<usize, LibraryError> {
        let kit = taxonomy::kit(kit_id).ok_or_else(|| LibraryError::UnknownKit(kit_id.into()))?;
        let tx = self.conn.unchecked_transaction()?;
        let mut added = 0;
        for kind in kit.tools {
            let have: Option<i64> = tx
                .query_row(
                    "SELECT id FROM tools WHERE kind = ?1 LIMIT 1",
                    params![kind],
                    |row| row.get(0),
                )
                .optional()?;
            if have.is_none() {
                let name = taxonomy::tool(kind).map_or(*kind, |t| t.name);
                tx.execute(
                    "INSERT INTO tools (kind, label, access) VALUES (?1, ?2, ?3)",
                    params![kind, name, access_key(kit.access)],
                )?;
                added += 1;
            }
        }
        tx.commit()?;
        Ok(added)
    }

    /// Every tool, oldest first.
    pub fn tools(&self) -> Result<Vec<Tool>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, kind, label, access, accuracy_mm, notes FROM tools ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        let mut tools = Vec::new();
        for row in rows {
            let (id, kind, label, access, accuracy_mm, notes) = row?;
            let access = access_from(&access).ok_or_else(|| {
                LibraryError::Corrupt(format!("tool {id} has the access \"{access}\""))
            })?;
            tools.push(Tool {
                id,
                kind,
                label,
                access,
                accuracy_mm,
                notes,
            });
        }
        Ok(tools)
    }

    /// Changes how the maker has a tool; false when there is no such tool.
    pub fn set_tool_access(&self, id: i64, access: Access) -> Result<bool, LibraryError> {
        let n = self.conn.execute(
            "UPDATE tools SET access = ?1 WHERE id = ?2",
            params![access_key(access), id],
        )?;
        Ok(n > 0)
    }

    /// Removes a tool; false when there is no such tool.
    pub fn remove_tool(&self, id: i64) -> Result<bool, LibraryError> {
        Ok(self
            .conn
            .execute("DELETE FROM tools WHERE id = ?1", params![id])?
            > 0)
    }

    /// Reads a quick-add line ("15 pine 2x4s, 3 foot") and keeps it with what it was understood
    /// as.
    pub fn add_material(&self, said: &str) -> Result<Material, LibraryError> {
        if said.trim().is_empty() {
            return Err(LibraryError::Empty);
        }
        let item = quick_add(said);
        self.conn.execute(
            "INSERT INTO materials (said, qty, kind, material, section_a_mm, section_b_mm,
                length_mm, thickness_mm, sheet_a_mm, sheet_b_mm, condition, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                item.text,
                item.qty,
                kind_key(item.kind),
                item.material,
                item.section_mm.map(|s| s.0),
                item.section_mm.map(|s| s.1),
                item.length_mm,
                item.thickness_mm,
                item.sheet_mm.map(|s| s.0),
                item.sheet_mm.map(|s| s.1),
                item.condition,
                item.notes.join("\n"),
            ],
        )?;
        Ok(Material {
            id: self.conn.last_insert_rowid(),
            item,
            on_hand: true,
        })
    }

    /// Every material line, oldest first.
    pub fn materials(&self) -> Result<Vec<Material>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, said, qty, kind, material, section_a_mm, section_b_mm, length_mm,
                thickness_mm, sheet_a_mm, sheet_b_mm, condition, notes, on_hand
             FROM materials ORDER BY id",
        )?;
        let rows = stmt.query_map([], material_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row??);
        }
        Ok(out)
    }

    /// Changes how many of a material line there are; false when there is no such line.
    pub fn set_qty(&self, id: i64, qty: u32) -> Result<bool, LibraryError> {
        let n = self.conn.execute(
            "UPDATE materials SET qty = ?1 WHERE id = ?2",
            params![qty, id],
        )?;
        Ok(n > 0)
    }

    /// Marks a material line as on hand or not (used up, lent); false when there is no such line.
    pub fn set_on_hand(&self, id: i64, on_hand: bool) -> Result<bool, LibraryError> {
        let n = self.conn.execute(
            "UPDATE materials SET on_hand = ?1 WHERE id = ?2",
            params![on_hand, id],
        )?;
        Ok(n > 0)
    }

    /// Removes a material line; false when there is no such line.
    pub fn remove_material(&self, id: i64) -> Result<bool, LibraryError> {
        Ok(self
            .conn
            .execute("DELETE FROM materials WHERE id = ?1", params![id])?
            > 0)
    }
}

/// One materials row back into a Material (an inner error when a value can't be read).
fn material_row(row: &Row<'_>) -> rusqlite::Result<Result<Material, LibraryError>> {
    let id: i64 = row.get(0)?;
    let qty: i64 = row.get(2)?;
    let kind: String = row.get(3)?;
    let pair = |a: Option<f64>, b: Option<f64>| a.zip(b);
    let notes: String = row.get(12)?;
    let Ok(qty) = u32::try_from(qty) else {
        return Ok(Err(LibraryError::Corrupt(format!(
            "material {id} has the quantity {qty}"
        ))));
    };
    let Some(kind) = kind_from(&kind) else {
        return Ok(Err(LibraryError::Corrupt(format!(
            "material {id} has the kind \"{kind}\""
        ))));
    };
    let item = Item {
        qty,
        kind,
        material: row.get(4)?,
        section_mm: pair(row.get(5)?, row.get(6)?),
        length_mm: row.get(7)?,
        thickness_mm: row.get(8)?,
        sheet_mm: pair(row.get(9)?, row.get(10)?),
        condition: row.get(11)?,
        notes: notes
            .split('\n')
            .filter(|n| !n.is_empty())
            .map(String::from)
            .collect(),
        text: row.get(1)?,
    };
    Ok(Ok(Material {
        id,
        item,
        on_hand: row.get(13)?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "makercad-library-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_new_library_is_set_up_at_version_1_and_empty() {
        let lib = Library::in_memory().unwrap();
        assert_eq!(lib.schema_version().unwrap(), SCHEMA_VERSION);
        assert!(lib.tools().unwrap().is_empty());
        assert!(lib.materials().unwrap().is_empty());
    }

    #[test]
    fn tools_are_kept_with_their_access_and_accuracy() {
        let lib = Library::in_memory().unwrap();
        let jig = lib
            .add_tool("jigsaw", "Ryobi jigsaw", Access::Owned, Some(1.0))
            .unwrap();
        let laser = lib
            .add_tool("laser_cutter", "", Access::Makerspace, Some(f64::NAN))
            .unwrap();
        let tools = lib.tools().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].id, jig);
        assert_eq!(tools[0].label, "Ryobi jigsaw");
        assert_eq!(tools[0].accuracy_mm, Some(1.0));
        assert_eq!(tools[1].id, laser);
        assert_eq!(tools[1].access, Access::Makerspace);
        assert_eq!(tools[1].accuracy_mm, None, "NaN is not an accuracy");
        assert!(!tools[1].label.is_empty());
        assert!(lib.set_tool_access(jig, Access::Shared).unwrap());
        assert_eq!(lib.tools().unwrap()[0].access, Access::Shared);
        assert!(lib.remove_tool(jig).unwrap());
        assert!(!lib.remove_tool(jig).unwrap());
        assert_eq!(lib.tools().unwrap().len(), 1);
    }

    #[test]
    fn a_kit_adds_each_tool_kind_once() {
        let lib = Library::in_memory().unwrap();
        let first = lib.add_kit("basic_household").unwrap();
        assert!(first > 0);
        assert_eq!(lib.tools().unwrap().len(), first);
        assert_eq!(lib.add_kit("basic_household").unwrap(), 0);
        assert_eq!(
            lib.add_kit("no_such_kit"),
            Err(LibraryError::UnknownKit("no_such_kit".into()))
        );
        let kit = taxonomy::kit("basic_household").unwrap();
        assert!(lib.tools().unwrap().iter().all(|t| t.access == kit.access));
    }

    #[test]
    fn materials_come_back_as_quick_add_read_them() {
        let lib = Library::in_memory().unwrap();
        let lines = [
            "15 pine 2x4s, 3 foot",
            "half sheet 3/4 ply",
            "an old bookshelf",
        ];
        for line in lines {
            let m = lib.add_material(line).unwrap();
            assert_eq!(m.item, quick_add(line));
        }
        let back = lib.materials().unwrap();
        assert_eq!(back.len(), 3);
        for (m, line) in back.iter().zip(lines) {
            assert_eq!(m.item, quick_add(line), "{line}");
            assert!(m.on_hand);
        }
        assert_eq!(lib.add_material("   "), Err(LibraryError::Empty));
        assert!(lib.set_qty(back[0].id, 9).unwrap());
        assert!(lib.set_on_hand(back[1].id, false).unwrap());
        let again = lib.materials().unwrap();
        assert_eq!(again[0].item.qty, 9);
        assert!(!again[1].on_hand);
        assert!(lib.remove_material(back[2].id).unwrap());
        assert_eq!(lib.materials().unwrap().len(), 2);
        assert!(!lib.set_qty(back[2].id, 1).unwrap());
    }

    #[test]
    fn the_library_file_keeps_everything_between_runs() {
        let dir = scratch("reopen");
        let path = dir.join(FILE_NAME);
        {
            let lib = Library::open(&path).unwrap();
            lib.add_tool("jigsaw", "", Access::Owned, None).unwrap();
            lib.add_material("15 pine 2x4s, 3 foot").unwrap();
        }
        let lib = Library::open(&path).unwrap();
        assert_eq!(lib.tools().unwrap().len(), 1);
        assert_eq!(lib.materials().unwrap()[0].item.qty, 15);
        drop(lib);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_from_a_newer_makercad_is_refused_and_left_alone() {
        let dir = scratch("newer");
        let path = dir.join(FILE_NAME);
        {
            let conn = Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", 7).unwrap();
        }
        assert_eq!(
            Library::open(&path).err(),
            Some(LibraryError::NewerSchema(7))
        );
        let conn = Connection::open(&path).unwrap();
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, 7);
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn access_words_round_trip() {
        for a in [
            Access::Owned,
            Access::Shared,
            Access::Makerspace,
            Access::Rentable,
            Access::Wishlist,
        ] {
            assert_eq!(access_from(access_key(a)), Some(a));
        }
        assert_eq!(access_from("borrowed"), None);
    }
}
