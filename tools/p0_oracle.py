"""P0: replay SolidCraft's reference parts (its oracle corpus) on this machine and time each one.

    python tools/p0_oracle.py [--cli PATH] [--only 07-fillet] [--timeout 600]

Runs `solidcraft-cli oracle <case> --json` once per case in `tests/oracle/` of the pinned checkout,
so each case gets its own wall-clock time. That time covers the whole check: replaying the recipe
through the command engine (the rebuild), measuring, the STEP export and read-back, and the surface
comparison where the case has a reference.stl. Writes target/p0/oracle.json and prints a summary.
"""

import argparse
import json
import os
import statistics
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SC = os.path.join(REPO, "third_party", "solidcraft")
DEFAULT_CLI = os.path.join(SC, "target", "debug", "solidcraft-cli.exe")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cli", default=DEFAULT_CLI)
    ap.add_argument("--only", default="", help="run only cases whose name contains this")
    ap.add_argument("--timeout", type=float, default=600.0)
    ap.add_argument("--out", default=os.path.join(REPO, "target", "p0", "oracle.json"))
    a = ap.parse_args()

    root = os.path.join(SC, "tests", "oracle")
    cases = sorted(d for d in os.listdir(root)
                   if os.path.isfile(os.path.join(root, d, "recipe.json")) and a.only in d)
    rows = []
    for name in cases:
        t = time.perf_counter()
        try:
            p = subprocess.run([a.cli, "oracle", os.path.join(root, name), "--json"], cwd=SC,
                               capture_output=True, text=True, timeout=a.timeout)
            secs = time.perf_counter() - t
            try:
                r = json.loads(p.stdout)[0]
            except (ValueError, IndexError):
                r = {"error": (p.stderr or p.stdout).strip()[-400:] or f"exit {p.returncode}"}
        except subprocess.TimeoutExpired:
            secs = time.perf_counter() - t
            r = {"error": f"timed out after {a.timeout:.0f} s"}
        status = "pass" if r.get("pass") else "skip" if r.get("skipped") else "fail"
        failed = [f"{c.get('check')} got {c.get('got')} want {c.get('want')}"
                  for c in r.get("checks", []) if not c.get("ok")]
        row = {"case": name, "status": status, "seconds": round(secs, 2),
               "error": r.get("error"), "failed_checks": failed}
        rows.append(row)
        print(f"{status.upper():4} {secs:7.2f} s  {name}" + (f"  ({r['error']})" if r.get("error") else "")
              + (f"  {'; '.join(failed)[:300]}" if failed else ""), flush=True)

    times = [r["seconds"] for r in rows]
    summary = {
        "cases": len(rows),
        "pass": sum(r["status"] == "pass" for r in rows),
        "fail": sum(r["status"] == "fail" for r in rows),
        "skip": sum(r["status"] == "skip" for r in rows),
        "seconds_total": round(sum(times), 1),
        "seconds_median": round(statistics.median(times), 2) if times else None,
        "seconds_max": max(times) if times else None,
        "slowest": sorted(rows, key=lambda r: -r["seconds"])[:5],
    }
    os.makedirs(os.path.dirname(a.out), exist_ok=True)
    with open(a.out, "w", encoding="utf-8") as f:
        json.dump({"summary": summary, "cases": rows}, f, indent=1)
    print(json.dumps({k: v for k, v in summary.items() if k != "slowest"}))
    print("slowest:", ", ".join(f"{r['case']} {r['seconds']} s" for r in summary["slowest"]))
    return 0 if summary["fail"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
