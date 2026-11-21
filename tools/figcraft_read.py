"""Reads a FigCraft file (.figcraft) and prints what a person wrote in it: pages and frames, every
text layer, comment threads, annotations and the prototype links between frames (decision D29).

    python tools/figcraft_read.py NOTES.figcraft [--json] [--depth 3]

A .figcraft file is a zip of `document.json` (FigCraft's Document, serialised with serde: pages,
a map of nodes by id, comments) and images. Nodes carry `"type"` (`page`, `frame`, `section`,
`text`, `component`, `instance`, …), `name`, `children`, and per type fields: a text node's words
are in `characters`; a comment thread has `author`, `text`, `resolved` and `replies`. Read from
FigCraft's source (storytold/figcraft, `crates/doc`, Apache-2.0) on 2026-11-21; anything the
reader does not recognise is skipped, never fatal.

The file is someone else's data: it is read from the zip in memory (nothing is extracted), sizes
are checked before reading, and nothing in it is run or followed.
"""

import argparse
import json
import sys
import zipfile

MAX_JSON = 256 * 1024 * 1024
FRAMEISH = {"frame", "section", "component", "componentSet", "instance", "group"}


class Doc:
    def __init__(self, data):
        self.data = data if isinstance(data, dict) else {}
        raw = self.data.get("nodes") or {}
        if isinstance(raw, list):  # a list of nodes instead of a map, just in case
            raw = {str(n.get("id")): n for n in raw if isinstance(n, dict)}
        self.nodes = {str(k): v for k, v in raw.items() if isinstance(v, dict)}

    def node(self, node_id):
        return self.nodes.get(str(node_id))

    def name(self, node_id):
        n = self.node(node_id)
        return (n or {}).get("name") or f"#{node_id}"

    def children(self, node_id):
        n = self.node(node_id) or {}
        return [str(c) for c in n.get("children") or []]

    def pages(self):
        ids = [str(p) for p in self.data.get("pages") or []]
        return ids or [k for k, n in self.nodes.items() if n.get("type") == "page"]

    def path(self, node_id, stop_at_page=True):
        """Names from the page's child down to the node: 'Design view / Panel / Title'."""
        names, seen = [], set()
        cur = self.node(node_id)
        while cur and str(cur.get("id")) not in seen:
            seen.add(str(cur.get("id")))
            if cur.get("type") == "page" and stop_at_page:
                break
            names.append(cur.get("name") or f"#{cur.get('id')}")
            parent = cur.get("parent")
            cur = self.node(parent) if parent is not None else None
        return " / ".join(reversed(names))

    def page_of(self, node_id):
        cur, seen = self.node(node_id), set()
        while cur and str(cur.get("id")) not in seen:
            seen.add(str(cur.get("id")))
            if cur.get("type") == "page":
                return str(cur.get("id"))
            parent = cur.get("parent")
            cur = self.node(parent) if parent is not None else None
        return None


def read(path):
    """The document and the names of the other files in the archive."""
    with zipfile.ZipFile(path) as z:
        names = z.namelist()
        member = "document.json" if "document.json" in names else next((n for n in names if n.endswith(".json") and "/" not in n), None)
        if member is None:
            raise ValueError("no document.json in this file: is it a .figcraft file?")
        info = z.getinfo(member)
        if info.file_size > MAX_JSON:
            raise ValueError(f"{member} is {info.file_size} bytes, more than this reader takes ({MAX_JSON})")
        with z.open(member) as f:
            data = json.loads(f.read(MAX_JSON + 1).decode("utf-8"))
    return Doc(data), [n for n in names if n != member]


def destinations(value):
    """Node ids an interaction (or anything inside it) points at."""
    out = []
    if isinstance(value, dict):
        for k, v in value.items():
            if k in ("destination", "destinationId", "destination_id", "target", "overlay") and isinstance(v, (int, str)):
                out.append(str(v))
            else:
                out.extend(destinations(v))
    elif isinstance(value, list):
        for v in value:
            out.extend(destinations(v))
    return out


