"""Start a real build and stop it before it recompiles SolidCraft's heavy crates.

    python tools/reuse_check.py [-p mk-desktop] [--allow-minutes 20]

`cargo build -v` prints, for every crate, `Fresh` (reused) or `Dirty ...: <reason>` before it
compiles anything above that crate. This runs the build with -j 1 and watches those lines. If
solidcore or solidcraft-engine is about to be recompiled it kills the build at once (the whole
process tree) and prints the reasons Cargo gave, so the cause can be fixed instead of paying
hours of compiling on the reference machine. Small crates that are dirty are left to compile.
Exits 0 when the build finished without touching a heavy crate, 1 when it was stopped.
"""

import argparse
import os
import re
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HEAVY = ("solidcore", "solidcraft-engine")


def kill_tree(pid):
    subprocess.run(["taskkill", "/T", "/F", "/PID", str(pid)], capture_output=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("-p", "--package", default="mk-desktop")
    ap.add_argument("--allow-minutes", type=float, default=20.0)
    a = ap.parse_args()
    env = dict(os.environ, CARGO_TERM_COLOR="never")
    proc = subprocess.Popen(["cargo", "build", "-p", a.package, "-j", "1", "-v"], cwd=REPO, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, bufsize=1)
    t0 = time.time()
    dirty, compiled, fresh = [], [], 0
    stopped = None
    pending = None  # a heavy crate whose rustc command line we still want to see
    for line in proc.stderr:
        line = line.rstrip()
        s = line.strip()
        if pending and s.startswith("Running ") and f"--crate-name {pending.replace('-', '_')} " in s:
            # A crate that is new rather than changed gets no `Dirty` line. Its command line says
            # why: compare -C metadata and each --extern with what target/debug/deps holds.
            print("RUSTC", s, flush=True)
            stopped = f"about to compile {pending}"
        if stopped:
            kill_tree(proc.pid)
            break
        if s.startswith("Fresh "):
            fresh += 1
        elif s.startswith("Dirty "):
            dirty.append(s)
            print(s[:400], flush=True)
        elif s.startswith("Compiling "):
            name = s.split()[1]
            compiled.append(name)
            print(s[:200], flush=True)
            if name in HEAVY:
                pending = name
                continue
        elif s.startswith("error"):
            print(s[:400], flush=True)
        if not stopped and any(s.startswith(f"Dirty {h} ") for h in HEAVY):
            stopped = "a heavy crate is dirty: " + s[:300]
        if not stopped and time.time() - t0 > a.allow_minutes * 60:
            stopped = f"over {a.allow_minutes} minutes"
        if stopped:
            kill_tree(proc.pid)
            break
    proc.wait()
    print(f"fresh {fresh}, dirty {len(dirty)}, compiled {len(compiled)}: {', '.join(compiled)[:600]}")
    if stopped:
        print("STOPPED:", stopped)
        return 1
    print("finished" if proc.returncode == 0 else f"cargo exited {proc.returncode}")
    return 0 if proc.returncode == 0 else 2


if __name__ == "__main__":
    sys.exit(main())
