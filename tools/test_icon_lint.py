"""Tests for tools/icon_lint.py:  python -m pytest tools/test_icon_lint.py"""

import os
import sys

sys.path.insert(0, os.path.dirname(__file__))

from icon_lint import lint_module_source, lint_svg  # noqa: E402

HEAD = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">'
INK = 'stroke="#2B2A4C" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"'
GOOD = (HEAD + f'<rect x="8" y="22" width="48" height="20" rx="4" fill="#E8C9A8" {INK}/>'
        f'<path d="M 14 32 C 24 28 40 36 50 32" fill="none" stroke="#2B2A4C" stroke-width="3" '
        'stroke-linecap="round" stroke-linejoin="round"/></svg>')


def rules(svg):
    return sorted({p.split(":")[0] for p in lint_svg(svg)})


def test_a_good_board_passes():
    assert lint_svg(GOOD) == []


def test_a_group_passes_its_stroke_down():
    svg = (HEAD + f'<g fill="none" {INK}><line x1="10" y1="10" x2="50" y2="50"/>'
           '<circle cx="32" cy="32" r="10"/></g></svg>')
    assert lint_svg(svg) == []


def test_each_rule_is_caught():
    cases = {
        "rule 1": GOOD.replace('viewBox="0 0 64 64"', 'viewBox="0 0 64 64" width="64"'),
        "rule 2": GOOD.replace("</svg>", '<text x="10" y="10" fill="#2B2A4C">A</text></svg>'),
        "rule 3": HEAD + f'<circle cx="32" cy="32" r="10" fill="none" {INK}/></svg>',
        "rule 4": GOOD.replace('<rect ', '<rect id="b" '),
        "rule 5": GOOD.replace('fill="#E8C9A8"', 'fill="#123456"'),
        "rule 6": GOOD.replace('stroke-width="3"', 'stroke-width="1"'),
        "rule 7": GOOD.replace('stroke="#2B2A4C" stroke-width="3"', 'stroke="#3E7BFA" stroke-width="3"'),
        "rule 8": GOOD.replace('fill="#E8C9A8"', 'fill="#FF8A7A"').replace('fill="none"', 'fill="#FF8A7A"'),
        "rule 9": GOOD.replace('stroke="#2B2A4C"', 'stroke="#E8C9A8"'),
        "rule 10": GOOD.replace('<rect ', '<rect opacity="0.3" '),
        "rule 11": GOOD.replace("M 14 32 C", "m 14 32 C"),
        "rule 12": GOOD.replace('x="8"', 'x="8.25"'),
        "rule 13": GOOD.replace('width="48"', 'width="60"'),
    }
    for rule, svg in cases.items():
        assert rule in rules(svg), (rule, lint_svg(svg))


def test_a_missing_fill_would_be_black():
    svg = HEAD + f'<circle cx="32" cy="32" r="10" {INK}/><line x1="10" y1="10" x2="20" y2="20" {INK} fill="none"/></svg>'
    assert "rule 5" in rules(svg)


def test_the_module_shape_and_ids():
    ok = f"ICONS = {{'a.one': {GOOD!r}, 'a.two': {GOOD.replace('22', '24')!r}}}"
    assert lint_module_source(ok, ["a.one", "a.two"]) == []
    assert any("missing ['a.two']" in p for p in lint_module_source(ok, ["a.one", "a.two", "a.three"]) + lint_module_source(f"ICONS = {{'a.one': {GOOD!r}}}", ["a.one", "a.two"]))
    assert any(p.startswith("module") for p in lint_module_source("import os\n" + ok))
    same = f"ICONS = {{'a.one': {GOOD!r}, 'a.two': {GOOD!r}}}"
    assert any(p.startswith("rule 14") for p in lint_module_source(same))
