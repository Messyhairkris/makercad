#!/usr/bin/env python3
"""Personal-data check (AGENTS.md rule 11: the repository is public).

One scanner, several triggers. Nothing personal is stored in this file or anywhere in the
repository: the words it looks for are read from the machine it runs on (the account name,
the computer name, the git email) and from two private places that are never committed.

    python tools/personal_scan.py FILE...        scan these files
    python tools/personal_scan.py --all          every tracked file
    python tools/personal_scan.py --staged       the staged copy of each staged file (pre-commit)
    python tools/personal_scan.py --commit-msg F a commit message file (commit-msg hook)
    python tools/personal_scan.py --pre-push     what a push would send (pre-push hook, stdin)
    python tools/personal_scan.py --history ARGS git rev-list arguments, e.g. origin/main..HEAD
    python tools/personal_scan.py --ci           the tree plus what this push or pull request adds
    python tools/personal_scan.py --hook         Claude Code PostToolUse hook (JSON on stdin)
    python tools/personal_scan.py --install      point git at tools/hooks (once per clone)
    python tools/personal_scan.py --terms        say which private words are active (lengths only)
    python tools/personal_scan.py --add-term W   remember a word for this PC only

What it looks for:
  personal-term  your account name, computer name, git email, anything on your private list
  home-path      a user folder with a real name in it (C:\\Users\\<name>, /Users/, /home/)
  unc-path       a machine or share name (\\\\machine\\share)
  hostname       a default Windows computer name
  email          an email address (the GitHub noreply address and example.* are fine)
  secret         something shaped like a key or token

A finding never prints what it found (CI logs are public): only the rule, the place, and the
first character and the length. A line that carries the marker  personal-ok  is skipped.
Exit status: 0 clean, 1 findings; --hook exits 2 on a finding, which Claude Code feeds back.
"""
from __future__ import annotations

import argparse
import bisect
import codecs
import fnmatch
import getpass
import json
import os
import re
import socket
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
MARKER = "personal-ok"
MAX_BYTES = 25 * 1024 * 1024
# Fetched or generated, never ours. Matched against the repo-relative path with "/" separators.
SKIP_GLOBS = ("third_party/*", "target/*", "*/target/*", "node_modules/*", "*.lock")

HINTS = {
    "personal-term": "your own account, computer or email, or a word on your private list",
    "home-path": "a user folder with a real name in it; write C:\\Users\\<you> or $env:USERPROFILE",
    "unc-path": "a machine or share name; say \"the share\" instead",
    "hostname": "a default Windows computer name",
    "email": "an email address; use the GitHub noreply address, or example.com",
    "secret": "shaped like a key or token",
}

# ---- what is fine -------------------------------------------------------------------------

PLACEHOLDER_NAMES = {
    "you", "user", "users", "username", "name", "yourname", "your-name", "me", "someone",
    "public", "default", "all users", "runneradmin", "runner", "example", "ubuntu", "vsts",
    "admin", "x", "...", "foo", "bar",
}
PLACEHOLDER_HOSTS = {
    "server", "host", "hostname", "computer", "machine", "pc", "name", "yourpc", "example",
    "localhost", "wsl$", "wsl.localhost", "share",
}
ALLOWED_EMAILS = {"noreply@anthropic.com", "noreply@github.com", "git@github.com"}
ALLOWED_EMAIL_ENDINGS = (
    "users.noreply.github.com", "example.com", "example.org", "example.net", "example.edu",
    ".example", ".test", ".invalid", ".localhost",
    # "icon@2x.png" is a file name, not an address
    ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico", ".css", ".js", ".json", ".html",
)
GENERIC_ACCOUNTS = {
    "root", "admin", "administrator", "user", "ubuntu", "runner", "runneradmin", "vsts",
    "azureuser", "vscode", "node", "docker", "default", "guest", "owner", "me",
}

# ---- structural rules (they need to know nothing about the person) ------------------------

