"""MakerCAD UI pass 2: the Design workspace in use, as editable SVG screens for FigCraft.

    python docs/design/ui-v2/make_screens.py [--render-dir DIR]

Writes 01-design-bench.svg … 06-sancho-stage.svg (1440 × 900) next to this file. The chrome
follows doc 11 §11.4 (project bar, environment tabs, toolbox, context panel, timeline, status
bar) in the Daydream look (apps/mk-desktop/src/theme.rs) with the app's real icons
(apps/mk-desktop/assets/icons). Screens 1 and 6 put MakerCAD's own render of the viewport
(bench, plate, Sancho) in the view, from --render-dir; the others draw the part as vectors.

Numbered coral circles (group "callouts") mark the questions in README.md; hide that group in
FigCraft to see the bare screen. Text is real text and shapes are real shapes, so every element
can be moved, recoloured or rewritten.
"""

import argparse
import ast
import base64
import html
import math
import os
import re
import xml.etree.ElementTree as ET

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
ICON_DIR = os.path.join(REPO, "apps", "mk-desktop", "assets", "icons")

W, H = 1440, 900
# Daydream (theme.rs) and doc 18's palette.
INK, PAPER, TINT, WOOD = "#2B2A4C", "#FFFDF7", "#CFE3FA", "#E8C9A8"
ACCENT, OK, WARN, FOLD = "#FF8A7A", "#2BA471", "#F08C00", "#3E7BFA"
BG_TOP, BG_BOTTOM, BG2, PANEL = "#FFF8EC", "#E6F2FF", "#FFF8EC", "#FFFDF7"
MUTED, LINE, CHIP = "#5B5775", "#E4DCCB", "#E6F2FF"
CHIP_WAIT, CHIP_DONE = "#FAD3B4", "#C3E6C4"
FONT = "Nunito, 'Segoe UI', system-ui, sans-serif"
HAND = "'Patrick Hand', 'Segoe Print', 'Comic Sans MS', cursive"

# Layout (doc 11 §11.4).
BAR_H, TABS_H, STATUS_H, TIMELINE_H = 56, 46, 30, 96
TOOL_W, PANEL_W = 104, 340
VIEW_X, VIEW_Y = TOOL_W, BAR_H + TABS_H
VIEW_W = W - TOOL_W - PANEL_W
VIEW_H = H - VIEW_Y - TIMELINE_H - STATUS_H

SVG_NS = "{http://www.w3.org/2000/svg}"
# Icon children serialise as plain <path …> in the SVG namespace, not as ns0:path.
ET.register_namespace("", "http://www.w3.org/2000/svg")


def esc(s):
    return html.escape(str(s), quote=True)


# ---------------------------------------------------------------- icons


def _load_icons():
    icons = {}
    for name in os.listdir(ICON_DIR):
        if name.endswith(".svg"):
            icons[name[:-4]] = open(os.path.join(ICON_DIR, name), encoding="utf-8").read()
    return icons


ICONS = _load_icons()


def add_drafts(paths):
    """Icons still in rework (mk-icons-4): the Crew's first drafts, so the screens are complete."""
    for p in paths:
        tree = ast.parse(open(p, encoding="utf-8").read())
        d = tree.body[0].value
        for k, v in zip(d.keys, d.values):
            ICONS.setdefault(k.value, v.value)


def icon(icon_id, x, y, size, title=None):
    svg = ICONS.get(icon_id)
    if svg is None:
        return f'<rect x="{x}" y="{y}" width="{size}" height="{size}" rx="6" fill="{CHIP}" stroke="{LINE}"/>'
    root = ET.fromstring(svg.strip())
    inner = "".join(ET.tostring(c, encoding="unicode") for c in root)
    label = f' id="icon-{esc(icon_id)}"'
    return f'<svg{label} x="{x}" y="{y}" width="{size}" height="{size}" viewBox="0 0 64 64">{inner}</svg>'


# ---------------------------------------------------------------- primitives


def rect(x, y, w, h, fill, stroke=None, rx=0, sw=1, extra=""):
    s = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ""
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}"{s}{extra}/>'


def text(x, y, s, size=14, weight=400, fill=INK, anchor="start", family=FONT, extra=""):
    return (f'<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" font-weight="{weight}" '
            f'fill="{fill}" text-anchor="{anchor}"{extra}>{esc(s)}</text>')


def button(x, y, w, h, label, primary=False, icon_id=None, size=14):
    fill, stroke, colour = (ACCENT, ACCENT, INK) if primary else (PANEL, LINE, INK)
    out = [rect(x, y, w, h, fill, stroke, rx=h / 2, sw=2)]
    tx = x + w / 2
    if icon_id:
        out.append(icon(icon_id, x + 10, y + (h - 20) / 2, 20))
        tx += 10
    out.append(text(tx, y + h / 2 + size * 0.36, label, size, 700, colour, "middle"))
    return "".join(out)


def chip(x, y, label, fill=CHIP, size=13, w=None, colour=INK, weight=700):
    w = w or (len(label) * size * 0.56 + 22)
    return rect(x, y, w, 26, fill, rx=13) + text(x + w / 2, y + 17.5, label, size, weight, colour, "middle"), w


def field(x, y, w, label, value, unit="", focused=False, h=34):
    out = [text(x, y - 7, label, 12, 700, MUTED)]
    out.append(rect(x, y, w, h, PAPER, ACCENT if focused else LINE, rx=8, sw=2))
    out.append(text(x + 12, y + h / 2 + 5, value, 15, 700, INK))
    if unit:
        out.append(text(x + w - 12, y + h / 2 + 5, unit, 13, 400, MUTED, "end"))
    return "".join(out)


