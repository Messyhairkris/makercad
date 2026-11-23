"""Tests for tools/personal_scan.py and tools/personal_lsp.py (AGENTS.md rule 11).

Every "personal" string here is invented and built from pieces at run time, so this file never
contains a whole path, address or key and the scanner can read it without complaint.
"""
from __future__ import annotations

import io
import json
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import personal_lsp as lsp  # noqa: E402
import personal_scan as ps  # noqa: E402

NONE = ps.build_terms()
ALICE = "Alice" + "Example"  # an invented account name
BS = "\\"


def home(name: str, drive: str = "C") -> str:
    return drive + ":" + BS + "Users" + BS + name


def rules(text: str, terms=NONE) -> list[str]:
    return [f.rule for f in ps.scan_text(text, "x.txt", terms)]


# ---- structural rules ---------------------------------------------------------------------


def test_a_home_folder_with_a_real_name_is_flagged():
    assert rules("see " + home("Zorbo") + BS + "x.exe") == ["home-path"]


def test_home_folder_placeholders_are_fine():
    for name in ("<you>", "%USERNAME%", "$env:USERNAME", "Public", "Default", "runneradmin", "you", "..."):
        assert rules("at " + home(name) + BS + "x") == [], name
    assert rules("/home/runner/work and /home/you/x and /Users/you/x") == []


def test_other_spellings_of_a_home_folder():
    esc = "C:" + BS * 2 + "Users" + BS * 2 + "Zorbo" + BS * 2 + "x"  # as written inside JSON
    assert rules(esc) == ["home-path"]
    assert rules("C:/" + "Users/Zorbo/x") == ["home-path"]
    assert rules("at /" + "Users/zorbo/x") == ["home-path"]
    assert rules("at /" + "home/zorbo/x") == ["home-path"]


def test_a_url_with_home_in_it_is_not_a_home_folder():
    assert rules("https://example.com/home/foo and https://api.example.com/users/zorbo") == []


def test_the_flag_names_only_the_name_not_the_rest_of_the_path():
    (f,) = ps.scan_text("x " + home("Zorbo") + BS + "deep", "p", NONE)
    assert (f.col, f.end) == (len("x " + home("")), len("x " + home("Zorbo")))


def test_machine_and_share_names():
    assert rules(BS * 2 + "BIGBOX" + BS + "projects") == ["unc-path"]
    assert rules(BS * 2 + "server" + BS + "share") == []
    assert rules("C:" + BS * 2 + "Users" + BS * 2 + "<you>") == []  # an escaped path is not a UNC path


def test_default_windows_computer_names():
    assert rules("on DESKTOP" + "-AB12CD3 today") == ["hostname"]
    assert rules("DESKTOP" + "-DEFAULT") == []  # no digit: an ordinary word


def test_email_addresses():
    assert rules("write to bob" + "@" + "mail.com") == ["email"]
    for ok in (
        "5360981+Someone@users.noreply.github.com",
        "you" + "@" + "example.com",
        "icon" + "@" + "2x.png",
        "git" + "@" + "github.com:o/r.git",
        "serde" + "@" + "1.0.2",
        "noreply" + "@" + "anthropic.com",
    ):
        assert rules(ok) == [], ok


def test_keys_and_tokens():
    assert rules("t = ghp" + "_" + "a1" * 20) == ["secret"]
    assert rules("k = sk" + "-" + "abcDEF123456" * 3) == ["secret"]
    assert rules("AKIA" + "ABCDEFGHIJKLMNOP") == ["secret"]
    assert rules('api_key = "' + "q9Zx3Lm7Vb2Nk8Rt5Yp1Wd4Hs6" + '"') == ["secret"]
    assert rules("-----BEGIN RSA PRIVATE" + " KEY-----") == ["secret"]
    assert rules('token = "' + "a" * 30 + '"') == []  # one repeated letter is a placeholder
    assert rules('api_key = "' + "your-key-goes-here-1234567" + '"') == []  # says so itself


def test_the_marker_skips_only_its_own_line():
    text = "a " + home("Zorbo") + "  personal-ok\nb " + home("Zorbo") + "\n"
    found = ps.scan_text(text, "p", NONE)
    assert [(f.rule, f.line) for f in found] == [("home-path", 2)]


# ---- private words ------------------------------------------------------------------------


