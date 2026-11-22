"""Lands a reviewed Crew icon set in the app.

    python tools/icons_land.py ICONS_MODULE.py --campaign mk-icons-1 --member Gordon

Checks the module with icon_lint (refuses on any problem), writes each icon to
apps/mk-desktop/assets/icons/<id>.svg with a <id>.svg.attribution sidecar (AGENTS.md rule 4),
then regenerates apps/mk-desktop/src/icon_data.rs from every SVG in that folder.
"""

import argparse
import ast
import datetime
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(__file__))

from icon_lint import lint_module_source  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON_DIR = os.path.join(REPO, "apps", "mk-desktop", "assets", "icons")
DATA = os.path.join(REPO, "apps", "mk-desktop", "src", "icon_data.rs")


def load(source):
    tree = ast.parse(source)
    for node in tree.body:
        value = getattr(node, "value", None)
        if isinstance(value, ast.Dict):
            return {k.value: v.value for k, v in zip(value.keys, value.values)}
    return {}


def regenerate():
    names = sorted(f for f in os.listdir(ICON_DIR) if f.endswith(".svg"))
    lines = [
        "//! The icon SVGs the app ships, by id. Written by `tools/icons_land.py` when a reviewed Crew",
        "//! icon set lands in `assets/icons/`; edit that folder and run the tool, not this file.",
        "",
        "/// (id, SVG document).",
        "pub const SVGS: &[(&str, &str)] = &[",
    ]
    for name in names:
        lines.append(f'    ("{name[:-4]}", include_str!("../assets/icons/{name}")),')
    lines.append("];")
    with open(DATA, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    # Long ids push a line past 100 characters, and the cloud's format gate refuses the file
    # unformatted (cloud run #5, 2026-11-21), so rustfmt it here.
    try:
        subprocess.run(["rustfmt", "--edition", "2021", DATA], check=True)
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"warning: rustfmt did not format {DATA}: {e}; run cargo fmt -p mk-desktop")
    return len(names)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("module")
    ap.add_argument("--campaign", required=True)
    ap.add_argument("--member", required=True)
    ap.add_argument("--only", nargs="*", help="land only these ids (the ones that passed review)")
    a = ap.parse_args()
    source = open(a.module, encoding="utf-8").read()
    problems = lint_module_source(source)
    if problems:
        print("\n".join(problems))
        print("refused: fix the problems first")
        return 1
    icons = load(source)
    if a.only:
        icons = {k: v for k, v in icons.items() if k in set(a.only)}
    os.makedirs(ICON_DIR, exist_ok=True)
    today = datetime.date.today().isoformat()
    for icon_id, svg in icons.items():
        path = os.path.join(ICON_DIR, f"{icon_id}.svg")
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(svg.strip() + "\n")
        with open(path + ".attribution", "w", encoding="utf-8", newline="\n") as f:
            f.write(
                f"author: {a.member} (the Crew), campaign {a.campaign}\n"
                f"made: {today}, from MakerCAD doc 18 (crew/contracts/mk_icons.md)\n"
                f"source: {os.path.basename(a.module)}, reviewed by Claude on a contact sheet\n"
                "licence: MIT OR Apache-2.0 (MakerCAD's own)\n")
    total = regenerate()
    print(f"landed {len(icons)} icons; the app now ships {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