def segmented(x, y, options, chosen, w_each=78, h=30):
    out = [rect(x, y, w_each * len(options), h, CHIP, LINE, rx=h / 2, sw=1.5)]
    for i, o in enumerate(options):
        if i == chosen:
            out.append(rect(x + i * w_each + 2, y + 2, w_each - 4, h - 4, PAPER, INK, rx=(h - 4) / 2, sw=1.5))
        out.append(text(x + i * w_each + w_each / 2, y + h / 2 + 5, o, 13, 700 if i == chosen else 400, INK, "middle"))
    return "".join(out)


def toggle(x, y, on, label):
    knob = x + 26 if on else x + 4
    return (rect(x, y, 44, 24, OK if on else LINE, rx=12) + f'<circle cx="{knob + 8}" cy="{y + 12}" r="9" fill="{PAPER}"/>'
            + text(x + 56, y + 17, label, 14, 400, INK))


def radio(x, y, on, label, sub=None):
    out = [f'<circle cx="{x + 9}" cy="{y + 9}" r="8" fill="{PAPER}" stroke="{INK}" stroke-width="2"/>']
    if on:
        out.append(f'<circle cx="{x + 9}" cy="{y + 9}" r="4" fill="{INK}"/>')
    out.append(text(x + 26, y + 14, label, 14, 700 if on else 400))
    if sub:
        out.append(text(x + 26, y + 32, sub, 12, 400, MUTED))
    return "".join(out)


def checkbox(x, y, on, label):
    out = [rect(x, y, 18, 18, PAPER if not on else INK, INK, rx=4, sw=2)]
    if on:
        out.append(f'<path d="M{x + 4} {y + 9} L{x + 8} {y + 13} L{x + 14} {y + 5}" fill="none" stroke="{PAPER}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>')
    out.append(text(x + 28, y + 14, label, 14))
    return "".join(out)


def callout(n, x, y):
    return (f'<g><circle cx="{x}" cy="{y}" r="13" fill="{ACCENT}" stroke="{INK}" stroke-width="2"/>'
            + text(x, y + 5, n, 14, 800, INK, "middle") + "</g>")


def bubble(x, y, w, lines, tail="left"):
    """Sancho's speech bubble (handwritten accent font, doc 11 §11.6)."""
    h = 18 + 22 * len(lines)
    out = [rect(x, y, w, h, PAPER, INK, rx=14, sw=2)]
    if tail == "left":
        out.append(f'<path d="M{x + 2} {y + h - 22} L{x - 14} {y + h - 6} L{x + 18} {y + h - 2}" fill="{PAPER}" stroke="{INK}" stroke-width="2" stroke-linejoin="round"/>')
        out.append(rect(x + 3, y + h - 24, 18, 20, PAPER))
    for i, l in enumerate(lines):
        out.append(text(x + 16, y + 30 + 22 * i, l, 17, 400, INK, family=HAND))
    return "".join(out)


# ---------------------------------------------------------------- chrome


def project_bar(intent="“A cable holder plate for under my desk, two screws, a hollow boss…”"):
    o = [rect(0, 0, W, BAR_H, PANEL), f'<line x1="0" y1="{BAR_H}" x2="{W}" y2="{BAR_H}" stroke="{LINE}" stroke-width="2"/>']
    o.append(icon("app.workshop", 14, 12, 32))
    o.append(text(56, 35, "Cable holder ▾", 18, 800))
    o.append(rect(196, 12, 430, 32, BG2, LINE, rx=16, sw=2))
    o.append(text(214, 33, intent, 13, 400, MUTED))
    c, w = chip(650, 15, "◷ due Sat · 2 days slack", CHIP_DONE)
    o.append(c)
    # Command search box (doc 11: Ctrl+K, plain language).
    o.append(rect(W - 470, 11, 300, 34, PAPER, LINE, rx=17, sw=2))
    o.append(icon("app.search", W - 460, 17, 22))
    o.append(text(W - 430, 33, "Search or say what to do…", 14, 400, MUTED))
    o.append(rect(W - 222, 17, 44, 22, CHIP, rx=6) + text(W - 200, 33, "Ctrl K", 11, 700, MUTED, "middle"))
    o.append(icon("app.undo", W - 156, 14, 28) + icon("app.redo", W - 122, 14, 28))
    o.append(icon("app.sancho", W - 84, 10, 36) + icon("app.settings", W - 44, 14, 28))
    return "".join(o)


ENVS = [("env.design", "Design"), ("env.stock", "Stock & Cut"), ("env.plan", "Plan"),
        ("env.build", "Build"), ("env.space", "Space"), ("env.publish", "Share")]


def env_tabs(active="Design"):
    y = BAR_H
    o = [rect(0, y, W, TABS_H, BG2), f'<line x1="0" y1="{y + TABS_H}" x2="{W}" y2="{y + TABS_H}" stroke="{LINE}" stroke-width="2"/>']
    x = 16
    for icon_id, label in ENVS:
        w = len(label) * 8.2 + 54
        on = label == active
        if on:
            o.append(rect(x, y + 6, w, TABS_H - 6, PANEL, LINE, rx=10, sw=2))
            o.append(rect(x + 12, y + TABS_H - 4, w - 24, 4, ACCENT, rx=2))
        o.append(icon(icon_id, x + 10, y + 13, 22))
        o.append(text(x + 38, y + 30, label, 15, 800 if on else 600, INK if on else MUTED))
        x += w + 6
    o.append(text(x + 8, y + 30, "+ environment", 14, 600, MUTED))
    return "".join(o)


TOOLS = {
    "View": [("view.orbit", "Turn"), ("view.fit", "Fit")],
    "Sketch": [("sketch.new", "Sketch"), ("sketch.rect", "Rectangle"), ("sketch.circle", "Circle"), ("sketch.dimension", "Size")],
    "Make": [("make.extrude", "Pull up"), ("make.hole", "Drill"), ("make.round", "Round over"), ("make.cut", "Cut away")],
}


