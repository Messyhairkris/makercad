"""P0 measurement: does stock SolidCraft run on this machine, and how well?

    python tools/p0_measure.py [--backend dx12|vulkan|gl] [--exe PATH] [--frames 120]

Starts SolidCraft with its sample part and a control channel (docs/control-protocol.md in the
pinned checkout), then measures:
  start      seconds from launch until the control channel answers with a document open
  adapter    the graphics adapter and backend SolidCraft reports (ui.inspect)
  orbit      the app's own frame time while the camera turns 3 degrees per step (ui.camera)
  memory     the process's working set after 10 s idle
and writes one JSON line to stdout, plus a screenshot next to --out. Exits 1 when SolidCraft
did not start or did not answer. Nothing here changes SolidCraft's settings files.
"""

import argparse
import json
import math
import os
import secrets
import socket
import statistics
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_EXE = os.path.join(REPO, "third_party", "solidcraft", "target", "debug", "solidcraft.exe")


class Channel:
    def __init__(self, port, token):
        self.sock = socket.create_connection(("127.0.0.1", port), timeout=30)
        self.file = self.sock.makefile("rwb")
        self.token = token
        self.next_id = 0

    def call(self, method, params=None):
        self.next_id += 1
        msg = {"id": self.next_id, "method": method, "params": params or {}}
        if self.token:
            msg["token"], self.token = self.token, None
        self.file.write((json.dumps(msg) + "\n").encode())
        self.file.flush()
        while True:
            line = self.file.readline()
            if not line:
                raise ConnectionError("the app closed the control channel")
            reply = json.loads(line)
            if reply.get("id") == self.next_id:
                if not reply.get("ok"):
                    raise RuntimeError(f"{method}: {reply.get('error')}")
                return reply.get("result")