_SEP = r"(?:\\{1,4}|/)"
_NAME = r"(?P<n>[^\\/\s\"'`<>|:*?%$,;(){}\[\]&=]+)"
HOME_PATTERNS = (
    re.compile(r"(?<![A-Za-z0-9])[A-Za-z]:" + _SEP + r"(?i:Users)" + _SEP + _NAME),
    re.compile(r"(?<![A-Za-z0-9:])\\{1,4}(?i:Users)\\{1,4}" + _NAME),
    re.compile(r"(?<![\w/.~-])/Users/" + _NAME),
    re.compile(r"(?<![\w/.~-])/home/" + _NAME),
)
UNC = re.compile(r"(?<![\w:\\.$])\\\\(?P<host>[A-Za-z][\w.-]{2,})\\(?P<share>[\w$.-]+)")
EMAIL = re.compile(
    r"(?<![\w.+-])(?P<local>[A-Za-z0-9._%+-]+)@(?P<domain>(?:[A-Za-z0-9-]+\.)+[A-Za-z]{2,})(?![\w-])"
)
HOSTNAME = re.compile(
    r"(?<![A-Za-z0-9-])(?:DESKTOP|LAPTOP|WIN)-(?=[A-Z0-9]*\d)[A-Z0-9]{6,8}(?![A-Za-z0-9-])"
)
SECRETS = (
    re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{20,})"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}"),
    re.compile(r"\bxox[abprs]-[A-Za-z0-9-]{10,}"),
    re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b"),
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(
        r"""(?i)\b(?:api[_-]?key|secret|token|passw(?:or)?d)\b["']?\s*[:=]\s*["'](?P<v>[A-Za-z0-9+/_=-]{24,})["']"""
    ),
)


@dataclass(frozen=True)
class Finding:
    rule: str
    path: str
    line: int  # 1-based; 0 for a position in a binary file
    col: int  # 0-based start (a byte offset in a binary file)
    end: int
    shown: str

    def where(self) -> str:
        if self.line:
            return f"{self.path}:{self.line}:{self.col + 1}"
        return f"{self.path}@{self.col}"

    def text(self) -> str:
        return f"{self.where()}: {self.rule}: {HINTS[self.rule]} [{self.shown}]"


def redact(s: str) -> str:
    return "*" * len(s) if len(s) <= 2 else f"{s[0]}***({len(s)})"


# ---- the private words --------------------------------------------------------------------


@dataclass
class Terms:
    patterns: list  # compiled, case-insensitive
    binary: list  # compiled bytes patterns (utf-8 and utf-16)
    sources: dict  # source label -> how many words

    def describe(self) -> str:
        if not self.sources:
            return "no private words are active (only the structural rules run)"
        return "private words active: " + ", ".join(f"{k}: {v}" for k, v in self.sources.items())


def _term_regex(term: str) -> str:
    camel = re.sub(r"(?<=[a-z])(?=[A-Z])", " ", term).split()
    body = r"[\s._-]*".join(re.escape(p) for p in camel) if len(camel) > 1 else re.escape(term)
    if len(term) <= 6:
        body = r"(?<![A-Za-z0-9])" + body + r"(?![A-Za-z0-9])"
    return body


def build_terms(account=None, host=None, email=None, extra=(), file_terms=()) -> Terms:
    """Pure: turn what is known about this PC into patterns. Words under 4 letters are ignored."""
    found: dict[str, list[str]] = {}

    def add(label: str, word: str | None) -> None:
        if word and len(word) >= 4 and word.lower() not in GENERIC_ACCOUNTS:
            found.setdefault(label, [])
            if word not in found[label]:
                found[label].append(word)

    if account and account.lower() not in GENERIC_ACCOUNTS:
        add("account", account)
        add("account", re.sub(r"[\s._-]", "", account))
        if len(account) > 8 and " " not in account:
            add("account", account[:6].upper() + "~1")  # the 8.3 short form in temp paths
    if host and host.lower() not in {"localhost", "localhost.localdomain"} and len(host) >= 6:
        add("computer", host)
        add("computer", host.split(".")[0])
    if email and not email.lower().endswith(ALLOWED_EMAIL_ENDINGS) and email.lower() not in ALLOWED_EMAILS:
        add("email", email)
        local = email.split("@")[0]
        if len(local) >= 5:
            add("email", local)
    for w in extra:
        add("env", w.strip())
    for w in file_terms:
        add("file", w.strip())

    patterns, binary = [], []
    for words in found.values():
        for w in words:
            patterns.append(re.compile(_term_regex(w), re.I))
            for enc in ("utf-8", "utf-16-le"):
                binary.append(re.compile(re.escape(w.encode(enc)), re.I))
    return Terms(patterns, binary, {k: len(v) for k, v in found.items()})


def private_term_files(repo: Path = REPO) -> list[Path]:
    return [Path.home() / ".makercad-private-terms", repo / ".git" / "personal-terms.txt"]