def toolbox(active=None, groups=TOOLS, title="Toolbox"):
    o = [rect(0, VIEW_Y, TOOL_W, VIEW_H, PANEL), f'<line x1="{TOOL_W}" y1="{VIEW_Y}" x2="{TOOL_W}" y2="{VIEW_Y + VIEW_H}" stroke="{LINE}" stroke-width="2"/>']
    y = VIEW_Y + 14
    o.append(text(TOOL_W / 2, y + 4, title, 11, 800, MUTED, "middle"))
    y += 14
    for group, tools in groups.items():
        o.append(text(12, y + 14, group.upper(), 10, 800, MUTED))
        y += 20
        for icon_id, label in tools:
            on = icon_id == active
            if on:
                o.append(rect(8, y - 2, TOOL_W - 16, 56, CHIP_WAIT, INK, rx=10, sw=2))
            o.append(icon(icon_id, TOOL_W / 2 - 16, y + 2, 32))
            o.append(text(TOOL_W / 2, y + 48, label, 11, 700 if on else 600, INK, "middle"))
            y += 58
        y += 4
    return "".join(o)


STEPS = [("sketch.new", "Outline", "sketch"), ("make.extrude", "Pull up 12 mm", ""), ("make.round", "Round over 8 mm ×4", ""),
         ("make.hole", "Drill 2 × Ø10", ""), ("make.extrude", "Boss Ø36 × 20", ""), ("make.hole", "Bore Ø18", "")]


def timeline(steps=STEPS, selected=None, rolled_back_after=None, maker=True):
    y = H - STATUS_H - TIMELINE_H
    o = [rect(0, y, W, TIMELINE_H, BG2), f'<line x1="0" y1="{y}" x2="{W}" y2="{y}" stroke="{LINE}" stroke-width="2"/>']
    o.append(text(18, y + 26, "How it's made", 14, 800))
    o.append(segmented(150, y + 9, ["Maker words", "CAD words"], 0 if maker else 1, w_each=104, h=26))
    o.append(text(W - 18, y + 26, "Drag a step to reorder · double-click to change it", 12, 400, MUTED, "end"))
    x = 18
    for i, (icon_id, label, kind) in enumerate(steps):
        w = len(label) * 7.6 + 62
        on = i == selected
        dim = rolled_back_after is not None and i > rolled_back_after
        fill = CHIP_WAIT if on else (PANEL if not dim else BG2)
        o.append(rect(x, y + 44, w, 40, fill, INK if on else LINE, rx=12, sw=2, extra=' opacity="0.5"' if dim else ""))
        o.append(f'<g{" opacity=" + chr(34) + "0.5" + chr(34) if dim else ""}>' + icon(icon_id, x + 8, y + 50, 28)
                 + text(x + 42, y + 69, label, 13, 700 if on else 600) + "</g>")
        if i < len(steps) - 1:
            o.append(f'<path d="M{x + w + 3} {y + 64} L{x + w + 11} {y + 64}" stroke="{MUTED}" stroke-width="2" stroke-linecap="round"/>')
        x += w + 14
        if rolled_back_after is not None and i == rolled_back_after:
            o.append(rect(x - 9, y + 38, 4, 52, ACCENT, rx=2))
    return "".join(o)


def status_bar(extra=""):
    y = H - STATUS_H
    o = [rect(0, y, W, STATUS_H, PANEL), f'<line x1="0" y1="{y}" x2="{W}" y2="{y}" stroke="{LINE}" stroke-width="2"/>']
    o.append(text(16, y + 20, "Pine 1×6 board", 12, 700))
    o.append(rect(118, y + 10, 120, 10, CHIP, rx=5) + rect(118, y + 10, 38, 10, OK, rx=5))
    o.append(text(246, y + 20, "uses 32% of one board · offcut keeps", 12, 400, MUTED))
    o.append(text(520, y + 20, "Fits the drawer: ✓", 12, 700, OK))
    o.append(text(660, y + 20, "$3.10 of $10", 12, 400, MUTED))
    o.append(text(780, y + 20, extra, 12, 400, MUTED))
    o.append(text(W - 16, y + 20, "mm · in   |   Sancho 1.68 m   |   dx12", 12, 400, MUTED, "end"))
    return "".join(o)


def context_panel(title, body, subtitle=None):
    x = W - PANEL_W
    o = [rect(x, VIEW_Y, PANEL_W, VIEW_H, PANEL), f'<line x1="{x}" y1="{VIEW_Y}" x2="{x}" y2="{VIEW_Y + VIEW_H}" stroke="{LINE}" stroke-width="2"/>']
    o.append(text(x + 20, VIEW_Y + 32, title, 18, 800))
    if subtitle:
        o.append(text(x + 20, VIEW_Y + 54, subtitle, 13, 400, MUTED))
    o.append(body)
    return "".join(o)


def viewport_background():
    return (f'<defs><linearGradient id="viewbg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{BG_TOP}"/>'
            f'<stop offset="1" stop-color="{BG_BOTTOM}"/></linearGradient></defs>'
            + rect(VIEW_X, VIEW_Y, VIEW_W, VIEW_H, "url(#viewbg)"))


def view_image(path):
    data = base64.b64encode(open(path, "rb").read()).decode("ascii")
    return (f'<image x="{VIEW_X}" y="{VIEW_Y}" width="{VIEW_W}" height="{VIEW_H}" preserveAspectRatio="xMidYMid slice" '
            f'href="data:image/png;base64,{data}"/>')


