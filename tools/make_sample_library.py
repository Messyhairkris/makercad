"""Writes a sample workshop library (the SQLite file the Quest's "Use my workshop" reads).

    python tools/make_sample_library.py PATH

Then `set MAKERCAD_LIBRARY=PATH` and `makercad --quest 4` (tools) or `--quest 5` (materials) to
see the buttons, for screenshots and reviews. The schema is the one in
`crates/mk-inventory/src/library.rs` (version 2: SCHEMA_V1 then SCHEMA_V2, which adds when a tool\ncan be used); a file with any other tables is refused.
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
    ("drill_press", "wishlist"), ("track_saw", "makerspace"), ("soldering_iron", "shared"),
]

# When a tool can be used (D31): kind -> (days as a mask, bit 0 Monday ... bit 6 Sunday, note).
# A tool that is not here is there every day.
WHEN = {
    "soldering_iron": (0b1100000, "at Sam's"),
    "track_saw": (0b0001010, "makerspace open evenings"),
}

# (said, qty, kind, material, section_a, section_b, length, thickness, sheet_a, sheet_b)
MATERIALS = [
    ("15 pine 2x4s, 3 foot", 15, "board", "pine", 38.0, 89.0, 914.4, None, None, None),
    ("half sheet 3/4 ply", 1, "sheet", "plywood", None, None, None, 19.0, 1220.0, 610.0),
    ("2 oak 1x6s, 4 foot", 2, "board", "oak", 19.0, 140.0, 1219.2, None, None, None),
]


def main(path):
    source = LIBRARY_RS.read_text(encoding="utf-8")
    schemas = []
    for name in ("SCHEMA_V1", "SCHEMA_V2"):
        found = re.search(r"const " + name + r': &str = "(.*?)";', source, re.S)
        if not found:
            raise SystemExit(f"could not find {name} in library.rs")
        schemas.append(found.group(1))
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        target.unlink()
    db = sqlite3.connect(target)
    for schema in schemas:
        db.executescript(schema)
    db.execute("PRAGMA user_version = 2")
    for kind, access in TOOLS:
        days, note = WHEN.get(kind, (0b1111111, ""))
        db.execute("INSERT INTO tools (kind, label, access, when_days, when_note) VALUES (?, ?, ?, ?, ?)",
                   (kind, kind.replace("_", " ").capitalize(), access, days, note))
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