def summary(doc, depth=3):
    """Everything a person wrote, as plain data."""
    out = {"name": doc.data.get("name"), "kind": doc.data.get("kind", "design"), "pages": []}
    for pid in doc.pages():
        page = {"name": doc.name(pid), "outline": [], "texts": [], "links": [], "annotations": [], "comments": []}

        def walk(nid, level):
            n = doc.node(nid) or {}
            kind = n.get("type", "?")
            if level <= depth and (kind in FRAMEISH or level == 1):
                status = n.get("dev_status") or n.get("devStatus")
                page["outline"].append({"level": level, "type": kind, "name": n.get("name", ""),
                                        **({"status": status} if status not in (None, "none", "None") else {})})
            if kind == "text" and str(n.get("characters", "")).strip():
                page["texts"].append({"where": doc.path(nid), "text": n.get("characters")})
            for d in destinations(n.get("interactions") or []):
                page["links"].append({"from": doc.path(nid), "to": doc.path(d) or doc.name(d)})
            for a in n.get("annotations") or ([n["annotation"]] if isinstance(n.get("annotation"), dict) else []):
                if isinstance(a, dict) and (a.get("label") or a.get("properties")):
                    page["annotations"].append({"on": doc.path(nid), "label": a.get("label", ""),
                                                "category": a.get("category"), "properties": a.get("properties", [])})
            for c in doc.children(nid):
                walk(c, level + 1)

        for top in doc.children(pid):
            walk(top, 1)
        page_node = doc.node(pid) or {}
        for flow in page_node.get("flows") or []:
            if isinstance(flow, dict):
                start = flow.get("start") or flow.get("start_node") or flow.get("startNode")
                page["links"].append({"flow": flow.get("name", "flow"), "starts at": doc.path(start) if start is not None else None})
        out["pages"].append(page)

    by_page = {p["name"]: p for p in out["pages"]}
    for c in doc.data.get("comments") or []:
        if not isinstance(c, dict):
            continue
        anchor = c.get("anchor")
        thread = {"on": doc.path(anchor) if anchor is not None else f"page point ({c.get('x')}, {c.get('y')})",
                  "author": c.get("author", ""), "text": c.get("text", ""), "resolved": bool(c.get("resolved")),
                  "replies": [{"author": r.get("author", ""), "text": r.get("text", "")} for r in c.get("replies") or [] if isinstance(r, dict)]}
        page_name = doc.name(c.get("page")) if c.get("page") is not None else "?"
        by_page.setdefault(page_name, {"name": page_name, "outline": [], "texts": [], "links": [], "annotations": [], "comments": []})
        by_page[page_name]["comments"].append(thread)
        if by_page[page_name] not in out["pages"]:
            out["pages"].append(by_page[page_name])
    return out


def as_text(s):
    lines = [f"FigCraft file: {s.get('name') or '(unnamed)'} ({s.get('kind')})"]
    for p in s["pages"]:
        lines.append("")
        lines.append(f"== Page: {p['name']}")
        if p["outline"]:
            lines.append("  Frames:")
            for o in p["outline"]:
                status = f"  [{o['status']}]" if o.get("status") else ""
                lines.append(f"  {'  ' * o['level']}{o['type']}: {o['name']}{status}")
        if p["comments"]:
            lines.append("  Comments:")
            for c in p["comments"]:
                mark = "resolved" if c["resolved"] else "open"
                lines.append(f"    ({mark}) on {c['on']}: {c['author']}: {c['text']}")
                for r in c["replies"]:
                    lines.append(f"        ↳ {r['author']}: {r['text']}")
        if p["annotations"]:
            lines.append("  Annotations:")
            for a in p["annotations"]:
                cat = f" [{a['category']}]" if a.get("category") else ""
                props = f" (pins: {', '.join(a['properties'])})" if a.get("properties") else ""
                lines.append(f"    on {a['on']}{cat}: {a['label']}{props}")
        if p["links"]:
            lines.append("  Prototype links:")
            for link in p["links"]:
                if "flow" in link:
                    lines.append(f"    flow '{link['flow']}' starts at {link['starts at']}")
                else:
                    lines.append(f"    {link['from']}  ->  {link['to']}")
        if p["texts"]:
            lines.append("  Text:")
            for t in p["texts"]:
                one_line = " ".join(str(t["text"]).split())
                lines.append(f"    {t['where']}: {one_line}")
    return "\n".join(lines)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("file")
    ap.add_argument("--json", action="store_true", help="print the summary as JSON")
    ap.add_argument("--depth", type=int, default=3, help="how deep the frame outline goes (default 3)")
    a = ap.parse_args(argv)
    try:
        doc, others = read(a.file)
    except (OSError, ValueError, zipfile.BadZipFile, json.JSONDecodeError) as e:
        print(f"figcraft_read: {a.file}: {e}", file=sys.stderr)
        return 1
    s = summary(doc, a.depth)
    s["other files"] = len(others)
    print(json.dumps(s, indent=2, ensure_ascii=False) if a.json else as_text(s))
    return 0


if __name__ == "__main__":
    sys.exit(main())
