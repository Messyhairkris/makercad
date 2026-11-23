"""Writes a sample workshop library (the SQLite file the Quest's "Use my workshop" reads).

    python tools/make_sample_library.py PATH

Then `set MAKERCAD_LIBRARY=PATH` and `makercad --quest 4` (tools) or `--quest 5` (materials) to
see the buttons, for screenshots and reviews. The schema is the one in
`crates/mk-inventory/src/library.rs` (version 1); a file with any other tables is refused.
"""
import re
import sqlite3
import sys
from pathlib import Path

LIBRARY_RS = Path(__file__).resolve().parent.parent / "crates/mk-inventory/src/library.rs"

# (kind id from the taxonomy, access)
TOOLS = [
    ("tape_measure", "owned"), ("square", "owned"), ("clamps", "owned"), ("hand_saw", "owned"),
    ("hammer", "owned"), ("screwdrivers", "owned"), ("drill_driver", "owned"),
    ("jigsaw", "owned"), ("circular_saw", "owned"), ("sander", "owned"), ("router", "shared"),
    ("drill_press", "wishlist"), ("track_saw", "makerspace"),
]

# (said, qty, kind, material, section_a, section_b, length, thickness, sheet_a, sheet_b)
MATERIALS = [
    ("15 pine 2x4s, 3 foot", 15, "board", "pine", 38.0, 89.0, 914.4, None, None, None),
    ("half sheet 3/4 ply", 1, "sheet", "plywood", None, None, None, 19.0, 1220.0, 610.0),
    ("2 oak 1x6s, 4 foot", 2, "board", "oak", 19.0, 140.0, 1219.2, None, None, None),
]


def main(path):
    schema = re.search(r'const SCHEMA_V1: &str = "(.*?)";', LIBRARY_RS.read_text(encoding="utf-8"), re.S)
    if not schema:
        raise SystemExit("could not find SCHEMA_V1 in library.rs")
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        target.unlink()
    db = sqlite3.connect(target)
    db.executescript(schema.group(1))
    db.execute("PRAGMA user_version = 1")
    for kind, access in TOOLS:
        db.execute("INSERT INTO tools (kind, label, access) VALUES (?, ?, ?)",
                   (kind, kind.replace("_", " ").capitalize(), access))
    for said, qty, kind, material, sa, sb, length, thick, ha, hb in MATERIALS:
        db.execute(
            "INSERT INTO materials (said, qty, kind, material, section_a_mm, section_b_mm, "
            "length_mm, thickness_mm, sheet_a_mm, sheet_b_mm) VALUES (?,?,?,?,?,?,?,?,?,?)",
            (said, qty, kind, material, sa, sb, length, thick, ha, hb))
    db.commit()
    db.close()
    print(f"{target}: {len(TOOLS)} tools, {len(MATERIALS)} material lines")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    main(sys.argv[1])
