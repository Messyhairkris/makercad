"""Checks MakerCAD icons against doc 18's SVG rules (docs/design/18_Design_Prompt.md).

    python tools/icon_lint.py icons_shell.py [icons_stock.py ...]

Each argument is an icon-set module (`ICONS: dict[str, str]`). Prints every problem and exits 1
if there are any. The same checks are the hidden tests of the Crew's icon campaigns, so the
rule numbers match crew/contracts/mk_icons.md (ATLAS-api-harness).
"""

import ast
import re
import sys
import xml.etree.ElementTree as ET

SVG_NS = "{http://www.w3.org/2000/svg}"
PALETTE = {
    "#2B2A4C": "ink",
    "#FFFDF7": "paper",
    "#CFE3FA": "tint",
    "#E8C9A8": "wood",
    "#FF8A7A": "accent",
    "#E5484D": "cut",
    "#3E7BFA": "fold",
    "#2BA471": "ok",
    "#F08C00": "warn",
}
INK, ACCENT, CUT, FOLD = "#2B2A4C", "#FF8A7A", "#E5484D", "#3E7BFA"
PRESENTATION = {"fill", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin",
                "stroke-dasharray", "opacity"}
GEOMETRY = {
    "path": {"d"},
    "circle": {"cx", "cy", "r"},
    "ellipse": {"cx", "cy", "rx", "ry"},
    "rect": {"x", "y", "width", "height", "rx", "ry"},
    "line": {"x1", "y1", "x2", "y2"},
    "polyline": {"points"},
    "polygon": {"points"},
    "g": set(),
}
INHERITED = {"fill", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin",
             "stroke-dasharray"}
NUMBER = re.compile(r"-?(?:\d+\.?\d*|\.\d+)")
SAFE_MIN, SAFE_MAX = 4.0, 60.0


def _nums(text):
    return [float(n) for n in NUMBER.findall(text or "")]


def _half_units(text, where, problems):
    for raw in NUMBER.findall(text or ""):
        value = float(raw)
        if "." in raw and len(raw.split(".")[1]) > 1 or abs(value * 2 - round(value * 2)) > 1e-9:
            problems.append(f"rule 12: {where}: {raw} is not a whole or half unit")


def _inside(x, y, where, problems):
    if not (SAFE_MIN <= x <= SAFE_MAX and SAFE_MIN <= y <= SAFE_MAX):
        problems.append(f"rule 13: {where}: point ({x:g}, {y:g}) is outside 4..60")


def _path_points(d, where, problems):
    """Every point and control point of path data, checking rule 11 on the way."""
    tokens = re.findall(r"[A-Za-z]|-?(?:\d+\.?\d*|\.\d+)", d or "")
    points, cmd, args = [], None, []
    sizes = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "Q": 4, "A": 7, "Z": 0}
    cx = cy = 0.0

    def flush():
        nonlocal cx, cy
        if cmd is None:
            return
        n = sizes[cmd]
        if n == 0:
            if args:
                problems.append(f"rule 11: {where}: Z takes no numbers")
            return
        if len(args) % n != 0 or not args:
            problems.append(f"rule 11: {where}: {cmd} has {len(args)} numbers")
            return
        for i in range(0, len(args), n):
            a = args[i:i + n]
            if cmd in "ML":
                cx, cy = a
                points.append((cx, cy))
            elif cmd == "H":
                cx = a[0]
                points.append((cx, cy))
            elif cmd == "V":
                cy = a[0]
                points.append((cx, cy))
            elif cmd == "C":
                points.extend([(a[0], a[1]), (a[2], a[3]), (a[4], a[5])])
                cx, cy = a[4], a[5]
            elif cmd == "Q":
                points.extend([(a[0], a[1]), (a[2], a[3])])
                cx, cy = a[2], a[3]
            elif cmd == "A":
                if a[0] > 28 or a[1] > 28:
                    problems.append(f"rule 13: {where}: arc radius over 28")
                cx, cy = a[5], a[6]
                points.append((cx, cy))

    for t in tokens:
        if t.isalpha():
            flush()
            if t not in sizes:
                problems.append(f"rule 11: {where}: command {t} is not one of M L H V C Q A Z")
                cmd = None
            else:
                cmd = t
            args = []
        else:
            if cmd is None:
                problems.append(f"rule 11: {where}: a number before any command")
                continue
            args.append(float(t))
    flush()
    return points


