"""Checks the generated test pass: item ids unique, the new ones present, no doubled escapes."""
import json
from pathlib import Path

PASS = (Path(__file__).resolve().parents[2] / "docs" / "test-passes"
        / "Test pass - MakerCAD Quest workshop Findings - 2026-11-22.html")
text = PASS.read_text(encoding="utf-8")
start = text.index("const ITEMS = ") + len("const ITEMS = ")
end = text.index("/* Final notes prompts")
items = json.loads(text[start:end].rstrip().rstrip(";"))
ids = [x["id"] for s in items for x in s["items"]]
print(len(items), "sections,", len(ids), "points")
print("duplicate ids:", sorted({i for i in ids if ids.count(i) > 1}))
print("new:", [i for i in ids if i.startswith(("w_days", "ws_when", "fin_", "y_"))])
doubled = [x["id"] for s in items for x in s["items"] if '\\"' in x["d"] + x["e"]]
print("text with a literal backslash-quote:", doubled)