def view_hud(lines):
    """The view's small corner readout."""
    out = [rect(VIEW_X + 14, VIEW_Y + 14, 250, 22 + 18 * len(lines), PAPER, LINE, rx=10, sw=1.5, extra=' opacity="0.92"')]
    for i, l in enumerate(lines):
        out.append(text(VIEW_X + 28, VIEW_Y + 36 + 18 * i, l, 12, 600 if i == 0 else 400, INK if i == 0 else MUTED))
    return "".join(out)


def dim_v(x, y0, y1, label, left=False):
    """A vertical dimension line with end ticks and its label beside it."""
    lx = x - 12 if left else x + 12
    anchor = "end" if left else "start"
    return (f'<g><line x1="{x}" y1="{y0}" x2="{x}" y2="{y1}" stroke="{INK}" stroke-width="2"/>'
            f'<line x1="{x - 7}" y1="{y0}" x2="{x + 7}" y2="{y0}" stroke="{INK}" stroke-width="2"/>'
            f'<line x1="{x - 7}" y1="{y1}" x2="{x + 7}" y2="{y1}" stroke="{INK}" stroke-width="2"/>'
            + rect(lx - (70 if left else 0), (y0 + y1) / 2 - 14, 70, 26, PAPER, INK, rx=8, sw=1.5)
            + text(lx - 35 if left else lx + 35, (y0 + y1) / 2 + 5, label, 13, 800, INK, "middle") + "</g>")


def page(name, title, body, callouts=""):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">'
            f"<title>{esc(title)}</title>"
            f'<rect width="{W}" height="{H}" fill="{BG2}"/>'
            f'<g id="screen">{body}</g><g id="callouts">{callouts}</g></svg>')


# ---------------------------------------------------------------- the part, drawn as vectors


class Iso:
    """An isometric drawing of the sample plate (mm, Z up) into the view."""

    def __init__(self, cx, cy, scale):
        self.cx, self.cy, self.s = cx, cy, scale
        self.c, self.k = math.cos(math.radians(30)), math.sin(math.radians(30))

    def p(self, x, y, z):
        return (self.cx + (x - y) * self.c * self.s, self.cy + (x + y) * self.k * self.s - z * self.s)

    def poly(self, pts, fill, stroke=INK, sw=1.5, extra=""):
        d = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts) + " Z"
        return f'<path d="{d}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round"{extra}/>'

    def ellipse(self, x, y, z, r, fill, stroke=INK, sw=1.5, extra=""):
        cx, cy = self.p(x, y, z)
        return (f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="{r * math.sqrt(2) * self.c * self.s:.1f}" '
                f'ry="{r * math.sqrt(2) * self.k * self.s:.1f}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"{extra}/>')


def rounded_outline(w, d, r, n=8):
    pts = []
    for cx, cy, a0 in [(w / 2 - r, d / 2 - r, 0), (-w / 2 + r, d / 2 - r, 90), (-w / 2 + r, -d / 2 + r, 180), (w / 2 - r, -d / 2 + r, 270)]:
        for i in range(n + 1):
            a = math.radians(a0 + 90 * i / n)
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def hull(points):
    pts = sorted(set((round(x, 2), round(y, 2)) for x, y in points))
    if len(pts) < 3:
        return pts

    def cross(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    lower, upper = [], []
    for p in pts:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], p) <= 0:
            lower.pop()
        lower.append(p)
    for p in reversed(pts):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], p) <= 0:
            upper.pop()
        upper.append(p)
    return lower[:-1] + upper[:-1]


TOP, SIDE, SIDE2 = "#B9C0CF", "#8E95A6", "#7D8496"


def plate(iso, radius=8.0, round_highlight=False, extra_top=""):
    """The sample plate: 120 × 80 × 12, rounded corners, two Ø10 holes, a Ø36 × 20 boss bored Ø18."""
    o = []
    out = rounded_outline(120, 80, radius)
    bottom = [iso.p(x, y, 0) for x, y in out]
    top = [iso.p(x, y, 12) for x, y in out]
    o.append(iso.poly(hull(bottom + top), SIDE))
    # The front-right sides a little darker, for depth.
    o.append(iso.poly(top, TOP))
    if round_highlight:
        for cx, cy, a0 in [(60 - radius, 40 - radius, 0), (-60 + radius, 40 - radius, 90), (60 - radius, -40 + radius, 270)]:
            arc_top = [iso.p(cx + radius * math.cos(math.radians(a0 + 90 * i / 8)), cy + radius * math.sin(math.radians(a0 + 90 * i / 8)), 12) for i in range(9)]
            arc_bot = [iso.p(cx + radius * math.cos(math.radians(a0 + 90 * i / 8)), cy + radius * math.sin(math.radians(a0 + 90 * i / 8)), 0) for i in range(9)]
            o.append(iso.poly(arc_top + list(reversed(arc_bot)), ACCENT, INK, 2, ' opacity="0.85"'))
    for hx in (-40, 40):
        o.append(iso.ellipse(hx, 0 if hx < 0 else 0, 12, 5, "#4F5566"))
    o.append(extra_top)
    # The boss.
    r, z0, z1 = 18, 12, 32
    lx, ly = iso.p(0, 0, z1)
    a = r * math.sqrt(2) * iso.c * iso.s
    b0 = iso.p(0, 0, z0)
    o.append(f'<path d="M{lx - a:.1f} {ly:.1f} L{b0[0] - a:.1f} {b0[1]:.1f} A{a:.1f} {r * math.sqrt(2) * iso.k * iso.s:.1f} 0 0 0 {b0[0] + a:.1f} {b0[1]:.1f} L{lx + a:.1f} {ly:.1f} Z" fill="{SIDE}" stroke="{INK}" stroke-width="1.5"/>')
    o.append(iso.ellipse(0, 0, z1, r, TOP))
    o.append(iso.ellipse(0, 0, z1, 9, "#4F5566"))
    return "".join(o)