def lint_svg(svg, name="icon"):
    """The problems with one SVG document; empty when it follows every rule."""
    problems = []
    try:
        root = ET.fromstring(svg.strip())
    except ET.ParseError as e:
        return [f"rule 1: {name}: not well-formed XML ({e})"]
    if root.tag != SVG_NS + "svg":
        return [f"rule 1: {name}: the root is {root.tag}, not an SVG <svg>"]
    if root.attrib != {"viewBox": "0 0 64 64"}:
        problems.append(f"rule 1: {name}: the root's attributes must be exactly viewBox=\"0 0 64 64\" "
                        f"(and xmlns), got {sorted(root.attrib)}")
    children = list(root.iter())[1:]
    if not 2 <= len(children) <= 12:
        problems.append(f"rule 3: {name}: {len(children)} elements (2 to 12 allowed)")
    accents, ink_strokes = 0, 0

    def walk(el, inherited, depth):
        nonlocal accents, ink_strokes
        for child in el:
            tag = child.tag.replace(SVG_NS, "")
            where = f"{name} <{tag}>"
            if not child.tag.startswith(SVG_NS) or tag not in GEOMETRY:
                problems.append(f"rule 2: {name}: <{child.tag}> is not allowed")
                continue
            allowed = PRESENTATION | GEOMETRY[tag]
            for attr in child.attrib:
                if attr not in allowed:
                    problems.append(f"rule 4: {where}: attribute {attr} is not allowed")
            eff = dict(inherited)
            eff.update({k: v for k, v in child.attrib.items() if k in INHERITED})
            for attr in ("fill", "stroke"):
                v = child.attrib.get(attr)
                if v is None:
                    continue
                if attr == "fill" and v == "none":
                    continue
                if v not in PALETTE:
                    problems.append(f"rule 5: {where}: {attr}={v} is not a palette colour")
            if "opacity" in child.attrib and child.attrib["opacity"] != "0.5":
                problems.append(f"rule 10: {where}: opacity must be 0.5")
            if tag == "g":
                walk(child, eff, depth + 1)
                continue
            fill, stroke = eff.get("fill"), eff.get("stroke")
            if fill is None:
                problems.append(f"rule 5: {where}: no fill (SVG would paint it black); say fill=\"none\" or a colour")
            if stroke is not None:
                if eff.get("stroke-width") not in ("3", "4"):
                    problems.append(f"rule 6: {where}: stroke-width must be 3 or 4")
                if eff.get("stroke-linecap") != "round" or eff.get("stroke-linejoin") != "round":
                    problems.append(f"rule 6: {where}: stroke needs round linecap and linejoin")
                if stroke == INK:
                    ink_strokes += 1
            dashed = "stroke-dasharray" in eff
            if fill in (CUT, FOLD):
                problems.append(f"rule 7: {where}: cut and fold colours are strokes only")
            if stroke == CUT and dashed:
                problems.append(f"rule 7: {where}: a cut line is never dashed")
            if stroke == FOLD and not dashed:
                problems.append(f"rule 7: {where}: a fold line is always dashed")
            if dashed and stroke not in (INK, FOLD):
                problems.append(f"rule 7: {where}: only ink and fold lines may be dashed")
            if dashed:
                _half_units(eff.get("stroke-dasharray"), where, problems)
            if ACCENT in (fill, stroke):
                accents += 1
            a = child.attrib
            for attr in GEOMETRY[tag]:
                if attr in a and attr != "d" and attr != "points":
                    _half_units(a[attr], where, problems)
            try:
                if tag == "path":
                    _half_units(a.get("d"), where, problems)
                    for x, y in _path_points(a.get("d"), where, problems):
                        _inside(x, y, where, problems)
                elif tag in ("polyline", "polygon"):
                    _half_units(a.get("points"), where, problems)
                    pts = _nums(a.get("points"))
                    if len(pts) % 2 or not pts:
                        problems.append(f"rule 4: {where}: points must be x,y pairs")
                    for i in range(0, len(pts) - 1, 2):
                        _inside(pts[i], pts[i + 1], where, problems)
                elif tag == "line":
                    _inside(float(a["x1"]), float(a["y1"]), where, problems)
                    _inside(float(a["x2"]), float(a["y2"]), where, problems)
                elif tag == "rect":
                    x, y = float(a.get("x", "0")), float(a.get("y", "0"))
                    w, h = float(a["width"]), float(a["height"])
                    _inside(x, y, where, problems)
                    _inside(x + w, y + h, where, problems)
                elif tag == "circle":
                    cx, cy, r = float(a["cx"]), float(a["cy"]), float(a["r"])
                    _inside(cx - r, cy - r, where, problems)
                    _inside(cx + r, cy + r, where, problems)
                elif tag == "ellipse":
                    cx, cy = float(a["cx"]), float(a["cy"])
                    rx, ry = float(a["rx"]), float(a["ry"])
                    _inside(cx - rx, cy - ry, where, problems)
                    _inside(cx + rx, cy + ry, where, problems)
            except (KeyError, ValueError) as e:
                problems.append(f"rule 4: {where}: missing or unreadable geometry ({e})")

    walk(root, {}, 0)
    if accents > 1:
        problems.append(f"rule 8: {name}: {accents} accent elements (at most 1)")
    if ink_strokes == 0:
        problems.append(f"rule 9: {name}: no ink (#2B2A4C) outline")
    return problems