def test_the_account_name_is_found_however_it_is_spelled():
    t = ps.build_terms(account=ALICE)
    for text in (ALICE, ALICE.lower(), "Alice Example", "alice_example", "ALICE.EXAMPLE"):
        assert rules("by " + text + " today", t) == ["personal-term"], text
    assert rules("by Alice today", t) == []  # a first name on its own is not the account
    assert rules("in ALICEE~1 temp", t) == ["personal-term"]  # the 8.3 short form


def test_a_personal_term_beats_the_path_rule_for_the_same_text():
    t = ps.build_terms(account=ALICE)
    (f,) = ps.scan_text(home(ALICE) + BS + "x", "p", t)
    assert f.rule == "personal-term"


def test_generic_accounts_and_runner_names_make_no_terms():
    assert ps.build_terms(account="runner", host="localhost").sources == {}
    assert ps.build_terms(account="Administrator").sources == {}


def test_a_noreply_email_makes_no_term_but_a_real_one_does():
    assert ps.build_terms(email="1+Someone@users.noreply.github.com").sources == {}
    t = ps.build_terms(email="someone.real" + "@" + "mail.com")
    assert t.sources == {"email": 2}
    assert rules("mail someone.real please", t) == ["personal-term"]


def test_extra_words_short_words_and_describe():
    t = ps.build_terms(extra=["Quillfeather", "ab"], file_terms=["Brindlewick"])
    assert t.sources == {"env": 1, "file": 1}
    assert "env: 1" in t.describe() and "Quillfeather" not in t.describe()
    assert ps.build_terms().describe().startswith("no private words")


def test_short_words_need_their_own_boundaries():
    t = ps.build_terms(extra=["Zork"])
    assert rules("Zork is here", t) == ["personal-term"]
    assert rules("Zorkmid is here", t) == []


def test_load_terms_ignores_the_runner_in_ci(monkeypatch):
    monkeypatch.setattr(ps, "private_term_files", lambda repo=ps.REPO: [])
    t = ps.load_terms(env={"CI": "true", "USERNAME": ALICE, "COMPUTERNAME": "BIGBOX-77"})
    assert t.sources == {}
    t = ps.load_terms(env={"CI": "true", "MAKERCAD_PERSONAL_TERMS": "Quillfeather, Brindlewick"})
    assert t.sources == {"env": 2}


def test_load_terms_reads_the_account_when_not_in_ci(monkeypatch):
    monkeypatch.setattr(ps, "private_term_files", lambda repo=ps.REPO: [])
    t = ps.load_terms(env={"USERNAME": ALICE})
    assert t.sources.get("account", 0) >= 1


def test_output_never_repeats_what_it_found():
    t = ps.build_terms(account=ALICE)
    token = "ghp" + "_" + "a1" * 20
    found = ps.scan_text("x " + ALICE + " " + token + " " + home("Zorbo"), "p", t)
    shown = " ".join(f.text() for f in found)
    for secret in (ALICE, token, "Zorbo"):
        assert secret not in shown
    assert len(found) == 3


# ---- files, binaries, git -----------------------------------------------------------------


def test_skipped_paths():
    assert ps.is_skipped("third_party/solidcraft/x.rs")
    assert ps.is_skipped("Cargo.lock") and ps.is_skipped("a/target/debug/x")
    assert not ps.is_skipped("tools/personal_scan.py")


def test_binary_files_are_searched_for_the_words_and_home_paths():
    t = ps.build_terms(account=ALICE)
    blob = b"\x89PNG\0\0" + ("meta " + home("Zorbo")).encode() + b"\0" + ALICE.encode("utf-16-le")
    got = sorted(f.rule for f in ps.scan_bytes(blob, "i.png", t))
    assert got == ["home-path", "personal-term"]
    assert all(f.line == 0 for f in ps.scan_bytes(blob, "i.png", t))
    assert ps.scan_bytes(b"\0\0clean\0", "i.png", t) == []


def test_utf16_text_is_decoded():
    data = ("x " + home("Zorbo")).encode("utf-16")
    assert [f.rule for f in ps.scan_bytes(data, "a.txt", NONE)] == ["home-path"]


def git(repo: Path, *args: str) -> str:
    r = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
    return r.stdout


