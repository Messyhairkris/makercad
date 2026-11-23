#!/usr/bin/env python3
"""A tiny language server (LSP over stdio) that underlines personal data in whatever file an
editor opens: the same scanner as the git hooks and CI (tools/personal_scan.py), shown as you
type instead of when you commit.

    python tools/personal_lsp.py

It answers initialize / shutdown / exit, listens to didOpen, didChange (full text), didSave and
didClose, and publishes one diagnostic per finding, source "makercad-personal-data". It never
echoes what it found: the message names the rule and the first character and length only.

Pointing an editor at it needs a generic language-client setting (for example a "custom
language server" entry for all file types running the command above). That wiring is the
editor's own, so it lives in your editor's settings, not in this repository.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

sys.path.insert(0, str(Path(__file__).resolve().parent))
import personal_scan as ps  # noqa: E402

SOURCE = "makercad-personal-data"


def read_message(stream):
    """One framed JSON-RPC message, or None at end of input."""
    length = None
    while True:
        line = stream.readline()
        if not line:
            return None
        line = line.strip()
        if not line:
            break
        key, _, value = line.decode("ascii", "replace").partition(":")
        if key.lower() == "content-length":
            length = int(value.strip())
    if length is None:
        return None
    body = stream.read(length)
    return json.loads(body.decode("utf-8"))


def write_message(stream, obj) -> None:
    body = json.dumps(obj, separators=(",", ":")).encode("utf-8")
    stream.write(b"Content-Length: " + str(len(body)).encode("ascii") + b"\r\n\r\n" + body)
    stream.flush()


def uri_to_path(uri: str) -> str:
    p = unquote(urlparse(uri).path)
    if len(p) > 2 and p[0] == "/" and p[2] == ":":  # file:///c:/dir/file
        p = p[1:]
    return p


def relative(path: str) -> str:
    try:
        return Path(path).resolve().relative_to(ps.REPO).as_posix()
    except (ValueError, OSError):
        return Path(path).name


def utf16(text: str, upto: int) -> int:
    return len(text[:upto].encode("utf-16-le")) // 2


def diagnostics(text: str, uri: str, terms) -> list[dict]:
    rel = relative(uri_to_path(uri))
    if ps.is_skipped(rel):
        return []
    lines = text.split("\n")
    out = []
    for f in ps.scan_text(text, rel, terms):
        line = lines[f.line - 1].rstrip("\r") if f.line - 1 < len(lines) else ""
        out.append(
            {
                "range": {
                    "start": {"line": f.line - 1, "character": utf16(line, f.col)},
                    "end": {"line": f.line - 1, "character": utf16(line, f.end)},
                },
                "severity": 1,
                "code": f.rule,
                "source": SOURCE,
                "message": f"{ps.HINTS[f.rule]} [{f.shown}]",
            }
        )
    return out


def serve(stdin, stdout) -> int:
    terms = ps.load_terms()
    shutdown = False

    def publish(uri: str, text: str) -> None:
        write_message(
            stdout,
            {
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {"uri": uri, "diagnostics": diagnostics(text, uri, terms)},
            },
        )

    while True:
        msg = read_message(stdin)
        if msg is None:
            return 0 if shutdown else 1
        method, mid, params = msg.get("method"), msg.get("id"), msg.get("params") or {}
        doc = params.get("textDocument") or {}
        if method == "initialize":
            write_message(
                stdout,
                {
                    "jsonrpc": "2.0",
                    "id": mid,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": {"openClose": True, "change": 1, "save": {"includeText": True}}
                        },
                        "serverInfo": {"name": SOURCE},
                    },
                },
            )
        elif method == "textDocument/didOpen":
            publish(doc.get("uri", ""), doc.get("text", ""))
        elif method == "textDocument/didChange":
            changes = params.get("contentChanges") or []
            if changes:
                publish(doc.get("uri", ""), changes[-1].get("text", ""))
        elif method == "textDocument/didSave":
            text = params.get("text")
            if text is None:
                try:
                    text = Path(uri_to_path(doc.get("uri", ""))).read_text(encoding="utf-8", errors="replace")
                except OSError:
                    text = ""
            publish(doc.get("uri", ""), text)
        elif method == "textDocument/didClose":
            write_message(
                stdout,
                {
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {"uri": doc.get("uri", ""), "diagnostics": []},
                },
            )
        elif method == "shutdown":
            shutdown = True
            write_message(stdout, {"jsonrpc": "2.0", "id": mid, "result": None})
        elif method == "exit":
            return 0 if shutdown else 1
        elif mid is not None:  # a request we do not know
            write_message(
                stdout, {"jsonrpc": "2.0", "id": mid, "error": {"code": -32601, "message": f"unknown method {method}"}}
            )


if __name__ == "__main__":
    sys.exit(serve(sys.stdin.buffer, sys.stdout.buffer))
