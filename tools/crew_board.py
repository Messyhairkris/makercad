"""One picture of what the Crew is doing for MakerCAD, for Kris's check-ins from his phone.

    python tools/crew_board.py [--out target/crew/board.png] [--jobs JOB_ID ...]

Reads only status files the Crew already writes (campaigns, PA builds, single jobs) and the
machine's free disk and memory; no model is called. Writes board.html and renders board.png
with Edge headless. Campaigns shown are the MakerCAD ones (names starting mk-).
"""

import argparse
import datetime as dt
import html
import json
import os
import shutil
import subprocess
from pathlib import Path

HOME = Path.home()
CREW = HOME / "ATLAS-api-harness" / "runs" / "crew"
EDGE = [Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
        Path(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe")]

STATE_COLOUR = {"accepted": "#2e7d4f", "done": "#2e7d4f", "passed": "#2e7d4f",
                "running": "#b26a00", "waiting": "#6b6b6b", "review": "#2e7d4f",
                "failed": "#b3261e", "stopped": "#b3261e"}


def load(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def chip(state: str) -> str:
    colour = STATE_COLOUR.get(state, "#6b6b6b")
    return f'<span class="chip" style="background:{colour}">{html.escape(state)}</span>'


def campaigns() -> list:
    rows = []
    for folder in sorted((CREW / "campaigns").glob("mk-*")):
        st = load(folder / "status.json")
        if not st:
            continue
        tasks = st.get("tasks", {})
        done = (folder / "summary.json").exists()
        counts = st.get("counts", {})
        state = "failed" if counts.get("failed") else ("done" if done else "running")
        who = ", ".join(f"{t}: {v.get('member', '?')} ({v.get('state')})"
                        for t, v in tasks.items())
        rows.append((st.get("updated", ""), folder.name, state, who,
                     f"{st.get('requests_used', 0)}/{st.get('requests_budget', 0)}"))
    rows.sort(reverse=True)
    return rows[:8]


def pa_builds() -> list:
    rows = []
    for folder in sorted((CREW / "pa" / "builds").glob("*mk-*")):
        st = load(folder / "status.json") or {}
        guard = st.get("guard", {})
        rows.append((st.get("started", ""), st.get("pa", "?"), st.get("branch", folder.name),
                     st.get("state", "?"), st.get("round", "?"), guard.get("tool_calls", 0)))
    rows.sort(reverse=True)
    return rows[:6]


def single_jobs(ids: list) -> list:
    rows = []
    for job_id in ids:
        job = load(CREW / "jobs" / job_id / "job.json") or {}
        spec = job.get("spec", {})
        rows.append((spec.get("title", job_id), spec.get("assigned_to", "?"),
                     job.get("status", "?"), job.get("stop_reason", ""), job.get("requests", "")))
    return rows


def machine() -> str:
    free = shutil.disk_usage("C:\\").free / 2**30
    ram = ""
    try:
        import ctypes

        class MemoryStatus(ctypes.Structure):
            _fields_ = [("length", ctypes.c_ulong), ("load", ctypes.c_ulong),
                        ("total", ctypes.c_ulonglong), ("avail", ctypes.c_ulonglong),
                        ("total_page", ctypes.c_ulonglong), ("avail_page", ctypes.c_ulonglong),
                        ("total_virtual", ctypes.c_ulonglong),
                        ("avail_virtual", ctypes.c_ulonglong), ("extended", ctypes.c_ulonglong)]

        status = MemoryStatus()
        status.length = ctypes.sizeof(MemoryStatus)
        if ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
            ram = f" &middot; RAM free {status.avail / 2**30:.1f} of {status.total / 2**30:.1f} GB"
    except (OSError, AttributeError):
        pass
    return f"C: free {free:.1f} GB{ram}"


def page(jobs: list) -> str:
    now = dt.datetime.now().strftime("%Y-%m-%d %H:%M")
    parts = [f"<h1>MakerCAD &middot; the Crew</h1><p class='sub'>{now} &middot; {machine()}</p>"]
    parts.append("<h2>Campaigns</h2><table><tr><th>campaign</th><th>state</th>"
                 "<th>who</th><th>requests</th></tr>")
    for _, name, state, who, req in campaigns():
        parts.append(f"<tr><td>{html.escape(name)}</td><td>{chip(state)}</td>"
                     f"<td>{html.escape(who)}</td><td>{req}</td></tr>")
    parts.append("</table><h2>PA jobs (reference + hidden tests)</h2><table><tr><th>PA</th>"
                 "<th>branch</th><th>state</th><th>round</th><th>tool calls</th></tr>")
    for _, pa, branch, state, rnd, calls in pa_builds():
        parts.append(f"<tr><td>{html.escape(pa)}</td><td>{html.escape(branch)}</td>"
                     f"<td>{chip(state)}</td><td>{rnd}</td><td>{calls}</td></tr>")
    parts.append("</table>")
    if jobs:
        parts.append("<h2>Drafts</h2><table><tr><th>draft</th><th>member</th><th>state</th>"
                     "<th>stop</th><th>requests</th></tr>")
        for title, who, state, stop, req in single_jobs(jobs):
            parts.append(f"<tr><td>{html.escape(title)}</td><td>{html.escape(who)}</td>"
                         f"<td>{chip(state)}</td><td>{html.escape(stop)}</td><td>{req}</td></tr>")
        parts.append("</table>")
    style = ("body{font-family:Segoe UI,sans-serif;margin:24px;background:#fbf7ef;color:#222}"
             "h1{margin:0;font-size:26px}h2{font-size:18px;margin:22px 0 8px}"
             ".sub{color:#666;margin:4px 0 0}table{border-collapse:collapse;width:100%}"
             "th,td{text-align:left;padding:6px 10px;border-bottom:1px solid #e4dccd;"
             "font-size:14px;vertical-align:top}th{color:#555;font-weight:600}"
             ".chip{color:#fff;border-radius:10px;padding:1px 9px;font-size:12px}")
    return f"<!doctype html><meta charset='utf-8'><style>{style}</style>{''.join(parts)}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="target/crew/board.png")
    ap.add_argument("--jobs", nargs="*", default=[])
    args = ap.parse_args()
    out = Path(args.out).resolve()
    out.parent.mkdir(parents=True, exist_ok=True)
    page_file = out.with_suffix(".html")
    page_file.write_text(page(args.jobs), encoding="utf-8")
    edge = next((e for e in EDGE if e.exists()), None)
    if edge is None:
        print(f"wrote {page_file} (no Edge for the picture)")
        return 0
    profile = out.parent / "edge-profile"
    before = out.stat().st_mtime if out.exists() else 0
    try:
        # Slow to start when memory is tight (a PA job running): give it time.
        subprocess.run([str(edge), "--headless=new", "--disable-gpu", "--hide-scrollbars",
                        "--no-first-run", "--disable-extensions",
                        f"--user-data-dir={profile}", "--window-size=1100,900",
                        f"--screenshot={out}", page_file.as_uri()],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=300)
    except subprocess.TimeoutExpired:
        print("Edge did not finish in 300 s")
    fresh = out.exists() and out.stat().st_mtime > before
    print(f"wrote {out}" if fresh else f"wrote {page_file}; the picture was not refreshed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