# ---------------------------------------------------------------- screens


def screen_1(render):
    body = [viewport_background(), view_image(render) if render else ""]
    body.append(view_hud(["Bench · scale 1:1", "Part 120 × 80 × 32 mm · bench top 900 mm"]))
    body.append(bubble(VIEW_X + VIEW_W - 330, VIEW_Y + 40, 300, ["About the size of my hand.", "The boss fits a 16 mm cable."]))
    body.append(project_bar() + env_tabs() + toolbox())
    panel = []
    x = W - PANEL_W + 20
    y = VIEW_Y + 76
    panel.append(text(x, y, "Plate", 16, 800) + text(x, y + 20, "one part · pine 1×6 (19 mm)", 13, 400, MUTED))
    y += 44
    for k, v in [("Size", "120 × 80 × 32 mm"), ("", "4 ¾ × 3 ⅛ × 1 ¼ in"), ("Volume", "142 cm³"), ("Weight", "about 70 g in pine")]:
        panel.append(text(x, y, k, 13, 700, MUTED) + text(x + 90, y, v, 14, 600))
        y += 24
    y += 14
    panel.append(f'<line x1="{x}" y1="{y}" x2="{W - 20}" y2="{y}" stroke="{LINE}" stroke-width="2"/>')
    y += 28
    panel.append(text(x, y, "Material", 15, 800))
    y += 16
    c, _ = chip(x, y, "Pine 1×6 · from your rack", WOOD, w=210)
    panel.append(c + text(x + 220, y + 18, "change", 13, 700, FOLD))
    y += 50
    panel.append(text(x, y, "Size against Sancho", 15, 800))
    y += 24
    for line in ["Hand 185 mm · the plate is ⅔ of it", "Finger 18 mm wide · the bore takes two"]:
        panel.append(text(x, y, line, 13, 400, INK))
        y += 20
    y += 16
    panel.append(text(x, y, "Findings", 15, 800))
    y += 14
    panel.append(rect(x, y, PANEL_W - 40, 58, CHIP_DONE, rx=10) + text(x + 12, y + 24, "✓ Both screw holes clear a #8 screw", 13, 700)
                 + text(x + 12, y + 44, "Your drill set has a 5 mm bit", 12, 400, MUTED))
    y += 70
    panel.append(rect(x, y, PANEL_W - 40, 58, CHIP_WAIT, rx=10) + text(x + 12, y + 24, "! The boss wall is 9 mm thick", 13, 700)
                 + text(x + 12, y + 44, "Fine in pine; thinner may split", 12, 400, MUTED))
    body.append(context_panel("Selected", "".join(panel), "click a part or face to see it here"))
    body.append(timeline() + status_bar())
    calls = callout(1, 182, 46) + callout(2, 640, 28) + callout(3, TOOL_W, VIEW_Y + 62) + callout(4, VIEW_X + 520, VIEW_Y + 200) + callout(5, W - PANEL_W, VIEW_Y + 330) + callout(6, 420, H - STATUS_H - 70) + callout(7, 262, H - STATUS_H)
    return page("01", "1 · Design: the part on the bench", "".join(body), calls)