@pytest.fixture
def repo(tmp_path, monkeypatch):
    git(tmp_path, "init", "-q")
    git(tmp_path, "config", "user.name", "Someone")
    git(tmp_path, "config", "user.email", "1+Someone@users.noreply.github.com")
    git(tmp_path, "config", "core.autocrlf", "false")
    monkeypatch.setattr(ps, "REPO", tmp_path)
    return tmp_path


def test_staged_flags_the_staged_copy_and_the_author_line(repo):
    (repo / "good.txt").write_text("nothing here\n")
    (repo / "bad.txt").write_text("path " + home("Zorbo") + "\n")
    git(repo, "add", "-A")
    got = ps.staged_findings(NONE)
    assert [(f.rule, f.path) for f in got] == [("home-path", "bad.txt")]
    git(repo, "config", "user.email", "me.real" + "@" + "mail.com")
    assert "email" in [f.rule for f in ps.staged_findings(NONE)]


def test_staged_ignores_a_file_fixed_after_staging(repo):
    p = repo / "bad.txt"
    p.write_text("path " + home("Zorbo") + "\n")
    git(repo, "add", "-A")
    p.write_text("path " + home("<you>") + "\n")  # fixed on disk, but the staged copy is what ships
    assert [f.rule for f in ps.staged_findings(NONE)] == ["home-path"]


def test_history_finds_what_a_later_commit_removed(repo):
    (repo / "a.txt").write_text("path " + home("Zorbo") + "\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "first")
    (repo / "a.txt").write_text("path " + home("<you>") + "\n")
    git(repo, "commit", "-q", "-am", "mail me at bob" + "@" + "mail.com")
    assert ps.tracked_findings(NONE) == []  # the tree is clean...
    got = ps.history_findings(NONE, ["HEAD"])
    assert sorted(f.rule for f in got) == ["email", "home-path"]  # ...the history is not
    assert any("first" not in f.path and ":a.txt" in f.path for f in got)


def test_history_in_a_range_and_a_bad_range(repo):
    (repo / "a.txt").write_text("fine\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "one")
    assert ps.history_findings(NONE, ["HEAD"]) == []
    with pytest.raises(SystemExit):
        ps.history_findings(NONE, ["no-such-revision..HEAD"])


def test_ci_falls_back_to_the_last_commits_when_the_old_head_is_gone(repo, monkeypatch):
    (repo / "a.txt").write_text("path " + home("Zorbo") + "\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "one")
    monkeypatch.setenv("PUSH_BEFORE_SHA", "1" * 40)  # not in this clone, as after a force-push
    monkeypatch.setenv("GITHUB_SHA", git(repo, "rev-parse", "HEAD").strip())
    assert "home-path" in [f.rule for f in ps.ci_findings(NONE)]
    monkeypatch.setenv("PUSH_BEFORE_SHA", "0" * 40)  # a new branch
    assert "home-path" in [f.rule for f in ps.ci_findings(NONE)]


def test_pre_push_reads_what_git_sends_on_stdin(repo):
    (repo / "a.txt").write_text("path " + home("Zorbo") + "\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "one")
    sha = git(repo, "rev-parse", "HEAD").strip()
    line = f"refs/heads/main {sha} refs/heads/main {'0' * 40}\n"
    assert [f.rule for f in ps.pre_push_findings(NONE, [line])] == ["home-path"]
    assert ps.pre_push_findings(NONE, [f"(delete) {'0' * 40} refs/heads/x {sha}\n"]) == []


# ---- Claude Code hook and the settings that run it ----------------------------------------


def test_hook_flags_a_file_inside_the_repository_only(repo, monkeypatch):
    inside = repo / "n.md"
    inside.write_text("see " + home("Zorbo") + "\n")
    rel, found = ps.hook_findings(NONE, {"tool_input": {"file_path": str(inside)}})
    assert rel == "n.md" and [f.rule for f in found] == ["home-path"]
    other = repo.parent / "outside.md"
    other.write_text("see " + home("Zorbo") + "\n")
    assert ps.hook_findings(NONE, {"tool_input": {"file_path": str(other)}}) == ("", [])
    assert ps.hook_findings(NONE, {"tool_input": {}}) == ("", [])


def test_hook_exit_codes(repo, monkeypatch, capsys):
    monkeypatch.setattr(ps, "load_terms", lambda *a, **k: NONE)
    f = repo / "n.md"
    f.write_text("see " + home("Zorbo") + "\n")
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps({"tool_input": {"file_path": str(f)}})))
    assert ps.main(["--hook"]) == 2  # 2 is what Claude Code feeds back to the model
    assert "home-path" in capsys.readouterr().err
    f.write_text("clean\n")
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps({"tool_input": {"file_path": str(f)}})))
    assert ps.main(["--hook"]) == 0
    monkeypatch.setattr(sys, "stdin", io.StringIO("not json"))
    assert ps.main(["--hook"]) == 0


