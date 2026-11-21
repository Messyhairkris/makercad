//! The icon SVGs the app ships, by id. Written by `tools/icons_land.py` when a reviewed Crew
//! icon set lands in `assets/icons/`; edit that folder and run the tool, not this file.

/// (id, SVG document).
pub const SVGS: &[(&str, &str)] = &[
    ("app.help", include_str!("../assets/icons/app.help.svg")),
    (
        "app.journal",
        include_str!("../assets/icons/app.journal.svg"),
    ),
    (
        "app.materials",
        include_str!("../assets/icons/app.materials.svg"),
    ),
    ("app.new", include_str!("../assets/icons/app.new.svg")),
    ("app.open", include_str!("../assets/icons/app.open.svg")),
    ("app.parts", include_str!("../assets/icons/app.parts.svg")),
    ("app.redo", include_str!("../assets/icons/app.redo.svg")),
    ("app.sancho", include_str!("../assets/icons/app.sancho.svg")),
    ("app.save", include_str!("../assets/icons/app.save.svg")),
    ("app.search", include_str!("../assets/icons/app.search.svg")),
    (
        "app.settings",
        include_str!("../assets/icons/app.settings.svg"),
    ),
    (
        "app.toolbox",
        include_str!("../assets/icons/app.toolbox.svg"),
    ),
    ("app.undo", include_str!("../assets/icons/app.undo.svg")),
    (
        "app.workshop",
        include_str!("../assets/icons/app.workshop.svg"),
    ),
    ("env.design", include_str!("../assets/icons/env.design.svg")),
    ("env.plan", include_str!("../assets/icons/env.plan.svg")),
    (
        "env.publish",
        include_str!("../assets/icons/env.publish.svg"),
    ),
    ("env.space", include_str!("../assets/icons/env.space.svg")),
    ("make.box", include_str!("../assets/icons/make.box.svg")),
    ("make.cut", include_str!("../assets/icons/make.cut.svg")),
    (
        "make.extrude",
        include_str!("../assets/icons/make.extrude.svg"),
    ),
    ("make.hole", include_str!("../assets/icons/make.hole.svg")),
    ("make.join", include_str!("../assets/icons/make.join.svg")),
    ("make.move", include_str!("../assets/icons/make.move.svg")),
    (
        "make.revolve",
        include_str!("../assets/icons/make.revolve.svg"),
    ),
    ("make.split", include_str!("../assets/icons/make.split.svg")),
    ("sketch.arc", include_str!("../assets/icons/sketch.arc.svg")),
    (
        "sketch.circle",
        include_str!("../assets/icons/sketch.circle.svg"),
    ),
    (
        "sketch.dimension",
        include_str!("../assets/icons/sketch.dimension.svg"),
    ),
    (
        "sketch.finish",
        include_str!("../assets/icons/sketch.finish.svg"),
    ),
    (
        "sketch.line",
        include_str!("../assets/icons/sketch.line.svg"),
    ),
    (
        "sketch.lock",
        include_str!("../assets/icons/sketch.lock.svg"),
    ),
    (
        "sketch.mirror",
        include_str!("../assets/icons/sketch.mirror.svg"),
    ),
    ("sketch.new", include_str!("../assets/icons/sketch.new.svg")),
    (
        "sketch.rect",
        include_str!("../assets/icons/sketch.rect.svg"),
    ),
    (
        "sketch.trim",
        include_str!("../assets/icons/sketch.trim.svg"),
    ),
    (
        "stock.assign",
        include_str!("../assets/icons/stock.assign.svg"),
    ),
    (
        "stock.board",
        include_str!("../assets/icons/stock.board.svg"),
    ),
    ("stock.kerf", include_str!("../assets/icons/stock.kerf.svg")),
    (
        "stock.layout",
        include_str!("../assets/icons/stock.layout.svg"),
    ),
    (
        "stock.meter",
        include_str!("../assets/icons/stock.meter.svg"),
    ),
    (
        "stock.offcut",
        include_str!("../assets/icons/stock.offcut.svg"),
    ),
    (
        "stock.sheet",
        include_str!("../assets/icons/stock.sheet.svg"),
    ),
    (
        "stock.shopping",
        include_str!("../assets/icons/stock.shopping.svg"),
    ),
    ("view.fit", include_str!("../assets/icons/view.fit.svg")),
    ("view.front", include_str!("../assets/icons/view.front.svg")),
    ("view.orbit", include_str!("../assets/icons/view.orbit.svg")),
    ("view.pan", include_str!("../assets/icons/view.pan.svg")),
    (
        "view.section",
        include_str!("../assets/icons/view.section.svg"),
    ),
    ("view.zoom", include_str!("../assets/icons/view.zoom.svg")),
];