def screen_2():
    body = [viewport_background()]
    iso = Iso(VIEW_X + VIEW_W / 2 - 20, VIEW_Y + VIEW_H / 2 - 40, 4.2)
    # The sketch plane: the top face, with its grid.
    grid = []
    for g in range(-50, 51, 10):
        a, b = iso.p(g, -35, 12.05), iso.p(g, 35, 12.05)
        grid.append(f'<line x1="{a[0]:.1f}" y1="{a[1]:.1f}" x2="{b[0]:.1f}" y2="{b[1]:.1f}" stroke="{FOLD}" stroke-width="0.8" opacity="0.35"/>')
    for g in range(-30, 31, 10):
        a, b = iso.p(-55, g, 12.05), iso.p(55, g, 12.05)
        grid.append(f'<line x1="{a[0]:.1f}" y1="{a[1]:.1f}" x2="{b[0]:.1f}" y2="{b[1]:.1f}" stroke="{FOLD}" stroke-width="0.8" opacity="0.35"/>')
    # A slot being drawn: 40 × 12 mm, along the plate's front edge, 6 mm in.
    corners = [iso.p(-15, 24, 12.1), iso.p(25, 24, 12.1), iso.p(25, 34, 12.1), iso.p(-15, 34, 12.1)]
    sketch = [iso.poly(corners, "#FFE3DE", ACCENT, 3, ' opacity="0.95"')]
    for cx, cy in corners:
        sketch.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="5" fill="{PAPER}" stroke="{INK}" stroke-width="2"/>')
    a, b = iso.p(-15, 46, 12.1), iso.p(25, 46, 12.1)
    sketch.append(f'<line x1="{a[0]:.1f}" y1="{a[1]:.1f}" x2="{b[0]:.1f}" y2="{b[1]:.1f}" stroke="{INK}" stroke-width="2"/>')
    mx, my = (a[0] + b[0]) / 2, (a[1] + b[1]) / 2
    sketch.append(rect(mx - 46, my + 6, 92, 32, PAPER, ACCENT, rx=8, sw=2.5) + text(mx - 34, my + 28, "40", 16, 800) + text(mx + 38, my + 28, "mm", 12, 400, MUTED, "end"))
    sketch.append(f'<line x1="{mx - 14:.1f}" y1="{my + 12:.1f}" x2="{mx - 14:.1f}" y2="{my + 32:.1f}" stroke="{INK}" stroke-width="1.5"/>')
    c2 = iso.p(-17, 29, 12.1)
    sketch.append(rect(c2[0] - 60, c2[1] - 12, 52, 24, PAPER, LINE, rx=6, sw=1.5) + text(c2[0] - 34, c2[1] + 5, "12 mm", 12, 700, INK, "middle"))
    # Cursor.
    cx, cy = corners[2]
    sketch.append(f'<path d="M{cx + 10} {cy + 6} l0 22 l6 -6 l5 11 l4 -2 l-5 -11 l8 0 Z" fill="{PAPER}" stroke="{INK}" stroke-width="1.5"/>')
    body.append(plate(iso, extra_top="".join(grid)) + "".join(sketch))
    body.append(view_hud(["Sketching on: Top face of Plate", "Snap 1 mm · Shift: no snap · Esc: stop"]))
    sk = {"Draw": [("sketch.line", "Line"), ("sketch.rect", "Rectangle"), ("sketch.circle", "Circle"), ("sketch.arc", "Arc")],
          "Shape": [("sketch.dimension", "Size"), ("sketch.lock", "Lock"), ("sketch.trim", "Trim"), ("sketch.mirror", "Mirror")]}
    body.append(project_bar() + env_tabs() + toolbox("sketch.rect", sk, "Sketch"))
    x = W - PANEL_W + 20
    y = VIEW_Y + 80
    p = [text(x, y, "Rectangle", 16, 800) + text(x, y + 20, "click two corners, or type the sizes", 13, 400, MUTED)]
    y += 58
    p.append(field(x, y, 140, "Width", "40", "mm", True) + field(x + 160, y, 140, "Depth", "12", "mm"))
    y += 66
    p.append(text(x, y, "From", 12, 700, MUTED))
    p.append(segmented(x, y + 8, ["Corner", "Centre"], 0, w_each=100))
    y += 62
    p.append(text(x, y, "How fixed is it?", 15, 800))
    y += 12
    p.append(rect(x, y, PANEL_W - 40, 74, CHIP_WAIT, rx=10) + text(x + 12, y + 24, "2 sizes to go", 14, 800)
             + text(x + 12, y + 44, "Its distance from the front edge and", 12, 400, MUTED) + text(x + 12, y + 61, "from the left side can still change", 12, 400, MUTED))
    y += 90
    p.append(text(x, y, "Rules it follows", 15, 800))
    y += 24
    for r in ["▭ sides square to the plate", "═ centred left to right"]:
        p.append(text(x, y, r, 13))
        y += 22
    y += 20
    p.append(button(x, y, 140, 40, "Finish", True, "sketch.finish") + button(x + 156, y, 120, 40, "Cancel"))
    body.append(context_panel("Sketch", "".join(p), "on the Plate's top face"))
    body.append(timeline(STEPS + [("sketch.new", "Sketch: slot", "")], selected=6) + status_bar("drawing: rectangle"))
    calls = callout(1, TOOL_W, VIEW_Y + 170) + callout(2, VIEW_X + 680, VIEW_Y + 470) + callout(3, VIEW_X + 290, VIEW_Y + 26) + callout(4, W - PANEL_W, VIEW_Y + 318) + callout(5, W - PANEL_W, VIEW_Y + 150)
    return page("02", "2 · Sketching on a face", "".join(body), calls)


def screen_3():
    body = [viewport_background()]
    iso = Iso(VIEW_X + VIEW_W / 2 - 20, VIEW_Y + VIEW_H / 2 + 10, 4.2)
    body.append(plate(iso))
    # The slot pulled up 15 mm as a tab: the preview in the accent, see-through.
    h = 15
    base = [(-15, 24), (25, 24), (25, 34), (-15, 34)]
    top = [iso.p(x, y, 12 + h) for x, y in base]
    bot = [iso.p(x, y, 12) for x, y in base]
    body.append(iso.poly(hull(top + bot), ACCENT, INK, 2, ' opacity="0.55"'))
    body.append(iso.poly(top, "#FFB8AD", INK, 2, ' opacity="0.8"'))
    # The drag handle and its distance.
    mid_top = iso.p(5, 29, 12 + h)
    body.append(f'<line x1="{mid_top[0]:.1f}" y1="{mid_top[1]:.1f}" x2="{mid_top[0]:.1f}" y2="{mid_top[1] - 70:.1f}" stroke="{INK}" stroke-width="3"/>')
    body.append(f'<path d="M{mid_top[0] - 12:.1f} {mid_top[1] - 62:.1f} L{mid_top[0]:.1f} {mid_top[1] - 84:.1f} L{mid_top[0] + 12:.1f} {mid_top[1] - 62:.1f} Z" fill="{ACCENT}" stroke="{INK}" stroke-width="2"/>')
    body.append(rect(mid_top[0] + 22, mid_top[1] - 92, 104, 34, PAPER, ACCENT, rx=8, sw=2.5) + text(mid_top[0] + 36, mid_top[1] - 69, "15", 17, 800) + text(mid_top[0] + 116, mid_top[1] - 69, "mm", 12, 400, MUTED, "end"))
    body.append(view_hud(["Pull up: Sketch: slot", "Drag the arrow, or type a distance"]))
    body.append(bubble(VIEW_X + VIEW_W - 330, VIEW_Y + VIEW_H - 150, 300, ["15 mm is a little less", "than my thumb is wide."]))
    body.append(project_bar() + env_tabs() + toolbox("make.extrude"))
    x = W - PANEL_W + 20
    y = VIEW_Y + 80
    p = [text(x, y, "Pull up", 16, 800) + text(x, y + 20, "(extrude) the slot sketch", 13, 400, MUTED)]
    y += 58
    p.append(field(x, y, 180, "How far", "15", "mm", True))
    p.append(text(x + 196, y + 22, "= ⅝ in", 13, 400, MUTED))
    y += 66
    p.append(text(x, y, "Which way", 12, 700, MUTED) + segmented(x, y + 8, ["Up", "Down", "Both"], 0, w_each=96))
    y += 62
    p.append(text(x, y, "What it does", 12, 700, MUTED) + segmented(x, y + 8, ["Add", "Cut", "New part"], 0, w_each=96))
    y += 62
    p.append(text(x, y, "Material", 12, 700, MUTED))
    c, _ = chip(x, y + 8, "Same as the Plate · pine", WOOD, w=200)
    p.append(c)
    y += 56
    p.append(rect(x, y, PANEL_W - 40, 58, CHIP_DONE, rx=10) + text(x + 12, y + 24, "✓ Still fits your 19 mm board", 13, 700) + text(x + 12, y + 44, "Plate 12 + tab 15 = 27 mm: glue on a block", 12, 400, MUTED))
    y += 78
    p.append(button(x, y, 140, 40, "Pull up", True, "make.extrude") + button(x + 156, y, 120, 40, "Cancel"))
    body.append(context_panel("Make", "".join(p), "live preview: Enter keeps it, Esc drops it"))
    body.append(timeline(STEPS + [("make.extrude", "Pull up tab 15 mm", "")], selected=6) + status_bar("preview"))
    calls = callout(1, mid_top[0] - 30, mid_top[1] - 60) + callout(2, W - PANEL_W, VIEW_Y + 228) + callout(3, W - PANEL_W, VIEW_Y + 412) + callout(4, W - PANEL_W, VIEW_Y + 480) + callout(5, VIEW_X + VIEW_W - 350, VIEW_Y + VIEW_H - 170)
    return page("03", "3 · Pull up (extrude) with a live preview", "".join(body), calls)