def test_the_settings_file_runs_the_hook_after_a_write_or_edit():
    root = Path(__file__).resolve().parents[1]
    cfg = json.loads((root / ".claude" / "settings.json").read_text(encoding="utf-8"))
    (entry,) = cfg["hooks"]["PostToolUse"]
    assert {"Write", "Edit"} <= set(entry["matcher"].split("|"))
    cmd = entry["hooks"][0]["command"]
    assert "tools/personal_scan.py" in cmd and cmd.endswith("--hook")


def test_the_git_hooks_exist_and_use_unix_line_endings():
    hooks = Path(__file__).resolve().parent / "hooks"
    for name in ("pre-commit", "commit-msg", "pre-push"):
        data = (hooks / name).read_bytes()
        assert data.startswith(b"#!/bin/sh\n") and b"\r" not in data, name
        assert b"personal_scan.py" in data


# ---- the language server ------------------------------------------------------------------


def frame(obj) -> bytes:
    body = json.dumps(obj).encode()
    return b"Content-Length: " + str(len(body)).encode() + b"\r\n\r\n" + body


def talk(messages) -> tuple[list[dict], int]:
    script = Path(__file__).resolve().parent / "personal_lsp.py"
    p = subprocess.run(
        [sys.executable, "-I", str(script)], input=b"".join(frame(m) for m in messages), capture_output=True, timeout=60
    )
    out, stream = [], io.BytesIO(p.stdout)
    while (m := lsp.read_message(stream)) is not None:
        out.append(m)
    return out, p.returncode


def test_the_language_server_underlines_and_clears(tmp_path):
    uri = tmp_path.joinpath("n.md").as_uri()
    line = "x " + home("Zorbo") + BS + "y"
    msgs = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        {"jsonrpc": "2.0", "method": "initialized", "params": {}},
        {"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "text": "ok\n" + line + "\n"}}},
        {"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {"textDocument": {"uri": uri}, "contentChanges": [{"text": "ok\n"}]}},
        {"jsonrpc": "2.0", "method": "textDocument/didClose", "params": {"textDocument": {"uri": uri}}},
        {"jsonrpc": "2.0", "id": 7, "method": "frobnicate", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "shutdown"},
        {"jsonrpc": "2.0", "method": "exit"},
    ]
    out, code = talk(msgs)
    assert code == 0
    assert out[0]["id"] == 1 and out[0]["result"]["capabilities"]["textDocumentSync"]["change"] == 1
    diags = [m["params"]["diagnostics"] for m in out if m.get("method") == "textDocument/publishDiagnostics"]
    assert len(diags) == 3 and len(diags[0]) == 1 and diags[1] == [] and diags[2] == []
    d = diags[0][0]
    assert d["range"]["start"] == {"line": 1, "character": len("x " + home(""))}
    assert d["range"]["end"]["character"] - d["range"]["start"]["character"] == len("Zorbo")
    assert d["source"] == "makercad-personal-data" and d["severity"] == 1 and d["code"] == "home-path"
    assert "Zorbo" not in d["message"]
    assert any(m.get("id") == 7 and m["error"]["code"] == -32601 for m in out)
    assert any(m.get("id") == 2 and m["result"] is None for m in out)


def test_the_language_server_counts_columns_in_utf16(tmp_path):
    uri = tmp_path.joinpath("n.md").as_uri()
    text = "\U0001F600 " + home("Zorbo")  # an emoji is two UTF-16 units but one character
    (d,) = lsp.diagnostics(text, uri, NONE)
    assert d["range"]["start"]["character"] == 2 + 1 + len(home(""))


def test_exit_without_shutdown_is_an_error():
    out, code = talk([{"jsonrpc": "2.0", "method": "exit"}])
    assert code == 1
