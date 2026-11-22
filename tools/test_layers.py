"""Tests for the layers gate's rules (tools/layers.py), on made-up crates."""
from layers import SHARED, Crate, Dep, check, members, read_crate, wasm_packages

TABLE = {"base": 0, "domain": 1, "project": 3, "other-project": 3, "app": 5, "probe": SHARED}


def crate(name, *deps, kind="normal"):
    return Crate(name, tuple(Dep(d, kind) for d in deps))


def names(problems):
    return [p.split(" ")[0] for p in problems]


def test_downward_dependencies_pass():
    crates = [crate("mk-app", "mk-project", "mk-base"), crate("mk-project", "mk-base")]
    table = {"mk-app": 5, "mk-project": 3, "mk-base": 0}
    assert check(crates, table, set()) == []


def test_an_upward_dependency_fails():
    problems = check([crate("mk-base", "mk-project")], {"mk-base": 0, "mk-project": 3}, set())
    assert len(problems) == 1 and "upward" in problems[0]


def test_same_layer_only_when_allowed():
    table = {"mk-a": 3, "mk-b": 3}
    assert len(check([crate("mk-a", "mk-b")], table, set())) == 1
    assert check([crate("mk-a", "mk-b")], table, {("mk-a", "mk-b")}) == []


def test_every_member_must_be_in_the_table():
    problems = check([crate("mk-new")], {}, set())
    assert problems == ["mk-new: not in the layers TABLE (tools/layers.py)"]


def test_a_dependency_missing_from_the_table_fails():
    problems = check([crate("mk-a", "solidcraft-mystery")], {"mk-a": 3}, set())
    assert "not in the layers TABLE" in problems[0]


def test_shared_crates_use_only_shared_and_external_crates():
    table = {"mk-probe": SHARED, "mk-view": SHARED, "mk-project": 3}
    assert check([crate("mk-probe", "mk-view", "serde")], table, set()) == []
    assert len(check([crate("mk-probe", "mk-project")], table, set())) == 1
    # Even in tests: a shared crate must build on its own.
    assert len(check([crate("mk-probe", "mk-project", kind="dev")], table, set())) == 1


def test_any_layer_may_use_a_shared_crate():
    table = {"mk-base": 0, "mk-probe": SHARED}
    assert check([crate("mk-base", "mk-probe")], table, set()) == []


def test_ui_toolkits_only_in_front_ends():
    table = {"mk-project": 3, "mk-app": 5}
    assert len(check([crate("mk-project", "egui")], table, set())) == 1
    assert len(check([crate("mk-project", "wgpu")], table, set())) == 1
    assert check([crate("mk-app", "egui", "eframe", "wgpu")], table, set()) == []


def test_dev_dependencies_of_layered_crates_are_not_checked():
    table = {"mk-base": 0, "mk-project": 3}
    assert check([crate("mk-base", "mk-project", kind="dev")], table, set()) == []


def test_a_renamed_dependency_is_read_by_its_package(tmp_path):
    (tmp_path / "Cargo.toml").write_text(
        '[package]\nname = "mk-x"\n[dependencies]\nunits = { package = "mk-units", path = "../u" }\n'
        '[dev-dependencies]\nmk-y = { path = "../y" }\n',
        encoding="utf-8",
    )
    c = read_crate(tmp_path / "Cargo.toml")
    assert c.name == "mk-x"
    assert [(d.name, d.kind) for d in c.deps] == [("mk-units", "normal"), ("mk-y", "dev")]


def test_the_wasm_gate_takes_every_crate_below_the_front_ends():
    table = {"mk-base": 0, "mk-project": 3, "mk-probe": SHARED, "mk-app": 5}
    crates = [crate(n) for n in ("mk-app", "mk-base", "mk-probe", "mk-project")]
    assert wasm_packages(crates, table) == ["mk-base", "mk-probe", "mk-project"]


def test_the_real_workspace_passes():
    crates = [read_crate(m) for m in members()]
    assert check(crates) == [], "run python tools/layers.py to see why"
