"""Writes a contact sheet of icon sets: every icon at 16, 32 and 64 px, on paper and on the night
background, with its id and lint result. The person's half of doc 18 §18.7.

    python tools/icon_sheet.py icons_shell.py [icons_stock.py ...] --out sheet.html
"""

import argparse
import ast
import html
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))

from icon_lint import lint_svg  # noqa: E402

SIZES = (16, 32, 64)
# The Night Workshop swap, the same as apps/mk-desktop/src/icons.rs.
NIGHT = (("#2B2A4C", "#F2EEE6"), ("#FFFDF7", "#2A2838"), ("#CFE3FA", "#3E4A6B"))


def night(svg):
    for i, (light, _) in enumerate(NIGHT):
        svg = svg.replace(light, f"@{i}@")
    for i, (_, dark) in enumerate(NIGHT):
        svg = svg.replace(f"@{i}@", dark)
    return svg


def load(path):
    tree = ast.parse(open(path, encoding="utf-8").read())
    for node in tree.body:
        value = getattr(node, "value", None)
        if isinstance(value, ast.Dict):
            return {k.value: v.value for k, v in zip(value.keys, value.values)
                    if isinstance(k, ast.Constant) and isinstance(v, ast.Constant)}
    return {}


def sheet(sets):
    rows = []
    for title, icons in sets:
        rows.append(f"<h2>{html.escape(title)} <small>{len(icons)} icons</small></h2><div class=grid>")
        for name, svg in icons.items():
            problems = lint_svg(svg, name)
            status = "ok" if not problems else f"{len(problems)} problem(s)"
            imgs = "".join(
                f'<span class="sz s{s}">{svg.replace("<svg ", f"<svg width={s} height={s} ", 1)}</span>'
                for s in SIZES)
            night_svg = night(svg).replace("<svg ", "<svg width=32 height=32 ", 1)
            rows.append(
                f'<figure class="{"" if not problems else "bad"}"><div class=day>{imgs}</div>'
                f'<div class=night>{night_svg}</div><figcaption><b>{html.escape(name)}</b>'
                f'<span>{html.escape(status)}</span></figcaption>'
                + (f"<pre>{html.escape(chr(10).join(problems))}</pre>" if problems else "")
                + "</figure>")
        rows.append("</div>")
    return ("<!doctype html><meta charset=utf-8><title>MakerCAD icons</title><style>"
            "body{font-family:system-ui,sans-serif;background:#FFF8EC;color:#2B2A4C;margin:24px}"
            "h2 small{font-weight:400;color:#5B5775;font-size:14px}"
            ".grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(230px,1fr));gap:14px}"
            "figure{margin:0;background:#FFFDF7;border:2px solid #E4DCCB;border-radius:16px;padding:12px}"
            "figure.bad{border-color:#E5484D}"
            ".day{display:flex;gap:14px;align-items:end;padding:6px 4px}"
            ".night{background:#1F1D2B;border-radius:10px;padding:8px;margin-top:8px;display:inline-block}"
            "figcaption{display:flex;justify-content:space-between;margin-top:8px;font-size:13px}"
            "pre{white-space:pre-wrap;font-size:11px;color:#E5484D}"
            "</style><h1>MakerCAD icons: contact sheet</h1>"
            "<p>Each icon at 16, 32 and 64 px on paper, and at 32 px on the Night Workshop background "
            "with its colours swapped by role, as the app does. "
            "Doc 18 §18.7: an icon that does not read as its label at 16 px goes back for rework.</p>"
            + "".join(rows))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("modules", nargs="+")
    ap.add_argument("--out", default="icons.html")
    a = ap.parse_args()
    sets = [(os.path.basename(m), load(m)) for m in a.modules]
    with open(a.out, "w", encoding="utf-8") as f:
        f.write(sheet(sets))
    print("wrote", a.out, "with", sum(len(i) for _, i in sets), "icons")


if __name__ == "__main__":
    main()