def _read_terms(path: Path) -> list[str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return []
    return [l.strip() for l in lines if l.strip() and not l.lstrip().startswith("#")]


def load_terms(repo: Path = REPO, env=None) -> Terms:
    env = os.environ if env is None else env
    in_ci = bool(env.get("CI") or env.get("GITHUB_ACTIONS"))
    account = host = email = None
    if not in_ci:  # a runner's own names mean nothing
        account = env.get("USERNAME") or env.get("USER")
        if not account:
            try:
                account = getpass.getuser()
            except Exception:
                account = None
        host = env.get("COMPUTERNAME") or socket.gethostname()
        r = _git(["config", "user.email"], repo)
        email = r.stdout.decode("utf-8", "replace").strip() or None
    extra = [w for w in re.split(r"[,;\n]", env.get("MAKERCAD_PERSONAL_TERMS", "")) if w.strip()]
    file_terms = [w for f in private_term_files(repo) for w in _read_terms(f)]
    return build_terms(account, host, email, extra, file_terms)


# ---- scanning -----------------------------------------------------------------------------


def is_skipped(rel: str) -> bool:
    rel = rel.replace("\\", "/")
    return any(fnmatch.fnmatch(rel, g) for g in SKIP_GLOBS)


def _structural(text: str):
    """Yield (rule, start, end, shown) for the rules that need no private words."""
    for pat in HOME_PATTERNS:
        for m in pat.finditer(text):
            n = m.group("n").rstrip(".")
            if len(n) > 1 and n.lower() not in PLACEHOLDER_NAMES and not set(n) <= {"."}:
                yield "home-path", m.start("n"), m.start("n") + len(n), redact(n)
    for m in UNC.finditer(text):
        if m.group("host").lower() not in PLACEHOLDER_HOSTS:
            yield "unc-path", m.start("host"), m.end("share"), redact(m.group("host"))
    for m in EMAIL.finditer(text):
        addr = m.group(0).lower()
        if addr not in ALLOWED_EMAILS and not m.group("domain").lower().endswith(ALLOWED_EMAIL_ENDINGS):
            yield "email", m.start(), m.end(), redact(m.group("local"))
    for m in HOSTNAME.finditer(text):
        yield "hostname", m.start(), m.end(), redact(m.group(0))
    for pat in SECRETS:
        for m in pat.finditer(text):
            span = m.span("v") if "v" in pat.groupindex else m.span()
            v = m.group("v") if "v" in pat.groupindex else m.group(0)
            if len(set(v)) > 4 and not re.search(r"(?i)example|placeholder|your|xxxx", v):
                yield "secret", span[0], span[1], redact(v)


def scan_text(text: str, path: str, terms: Terms) -> list[Finding]:
    starts = [0] + [m.end() for m in re.finditer(r"\n", text)]
    skipped_lines = {
        bisect.bisect_right(starts, m.start())
        for m in re.finditer(re.escape(MARKER), text)
    }
    raw = [(r, s, e, sh) for r, s, e, sh in _structural(text)]
    for pat in terms.patterns:
        for m in pat.finditer(text):
            raw.append(("personal-term", m.start(), m.end(), redact(m.group(0))))
    out: list[Finding] = []
    taken: list[tuple[int, int]] = []
    for rule, s, e, sh in sorted(raw, key=lambda t: (t[0] != "personal-term", t[1])):
        line = bisect.bisect_right(starts, s)
        if line in skipped_lines or any(s < b and a < e for a, b in taken):
            continue
        taken.append((s, e))
        out.append(Finding(rule, path, line, s - starts[line - 1], e - starts[line - 1], sh))
    return sorted(out, key=lambda f: (f.line, f.col))


def scan_bytes(data: bytes, path: str, terms: Terms) -> list[Finding]:
    if is_skipped(path) or len(data) > MAX_BYTES:
        return []
    head = data[:8192]
    if data.startswith((codecs.BOM_UTF16_LE, codecs.BOM_UTF16_BE)):
        return scan_text(data.decode("utf-16", errors="replace"), path, terms)
    if b"\0" not in head:
        return scan_text(data.decode("utf-8", errors="replace"), path, terms)
    # Binary (an image, an archive): look for the private words and for home paths only.
    out = []
    for pat in terms.binary:
        for m in pat.finditer(data):
            out.append(Finding("personal-term", path, 0, m.start(), m.end(), redact(m.group(0).decode("latin-1"))))
    latin = data.decode("latin-1")
    for pat in HOME_PATTERNS:
        for m in pat.finditer(latin):
            n = m.group("n").rstrip(".")
            if len(n) > 1 and n.lower() not in PLACEHOLDER_NAMES and not set(n) <= {"."}:
                out.append(Finding("home-path", path, 0, m.start("n"), m.end("n"), redact(n)))
    return out[:20]


def scan_file(path: Path, rel: str, terms: Terms) -> list[Finding]:
    try:
        return scan_bytes(path.read_bytes(), rel, terms)
    except OSError:
        return []


# ---- git ----------------------------------------------------------------------------------


def _git(args, cwd: Path | None = None, data: bytes | None = None) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(["git", *args], cwd=cwd or REPO, input=data, capture_output=True)
    except OSError as e:  # git is not installed
        return subprocess.CompletedProcess(args, 127, b"", str(e).encode())


def _paths(out: bytes) -> list[str]:
    return [p for p in out.decode("utf-8", "replace").split("\0") if p]


def tracked_findings(terms: Terms) -> list[Finding]:
    out: list[Finding] = []
    for rel in _paths(_git(["ls-files", "-z"]).stdout):
        if not is_skipped(rel):
            out += scan_file(REPO / rel, rel, terms)
    return out


def identity_findings(terms: Terms) -> list[Finding]:
    r = _git(["var", "GIT_AUTHOR_IDENT"])
    if r.returncode:
        return []
    ident = r.stdout.decode("utf-8", "replace").strip()
    return scan_text(ident, "the commit's author line", terms)


def staged_findings(terms: Terms) -> list[Finding]:
    out = identity_findings(terms)
    names = _git(["diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z"]).stdout
    for rel in _paths(names):
        if is_skipped(rel):
            continue
        r = _git(["show", f":{rel}"])
        if r.returncode == 0:
            out += scan_bytes(r.stdout, rel, terms)
    return out


def history_findings(terms: Terms, rev_args: list[str]) -> list[Finding]:
    """Messages, author lines and every file version the given commits add or change."""
    r = _git(["rev-list", "--reverse", *rev_args])
    if r.returncode:
        raise SystemExit("personal-data check: git could not read that range: " + r.stderr.decode("utf-8", "replace").strip())
    out: list[Finding] = []
    seen_blobs: set[str] = set()
    for sha in r.stdout.decode().split():
        label = sha[:7]
        meta = _git(["show", "-s", "--format=%an%n%ae%n%cn%n%ce%n%B", sha]).stdout
        out += scan_text(meta.decode("utf-8", "replace"), f"commit {label} (message and authors)", terms)
        raw = _git(["diff-tree", "--root", "-r", "-z", "--no-commit-id", "--diff-filter=ACMR", "--raw", sha]).stdout
        toks = raw.decode("utf-8", "replace").split("\0")
        i = 0
        while i + 1 < len(toks):
            head, rel = toks[i], toks[i + 1]
            i += 2
            if not head.startswith(":"):
                continue
            blob = head.split()[3]
            if blob in seen_blobs or is_skipped(rel):
                continue
            seen_blobs.add(blob)
            data = _git(["cat-file", "blob", blob]).stdout
            out += scan_bytes(data, f"{label}:{rel}", terms)
    return out


def ci_findings(terms: Terms) -> list[Finding]:
    env = os.environ
    head = env.get("GITHUB_SHA") or "HEAD"
    base = env.get("PR_BASE_SHA", "")
    before = env.get("PUSH_BEFORE_SHA", "")
    if base:
        args = [f"{base}..{head}"]
    elif before and set(before) != {"0"}:
        args = [f"{before}..{head}"]
    else:
        args = ["-n", "50", head]
    out = tracked_findings(terms)
    if _git(["rev-list", "--count", *args]).returncode:  # e.g. the old head after a force-push
        args = ["-n", "50", head]
    return out + history_findings(terms, args)


def pre_push_findings(terms: Terms, lines) -> list[Finding]:
    out: list[Finding] = []
    for line in lines:
        parts = line.split()
        if len(parts) < 4:
            continue
        _, local, _, remote = parts[:4]
        if set(local) == {"0"}:
            continue  # deleting a branch
        args = [local, "--not", "--remotes"]
        if set(remote) != {"0"} and not _git(["rev-list", "--count", f"{remote}..{local}"]).returncode:
            args = [f"{remote}..{local}"]
        out += history_findings(terms, args)
    return out


def hook_findings(terms: Terms, payload: dict) -> tuple[str, list[Finding]]:
    ti = payload.get("tool_input") or {}
    fp = ti.get("file_path") or ti.get("path")
    if not fp:
        return "", []
    p = Path(fp)
    if not p.is_absolute():
        p = Path(payload.get("cwd") or os.getcwd()) / p
    try:
        rel = p.resolve().relative_to(REPO).as_posix()
    except ValueError:
        return "", []  # outside this repository
    if is_skipped(rel) or not p.is_file():
        return rel, []
    return rel, scan_file(p, rel, terms)


# ---- reporting and entry point ------------------------------------------------------------


def report(findings: list[Finding], what: str) -> int:
    if not findings:
        return 0
    print(f"personal-data check: {len(findings)} finding(s) in {what} (AGENTS.md rule 11)", file=sys.stderr)
    for f in findings[:40]:
        print("  " + f.text(), file=sys.stderr)
    if len(findings) > 40:
        print(f"  ... and {len(findings) - 40} more", file=sys.stderr)
    print(
        "Fix the text (placeholders: C:\\Users\\<you>, you@example.com), or mark a line that is "
        f"fine with the word {MARKER}.",
        file=sys.stderr,
    )
    return len(findings)


def install() -> int:
    r = _git(["config", "core.hooksPath", "tools/hooks"])
    if r.returncode:
        print("could not set core.hooksPath: " + r.stderr.decode("utf-8", "replace"), file=sys.stderr)
        return 1
    hooks = REPO / "tools" / "hooks"
    for h in hooks.iterdir() if hooks.is_dir() else []:
        try:
            h.chmod(h.stat().st_mode | 0o111)
        except OSError:
            pass
    print("git now runs tools/hooks (pre-commit, commit-msg, pre-push) in this clone.")
    return 0


def add_term(word: str) -> int:
    if len(word.strip()) < 4:
        print("a private word needs at least 4 characters", file=sys.stderr)
        return 1
    f = private_term_files()[0]
    with f.open("a", encoding="utf-8") as fh:
        fh.write(word.strip() + "\n")
    print(f"remembered a word of {len(word.strip())} characters for this PC (kept outside the repository)")
    return 0


def main(argv: list[str] | None = None) -> int:
    for s in (sys.stdout, sys.stderr):
        try:
            s.reconfigure(errors="replace")
        except Exception:
            pass
    ap = argparse.ArgumentParser(description="Personal-data check (AGENTS.md rule 11).")
    ap.add_argument("files", nargs="*")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--staged", action="store_true")
    ap.add_argument("--commit-msg", metavar="FILE")
    ap.add_argument("--pre-push", action="store_true")
    ap.add_argument("--history", nargs=argparse.REMAINDER, metavar="REV")
    ap.add_argument("--ci", action="store_true")
    ap.add_argument("--hook", action="store_true")
    ap.add_argument("--install", action="store_true")
    ap.add_argument("--terms", action="store_true")
    ap.add_argument("--add-term", metavar="WORD")
    a = ap.parse_args(argv)

    if a.install:
        return install()
    if a.add_term:
        return add_term(a.add_term)
    terms = load_terms()
    if a.terms:
        print(terms.describe())
        return 0
    if a.hook:
        try:
            payload = json.load(sys.stdin)
        except ValueError:
            return 0
        rel, found = hook_findings(terms, payload)
        return 2 if report(found, rel or "the file just written") else 0
    if a.staged:
        return 1 if report(staged_findings(terms), "what you are about to commit") else 0
    if a.commit_msg:
        text = Path(a.commit_msg).read_text(encoding="utf-8", errors="replace")
        return 1 if report(scan_text(text, "the commit message", terms), "the commit message") else 0
    if a.pre_push:
        return 1 if report(pre_push_findings(terms, sys.stdin), "what you are about to push") else 0
    if a.history:
        return 1 if report(history_findings(terms, a.history), "that range of commits") else 0
    if a.ci:
        return 1 if report(ci_findings(terms), "this repository") else 0
    if a.all:
        return 1 if report(tracked_findings(terms), "the tracked files") else 0
    if a.files:
        found = []
        for f in a.files:
            p = Path(f)
            try:
                rel = p.resolve().relative_to(REPO).as_posix()
            except ValueError:
                rel = p.as_posix()
            found += scan_file(p, rel, terms)
        return 1 if report(found, "those files") else 0
    ap.print_help()
    return 0


if __name__ == "__main__":
    sys.exit(main())