def screen_4(render):
    s1 = screen_1(render)
    inner = re.search(r'<g id="screen">(.*)</g><g id="callouts">', s1, re.S).group(1)
    body = [inner, rect(0, 0, W, H, INK, extra=' opacity="0.35"')]
    x0, y0, w = W / 2 - 330, 90, 660
    body.append(rect(x0, y0, w, 520, PANEL, INK, rx=18, sw=2))
    body.append(rect(x0 + 16, y0 + 16, w - 32, 48, PAPER, ACCENT, rx=12, sw=2.5))
    body.append(icon("app.search", x0 + 28, y0 + 28, 24) + text(x0 + 62, y0 + 47, "put a hole for a screw", 18, 700))
    body.append(text(x0 + w - 32, y0 + 47, "Esc", 12, 700, MUTED, "end"))
    rows = [("make.hole", "Drill a hole", "Make · click a face, then where", "H", True),
            ("make.hole", "Screw hole with countersink", "Make · picks the size from your screws", "", False),
            ("make.repeat", "Repeat holes in a row", "Make · after a first hole", "", False),
            ("stock.kerf", "Pilot hole size for #8 screws", "Stock & Cut · not loaded: load it?", "", False),
            ("app.help", "How do I put a hole for a screw?", "Ask Sancho · explains with this part", "?", False)]
    y = y0 + 84
    body.append(text(x0 + 24, y + 4, "Best matches", 12, 800, MUTED))
    y += 14
    for icon_id, title, sub, key, on in rows:
        if on:
            body.append(rect(x0 + 12, y, w - 24, 62, CHIP_WAIT, rx=12))
        body.append(icon(icon_id, x0 + 26, y + 13, 36) + text(x0 + 76, y + 28, title, 16, 800) + text(x0 + 76, y + 48, sub, 13, 400, MUTED))
        if key:
            body.append(rect(x0 + w - 60, y + 18, 34, 26, CHIP, rx=6) + text(x0 + w - 43, y + 36, key, 13, 800, INK, "middle"))
        y += 66
    body.append(text(x0 + 24, y + 22, "Also understood: “make it thinner”, “round the corners”, “cut a slot for a cable”", 12, 400, MUTED))
    body.append(text(x0 + 24, y + 48, "↑ ↓ choose   ·   Enter run   ·   Tab change options first", 12, 700, MUTED))
    calls = callout(1, x0 - 20, y0 + 40) + callout(2, x0 + w + 20, y0 + 240) + callout(3, x0 + w + 20, y0 + 310) + callout(4, x0 - 20, y0 + 380)
    return page("04", "4 · Command search (Ctrl K)", "".join(body), calls)


def screen_5():
    body = [viewport_background()]
    iso = Iso(VIEW_X + VIEW_W / 2 - 20, VIEW_Y + VIEW_H / 2 - 20, 4.4)
    body.append(plate(iso, radius=4.0, round_highlight=True))
    a = iso.p(60, 40, 12)
    body.append(rect(a[0] + 18, a[1] - 10, 112, 32, PAPER, ACCENT, rx=8, sw=2.5) + text(a[0] + 30, a[1] + 12, "R 4", 16, 800) + text(a[0] + 120, a[1] + 12, "mm", 12, 400, MUTED, "end"))
    body.append(view_hud(["Changing step 3: Round over", "Steps after it rebuild when you press Done"]))
    body.append(project_bar() + env_tabs() + toolbox("make.round"))
    x = W - PANEL_W + 20
    y = VIEW_Y + 80
    p = [text(x, y, "Round over", 16, 800) + text(x, y + 20, "(fillet) 4 vertical corners", 13, 400, MUTED)]
    y += 58
    p.append(field(x, y, 180, "Radius", "4", "mm", True) + text(x + 196, y + 22, "was 8 mm", 13, 400, MUTED))
    y += 64
    p.append(text(x, y, "Corners", 12, 700, MUTED) + text(x, y + 22, "4 picked · click to add or drop", 13))
    y += 50
    p.append(text(x, y, "Your tools", 15, 800))
    y += 12
    p.append(rect(x, y, PANEL_W - 40, 74, CHIP_DONE, rx=10) + text(x + 12, y + 24, "✓ Router with a ⅛ in round-over bit", 13, 700)
             + text(x + 12, y + 44, "3.2 mm: close to 4 mm. Use 3.2?", 12, 400, MUTED))
    y += 92
    p.append(button(x, y, 170, 36, "Use 3.2 mm (⅛ in)", False) + text(x + 186, y + 24, "or keep 4", 13, 400, MUTED))
    y += 60
    p.append(text(x, y, "Changed by you · just now", 12, 400, MUTED) + text(x, y + 18, "Step 5 and 6 rebuild: no problems", 12, 700, OK))
    y += 44
    p.append(button(x, y, 140, 40, "Done", True) + button(x + 156, y, 120, 40, "Cancel"))
    body.append(context_panel("Change a step", "".join(p), "step 3 of 6"))
    steps = list(STEPS)
    steps[2] = ("make.round", "Round over → 4 mm ×4", "")
    body.append(timeline(steps, selected=2, rolled_back_after=2) + status_bar("rolled back to step 3"))
    calls = callout(1, 300, H - STATUS_H - 30) + callout(2, VIEW_X + 640, VIEW_Y + 210) + callout(3, W - PANEL_W, VIEW_Y + 282) + callout(4, W - PANEL_W, VIEW_Y + 480)
    return page("05", "5 · Changing a step from the timeline", "".join(body), calls)