def memory_mb(pid):
    """(working set, private commit) in MB, read straight from Windows. Starting PowerShell to ask
    took over 30 s on the reference machine, so no subprocess here."""
    import ctypes
    from ctypes import wintypes

    class Counters(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD),
                    ("PeakWorkingSetSize", ctypes.c_size_t), ("WorkingSetSize", ctypes.c_size_t),
                    ("QuotaPeakPagedPoolUsage", ctypes.c_size_t), ("QuotaPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t), ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                    ("PagefileUsage", ctypes.c_size_t), ("PeakPagefileUsage", ctypes.c_size_t)]

    k32 = ctypes.WinDLL("kernel32", use_last_error=True)
    k32.OpenProcess.restype = wintypes.HANDLE
    handle = k32.OpenProcess(0x1000 | 0x0010, False, pid)  # QUERY_LIMITED_INFORMATION | VM_READ
    if not handle:
        return None, None
    try:
        c = Counters()
        c.cb = ctypes.sizeof(Counters)
        if not k32.K32GetProcessMemoryInfo(handle, ctypes.byref(c), c.cb):
            return None, None
        return round(c.WorkingSetSize / 2**20, 1), round(c.PagefileUsage / 2**20, 1)
    finally:
        k32.CloseHandle(handle)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--backend", default="")
    ap.add_argument("--exe", default=DEFAULT_EXE)
    ap.add_argument("--port", type=int, default=27990)
    ap.add_argument("--frames", type=int, default=120)
    ap.add_argument("--timeout", type=float, default=180.0)
    ap.add_argument("--spin", type=float, default=0.0,
                    help="seconds of continuous spin with the frame-rate readout on, then a screenshot")
    ap.add_argument("--out", default=os.path.join(REPO, "target", "p0"))
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    label = a.backend or "default"
    result = {"backend_asked": label, "exe": a.exe}

    env = dict(os.environ)
    token = secrets.token_hex(32)
    env["SOLIDCRAFT_CONTROL_TOKEN"] = token
    if a.backend:
        env["WGPU_BACKEND"] = a.backend
    else:
        env.pop("WGPU_BACKEND", None)
    log_path = os.path.join(a.out, f"solidcraft-{label}.log")
    result["log"] = log_path
    t0 = time.perf_counter()
    with open(log_path, "wb") as log:
        proc = subprocess.Popen([a.exe, "--new-instance", "--control", str(a.port), "--sample"],
                                env=env, stdout=log, stderr=subprocess.STDOUT)
    result["pid"] = proc.pid
    ch = None
    try:
        # Start: the channel answers and a document is open.
        while time.perf_counter() - t0 < a.timeout:
            if proc.poll() is not None:
                result["started"] = False
                result["exit_code"] = proc.returncode
                result["error"] = "SolidCraft exited before answering"
                return finish(result, 1)
            try:
                if ch is None:
                    ch = Channel(a.port, token)
                info = ch.call("ui.inspect")
                if info.get("documents", 0) >= 1 and not info.get("file_busy"):
                    break
            except (OSError, ConnectionError):
                ch = None
            time.sleep(0.25)
        else:
            result["started"] = False
            result["error"] = f"no answer with a document within {a.timeout:.0f} s"
            return finish(result, 1)
        result["started"] = True
        result["start_s"] = round(time.perf_counter() - t0, 2)
        result["adapter"] = info.get("adapter")
        result["renderer"] = info.get("renderer")

        # Orbit: turn the camera a step at a time and read the app's frame time.
        cam = ch.call("ui.camera") or {}
        yaw0 = float(cam.get("yaw", 0.0))
        frame_ms, wall = [], []
        for i in range(a.frames):
            t = time.perf_counter()
            ch.call("ui.camera", {"yaw": yaw0 + math.radians(3.0 * (i + 1))})
            info = ch.call("ui.inspect")
            wall.append((time.perf_counter() - t) * 1000.0)
            if isinstance(info.get("frame_ms"), (int, float)) and info["frame_ms"] > 0:
                frame_ms.append(float(info["frame_ms"]))
        result["orbit_steps"] = a.frames
        if frame_ms:
            q = statistics.quantiles(frame_ms, n=20)
            result["frame_ms_median"] = round(statistics.median(frame_ms), 2)
            result["frame_ms_p95"] = round(q[-1], 2)
            result["fps_from_median"] = round(1000.0 / statistics.median(frame_ms), 1)
        result["step_round_trip_ms_median"] = round(statistics.median(wall), 2)
        result["draw"] = info.get("draw")

        # Evidence: a screenshot of the window.
        shot(ch, os.path.join(a.out, f"solidcraft-{label}.png"), result, "screenshot")

        # Spin: frame_ms above is the UI's own time per frame, not frame to frame. The real rate
        # (frame to frame, the 1% low and the worst frame) is only painted by SolidCraft's
        # frame-rate readout, so turn the view continuously with the readout on and capture it.
        if a.spin > 0:
            try:
                ch.call("ui.prefs", {"show_frame_rate": True, "frame_rate_details": True})
                ch.call("engine.execute", {"command": "view.spin", "params": {"on": True}})
                time.sleep(a.spin)
                shot(ch, os.path.join(a.out, f"solidcraft-{label}-spin.png"), result, "spin_screenshot")
                result["spin_s"] = a.spin
            except Exception as e:
                result["spin_error"] = str(e)[:200]
            finally:
                for method, params in (("engine.execute", {"command": "view.spin", "params": {"on": False}}),
                                       ("ui.prefs", {"show_frame_rate": False, "frame_rate_details": False})):
                    try:
                        ch.call(method, params)
                    except Exception:
                        pass

        # Memory after 10 s idle.
        time.sleep(10)
        result["idle_working_set_mb"], result["idle_private_mb"] = memory_mb(proc.pid)
        return finish(result, 0)
    except Exception as e:
        result["error"] = f"{type(e).__name__}: {e}"[:400]
        return finish(result, 1)
    finally:
        try:
            if ch is not None:
                ch.call("app.quit")
        except Exception:
            pass
        try:
            proc.wait(timeout=20)
        except subprocess.TimeoutExpired:
            proc.kill()


def shot(ch, path, result, key):
    """Ask for a screenshot and wait for the PNG (written a frame or two later); optional evidence."""
    try:
        if os.path.isfile(path):
            os.remove(path)
        ch.call("ui.screenshot", {"path": path})
        for _ in range(40):
            if os.path.isfile(path):
                result[key] = path
                return
            ch.call("ui.inspect")
            time.sleep(0.1)
    except Exception as e:
        result[key + "_error"] = str(e)[:200]


def finish(result, code):
    print(json.dumps(result))
    return code


if __name__ == "__main__":
    sys.exit(main())
