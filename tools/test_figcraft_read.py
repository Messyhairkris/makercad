"""Tests for figcraft_read.py on a small file built to FigCraft's document schema (crates/doc)."""

import json
import os
import sys
import zipfile

sys.path.insert(0, os.path.dirname(__file__))

import figcraft_read  # noqa: E402


def make(tmp_path, document, extra=None):
    path = tmp_path / "notes.figcraft"
    with zipfile.ZipFile(path, "w") as z:
        z.writestr("document.json", json.dumps(document))
        for name, data in (extra or {}).items():
            z.writestr(name, data)
    return str(path)


NOTES = {
    "name": "MakerCAD UI notes",
    "pages": [1],
    "nodes": {
        "1": {"id": 1, "name": "Design workspace", "type": "page", "children": [2, 5], "width": 0, "height": 0,
              "flows": [{"name": "Make a part", "start": 2}]},
        "2": {"id": 2, "name": "01 Design bench", "type": "frame", "parent": 1, "children": [3, 4], "width": 1440, "height": 900,
              "interactions": [{"trigger": {"type": "onClick"}, "actions": [{"type": "navigate", "destination": 5}]}],
              "dev_status": "readyForDev"},
        "3": {"id": 3, "name": "Toolbox note", "type": "text", "parent": 2, "width": 200, "height": 40,
              "characters": "Icons only after the first week", "style": {}},
        "4": {"id": 4, "name": "Toolbox", "type": "frame", "parent": 2, "width": 104, "height": 600,
              "annotations": [{"label": "Keep the words under icons", "category": "decision"}]},
        "5": {"id": 5, "name": "02 Sketch on a face", "type": "frame", "parent": 1, "width": 1440, "height": 900},
    },
    "comments": [
        {"id": 7, "page": 1, "x": 10, "y": 20, "anchor": 4, "author": "Kris", "text": "Toolbox: icons with words", "created": 0,
         "resolved": False, "replies": [{"author": "Mookie", "text": "Agree", "created": 0}]},
        {"id": 8, "page": 1, "x": 900, "y": 40, "author": "Kris", "text": "Timeline at the bottom is good", "resolved": True},
    ],
}


def test_it_reads_frames_text_comments_annotations_and_links(tmp_path):
    doc, others = figcraft_read.read(make(tmp_path, NOTES, {"images/abc.png": b"\x89PNG"}))
    assert others == ["images/abc.png"]
    s = figcraft_read.summary(doc)
    page = s["pages"][0]
    assert page["name"] == "Design workspace"
    assert [o["name"] for o in page["outline"]] == ["01 Design bench", "Toolbox", "02 Sketch on a face"]
    assert page["outline"][0]["status"] == "readyForDev"
    assert page["texts"] == [{"where": "01 Design bench / Toolbox note", "text": "Icons only after the first week"}]
    assert {"from": "01 Design bench", "to": "02 Sketch on a face"} in page["links"]
    assert {"flow": "Make a part", "starts at": "01 Design bench"} in page["links"]
    assert page["annotations"][0]["label"] == "Keep the words under icons"
    assert page["annotations"][0]["on"] == "01 Design bench / Toolbox"
    open_thread = page["comments"][0]
    assert open_thread["on"] == "01 Design bench / Toolbox" and not open_thread["resolved"]
    assert open_thread["replies"] == [{"author": "Mookie", "text": "Agree"}]
    assert page["comments"][1]["resolved"] and page["comments"][1]["on"].startswith("page point")
    text = figcraft_read.as_text(s)
    assert "(open) on 01 Design bench / Toolbox: Kris: Toolbox: icons with words" in text
    assert "01 Design bench  ->  02 Sketch on a face" in text


def test_unknown_and_missing_fields_are_skipped(tmp_path):
    odd = {"pages": ["p"], "nodes": {"p": {"id": "p", "type": "page", "children": ["x", "missing"]},
                                     "x": {"id": "x", "type": "somethingNew", "parent": "p", "mystery": {"a": 1}}},
           "comments": ["not a thread", {"page": "p"}]}
    s = figcraft_read.summary(figcraft_read.read(make(tmp_path, odd))[0])
    assert s["pages"][0]["name"] == "#p"
    assert s["pages"][0]["comments"][0]["text"] == ""


def test_a_broken_file_is_an_error_not_a_crash(tmp_path, capsys):
    bad = tmp_path / "bad.figcraft"
    bad.write_bytes(b"not a zip")
    assert figcraft_read.main([str(bad)]) == 1
    empty = tmp_path / "empty.figcraft"
    with zipfile.ZipFile(empty, "w") as z:
        z.writestr("images/a.png", b"x")
    assert figcraft_read.main([str(empty)]) == 1
    garbled = make(tmp_path, {})
    with zipfile.ZipFile(garbled, "w") as z:
        z.writestr("document.json", "{not json")
    assert figcraft_read.main([garbled]) == 1
    assert "figcraft_read:" in capsys.readouterr().err