def screen_6(render):
    body = [viewport_background(), view_image(render) if render else ""]
    # Measuring-stick dimensions over the render (placed by eye on the render).
    body.append(dim_v(722, 337, 616, "1.68 m") + dim_v(462, 398, 566, "900 mm", left=True))
    body.append(view_hud(["Sancho beside the bench", "Stage: bench (part up to 500 mm)"]))
    body.append(project_bar() + env_tabs() + toolbox())
    x = W - PANEL_W + 20
    y = VIEW_Y + 76
    p = [text(x, y, "Who stands beside your work", 14, 800)]
    y += 18
    p.append(radio(x, y, True, "Sancho (for now)", "BodyKun by vinchau, CC0") )
    y += 44
    p.append(radio(x, y, False, "My own figure…", "any humanoid glTF, rigged"))
    y += 44
    p.append(radio(x, y, False, "A scan of me…", "from your phone; we rig it"))
    y += 70
    p.append(field(x, y, 140, "Height", "1.68", "m") + text(x + 156, y + 22, "5 ft 6 in", 13, 400, MUTED))
    y += 60
    p.append(text(x, y, "Stage", 14, 800))
    y += 10
    p.append(segmented(x, y, ["Auto", "Bench", "Floor", "Room"], 0, w_each=74))
    y += 44
    p.append(text(x, y, "Auto: parts up to 500 mm go on the bench", 12, 400, MUTED))
    y += 34
    p.append(text(x, y, "Measure with him", 14, 800))
    y += 14
    for on, label in [(True, "Height"), (True, "Hand length (185 mm)"), (False, "Finger width (18 mm)"), (False, "Reach and eye height")]:
        p.append(checkbox(x, y, on, label))
        y += 28
    y += 22
    p.append(text(x, y, "Pose", 14, 800))
    y += 10
    p.append(segmented(x, y, ["Stand", "Lean in", "Hold it"], 0, w_each=96))
    body.append(context_panel("Sancho", "".join(p), "a measuring stick that looks like a person"))
    body.append(timeline() + status_bar())
    calls = callout(1, W - PANEL_W, VIEW_Y + 120) + callout(2, W - PANEL_W, VIEW_Y + 250) + callout(3, W - PANEL_W, VIEW_Y + 330) + callout(4, W - PANEL_W, VIEW_Y + 420) + callout(5, W - PANEL_W, VIEW_Y + 535)
    return page("06", "6 · Sancho and the stage", "".join(body), calls)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--render-dir", default=None, help="folder with view-bench.png (MakerCAD's viewport render)")
    ap.add_argument("--drafts", nargs="*", default=[], help="icons_*.py drafts for icons still in rework")
    ap.add_argument("--png", action="store_true", help="also draw each screen as a PNG with headless Edge")
    ap.add_argument("--profile", default=os.path.join(os.environ.get("TEMP", "."), "mk-ui-edge"), help="Edge profile folder for --png")
    a = ap.parse_args()
    add_drafts(a.drafts)
    render = os.path.join(a.render_dir, "view-bench.png") if a.render_dir else None
    if render and not os.path.exists(render):
        render = None
    screens = {
        "01-design-bench.svg": screen_1(render),
        "02-sketch-on-a-face.svg": screen_2(),
        "03-pull-up.svg": screen_3(),
        "04-command-search.svg": screen_4(render),
        "05-change-a-step.svg": screen_5(),
        "06-sancho-stage.svg": screen_6(render),
    }
    for name, svg in screens.items():
        ET.fromstring(svg)  # well-formed, or this raises
        with open(os.path.join(HERE, name), "w", encoding="utf-8", newline="\n") as f:
            f.write(svg)
        print("wrote", name, f"{len(svg) // 1024} KB")
        if a.png:
            png(os.path.join(HERE, name), a.profile)


EDGE = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"


def png(svg_path, profile):
    """A PNG beside the SVG, drawn by headless Edge at 1440 × 900."""
    import pathlib
    import subprocess

    out = svg_path[:-4] + ".png"
    url = pathlib.Path(svg_path).as_uri()
    subprocess.run([EDGE, "--headless=new", "--disable-gpu", "--hide-scrollbars", f"--user-data-dir={profile}",
                    f"--window-size={W},{H}", f"--screenshot={out}", url],
                   # Not captured: Edge's helper processes would hold the pipes open.
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=120, check=False)
    print("  png", os.path.basename(out), os.path.exists(out))


if __name__ == "__main__":
    main()
