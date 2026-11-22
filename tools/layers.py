"""The layers gate (doc 03 sections 3.1 and 3.10): dependencies point down, never up.

    python tools/layers.py                  # exit 1, one line per violation, when a rule is broken
    python tools/layers.py --wasm-packages  # the crates the wasm gate checks, one per line

SolidCraft enforces its own layers with `cargo xtask layers`; this is the same rule for our
crates, read from each workspace member's Cargo.toml (no build, no `cargo metadata`):

1. every workspace member is in TABLE;
2. a dependency on one of our or SolidCraft's crates goes to a lower layer (the same layer only
   when INTRA_ALLOWED lists the pair);
3. a SHARED crate depends only on shared crates and external crates;
4. nothing below the front ends (L5) depends on a UI toolkit (egui, eframe, wgpu, winit).

Normal and build dependencies are checked; dev-dependencies (tests) are not, except rule 3.
"""
from __future__ import annotations

import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SHARED = "shared"

# doc 03 section 3.1. Ours, then the SolidCraft crates ours may use (theirs as SolidCraft's TABLE).
TABLE: dict[str, int | str] = {
    # L0 base
    "mk-units": 0,
    "solidcraft-geom": 0,
    "solidcraft-color": 0,
    "solidcraft-space-mouse": 0,
    # L1 domain
    "mk-nest": 1,
    "solidcraft-kernel": 1,
    "solidcraft-sketch": 1,
    "solidcore-record": 1,
    "solidcore-ops": 1,
    "solidcore-tess": 1,
    # L2 document
    "solidcraft-doc": 2,
    # L3 project
    "mk-project": 3,
    "mk-inventory": 3,
    "mk-log": 3,
    "mk-assist": 3,
    "mk-sancho": 3,
    "solidcraft-io": 3,
    # L4 engine
    "solidcraft-engine": 4,
    # L5 front ends
    "mk-desktop": 5,
    "solidcraft-gpu": 5,
    # shared: usable from any layer; they use only shared and external crates
    "mk-probe": SHARED,
    "solidcraft-viewport": SHARED,
    "solidcraft-viewport-gpu": SHARED,
    "solidcraft-viewport-egui": SHARED,
    "solidcraft-control": SHARED,
    "solidcraft-control-egui": SHARED,
    "solidcraft-ui-registry": SHARED,
    "solidcraft-ui-text": SHARED,
}

# Same-layer dependencies that are allowed: (from, to).
INTRA_ALLOWED: set[tuple[str, str]] = {
    # The window draws with SolidCraft's GPU crate, an L5 library its own apps use the same way.
    ("mk-desktop", "solidcraft-gpu"),
}

UI_TOOLKITS = {"egui", "eframe", "wgpu", "winit", "egui-wgpu", "egui-winit", "egui_extras"}
FRONT_END = 5


@dataclass(frozen=True)
class Dep:
    name: str
    kind: str  # "normal", "build" or "dev"


@dataclass(frozen=True)
class Crate:
    name: str
    deps: tuple[Dep, ...]


def is_ours(name: str) -> bool:
    return name.startswith(("mk-", "solidcraft-", "solidcore-"))


def check(crates: list[Crate], table: dict[str, int | str] = TABLE,
          intra: set[tuple[str, str]] = INTRA_ALLOWED) -> list[str]:
    out: list[str] = []
    for crate in crates:
        cls = table.get(crate.name)
        if cls is None:
            out.append(f"{crate.name}: not in the layers TABLE (tools/layers.py)")
            continue
        for dep in crate.deps:
            if dep.name == crate.name:
                continue
            if cls == SHARED:
                if is_ours(dep.name) and table.get(dep.name) != SHARED:
                    out.append(f"{crate.name} (shared) depends on {dep.name}, which is not shared")
                continue
            if dep.kind == "dev":
                continue
            if dep.name in UI_TOOLKITS and cls < FRONT_END:
                out.append(f"{crate.name} (L{cls}) depends on the UI toolkit {dep.name}")
                continue
            if not is_ours(dep.name):
                continue
            to = table.get(dep.name)
            if to is None:
                out.append(f"{crate.name} depends on {dep.name}, which is not in the layers TABLE")
            elif to == SHARED:
                continue
            elif to > cls or (to == cls and (crate.name, dep.name) not in intra):
                out.append(f"{crate.name} (L{cls}) depends upward or sideways on {dep.name} (L{to})")
    return out


def read_crate(manifest: Path) -> Crate:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    deps: list[Dep] = []
    for section, kind in (("dependencies", "normal"), ("build-dependencies", "build"),
                          ("dev-dependencies", "dev")):
        for key, spec in data.get(section, {}).items():
            name = spec.get("package", key) if isinstance(spec, dict) else key
            deps.append(Dep(name, kind))
    return Crate(data["package"]["name"], tuple(deps))


def members(root: Path = ROOT) -> list[Path]:
    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]
    return [root / m / "Cargo.toml" for m in workspace["members"]]


def wasm_packages(crates: list[Crate], table: dict[str, int | str] = TABLE) -> list[str]:
    """The workspace members below the front ends: every one must build for wasm32 (doc 03)."""
    return [c.name for c in crates
            if table.get(c.name) == SHARED or (isinstance(table.get(c.name), int)
                                               and table[c.name] < FRONT_END)]


def main() -> int:
    crates = [read_crate(m) for m in members()]
    if sys.argv[1:] == ["--wasm-packages"]:
        # One per line, for: cargo check --target wasm32-unknown-unknown -p ... (the wasm gate).
        print("\n".join(wasm_packages(crates)))
        return 0
    problems = check(crates)
    for p in problems:
        print(f"layers: {p}")
    if not problems:
        print(f"layers: {len(crates)} crates, every dependency points down")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
