"""Gather openly licensed CAD-interface screenshots from Wikimedia Commons for local UI reference.

Writes <out>/<program>/<file>.jpg|png and <out>/SOURCES.csv (program, title, licence, artist,
page url, file url). Downloads a 1600 px rendition, never the original. Read-only on the web.
"""
import csv
import json
import re
import sys
import time
import urllib.parse
import urllib.error
import urllib.request
from pathlib import Path

UA = "makercad-ui-reference/0.1 (local research; contact via repository owner)"
API = "https://commons.wikimedia.org/w/api.php"
PROGRAMS = {
    "freecad": ["FreeCAD screenshot", "FreeCAD Part Design", "FreeCAD sketcher"],
    "solvespace": ["SolveSpace screenshot"],
    "librecad": ["LibreCAD screenshot"],
    "openscad": ["OpenSCAD screenshot"],
    "blender": ["Blender 4 interface screenshot", "Blender user interface"],
    "kicad": ["KiCad 3D viewer screenshot", "KiCad screenshot"],
    "brlcad": ["BRL-CAD screenshot"],
    "fusion360": ["Fusion 360 screenshot", "Autodesk Fusion 360"],
    "onshape": ["Onshape screenshot"],
    "solidworks": ["SolidWorks screenshot"],
    "sketchup": ["SketchUp screenshot"],
    "inventor": ["Autodesk Inventor screenshot"],
    "rhino": ["Rhinoceros 3D screenshot"],
    "autocad": ["AutoCAD screenshot"],
    "tinkercad": ["Tinkercad screenshot"],
    "plasticity": ["Plasticity CAD screenshot"],
    "dune3d": ["Dune 3D screenshot"],
}
PER_PROGRAM = 6
SKIP = re.compile(r"logo|icon|\.svg$|\.pdf$|\.webm$|\.ogv$|\.tif", re.I)


def _open(req, timeout):
    for wait in (0, 60, 180):
        if wait:
            time.sleep(wait)
        try:
            return urllib.request.urlopen(req, timeout=timeout)
        except urllib.error.HTTPError as e:
            if e.code != 429:
                raise
    raise RuntimeError("still rate limited after backoff")


def get(params):
    q = API + "?" + urllib.parse.urlencode({**params, "format": "json"})
    req = urllib.request.Request(q, headers={"User-Agent": UA})
    with _open(req, 60) as r:
        return json.loads(r.read().decode("utf-8"))


def search(term):
    data = get({"action": "query", "list": "search", "srsearch": term, "srnamespace": 6,
                "srlimit": 20})
    return [h["title"] for h in data.get("query", {}).get("search", [])]


def info(titles):
    data = get({"action": "query", "titles": "|".join(titles), "prop": "imageinfo",
                "iiprop": "url|extmetadata|size|mime", "iiurlwidth": 1280})
    out = []
    for page in data.get("query", {}).get("pages", {}).values():
        ii = (page.get("imageinfo") or [None])[0]
        if not ii:
            continue
        meta = ii.get("extmetadata") or {}
        def m(k):
            return re.sub(r"<[^>]+>", "", (meta.get(k) or {}).get("value", "")).strip()
        out.append({"title": page["title"], "url": ii.get("thumburl") or ii.get("url"),
                    "page": ii.get("descriptionurl"), "licence": m("LicenseShortName"),
                    "artist": m("Artist")[:120], "mime": ii.get("mime", ""),
                    "width": ii.get("width", 0)})
    return out


def main(out_dir):
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    done = {d.name for d in out.iterdir() if d.is_dir() and any(d.iterdir())}
    prev = out / "SOURCES.csv"
    if prev.is_file():
        with prev.open(encoding="utf-8") as fh:
            rows = list(csv.DictReader(fh))
    for prog, terms in PROGRAMS.items():
        if prog in done:
            continue
        titles = []
        for t in terms:
            try:
                for title in search(t):
                    if title not in titles and not SKIP.search(title):
                        titles.append(title)
            except Exception as e:  # noqa: BLE001
                print(f"{prog}: search failed: {e}", file=sys.stderr)
            time.sleep(2)
        got = 0
        for chunk in [titles[i:i + 20] for i in range(0, len(titles), 20)]:
            try:
                infos = info(chunk)
            except Exception as e:  # noqa: BLE001
                print(f"{prog}: info failed: {e}", file=sys.stderr)
                continue
            for it in infos:
                if got >= PER_PROGRAM:
                    break
                if not it["mime"].startswith("image/") or it["mime"] == "image/svg+xml":
                    continue
                if it["width"] and it["width"] < 800:
                    continue
                ext = ".png" if "png" in it["mime"] else ".jpg"
                name = re.sub(r"[^A-Za-z0-9._-]+", "_", it["title"].split(":", 1)[-1])[:90]
                name = Path(name).stem + ext
                dest = out / prog / name
                dest.parent.mkdir(parents=True, exist_ok=True)
                try:
                    req = urllib.request.Request(it["url"], headers={"User-Agent": UA})
                    with _open(req, 120) as r:
                        data = r.read()
                    if len(data) > 8_000_000:
                        continue
                    dest.write_bytes(data)
                except Exception as e:  # noqa: BLE001
                    print(f"{prog}: download failed {it['title']}: {e}", file=sys.stderr)
                    continue
                rows.append({"program": prog, "file": f"{prog}/{name}", "title": it["title"],
                             "licence": it["licence"], "artist": it["artist"],
                             "page": it["page"], "url": it["url"]})
                got += 1
                time.sleep(4)
        print(f"{prog}: {got} image(s)")
    with (out / "SOURCES.csv").open("w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=["program", "file", "title", "licence", "artist",
                                           "page", "url"])
        w.writeheader()
        w.writerows(rows)
    print(f"total {len(rows)}")


if __name__ == "__main__":
    main(sys.argv[1])