def lint_module_source(source, expected_ids=None):
    """Problems with an icon-set module's source: its shape, ids, and every SVG in it."""
    problems = []
    try:
        tree = ast.parse(source)
    except SyntaxError as e:
        return [f"module: not valid Python ({e})"]
    body = tree.body
    if len(body) != 1:
        problems.append(f"module: {len(body)} statements; exactly one (ICONS = {{...}}) is allowed")
    icons = {}
    for node in body:
        target = node.target if isinstance(node, ast.AnnAssign) else (
            node.targets[0] if isinstance(node, ast.Assign) and len(node.targets) == 1 else None)
        if not (isinstance(target, ast.Name) and target.id == "ICONS"):
            problems.append("module: the only statement must assign ICONS")
            continue
        value = node.value
        if not isinstance(value, ast.Dict):
            problems.append("module: ICONS must be a dict literal")
            continue
        for k, v in zip(value.keys, value.values):
            if not (isinstance(k, ast.Constant) and isinstance(k.value, str)
                    and isinstance(v, ast.Constant) and isinstance(v.value, str)):
                problems.append("module: every key and value must be a plain string literal")
                continue
            if k.value in icons:
                problems.append(f"module: {k.value} appears twice")
            icons[k.value] = v.value
    if expected_ids is not None and set(icons) != set(expected_ids):
        missing = sorted(set(expected_ids) - set(icons))
        extra = sorted(set(icons) - set(expected_ids))
        problems.append(f"module: ids differ from the set: missing {missing}, extra {extra}")
    seen = {}
    for name, svg in icons.items():
        if svg in seen:
            problems.append(f"rule 14: {name} is the same SVG as {seen[svg]}")
        seen.setdefault(svg, name)
        problems += lint_svg(svg, name)
    return problems


def main(paths):
    bad = 0
    for path in paths:
        with open(path, encoding="utf-8") as f:
            problems = lint_module_source(f.read())
        for p in problems:
            print(f"{path}: {p}")
        bad += len(problems)
        if not problems:
            print(f"{path}: ok")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
