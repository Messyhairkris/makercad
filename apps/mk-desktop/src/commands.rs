//! MakerCAD's commands as a person finds them: maker words first ("Pull up"), CAD words one step
//! away ("Extrude"), and the other words people use ("make it thinner"). Command search
//! (Ctrl+K, doc 11 §11.4) reads this list; the toolbox will too.
//!
//! Every command says what it does in the shell ([`Act`]). Commands whose tools are not built yet
//! are listed as well, so search shows where they will live and in which phase they come
//! ([`Act::Later`]): the whole program's shape is findable from the first test build.
//!
//! Search ([`search`]) is plain word matching, no model: each word of the query is matched
//! against the command's words (exactly, as the start of a word, a word inside it, or one typo
//! away), weighted by where the word is (name 3, CAD words 2, other words 1.5, place 1), and a
//! command that matches more of the query's words ranks higher.

use solidcraft_viewport::StandardView;

use crate::environments::Env;
use crate::theme::Look;

/// Lengths shown in millimetres or inches (doc 11; the typing side is `mk-units`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Units {
    /// Millimetres (and metres for big things).
    Mm,
    /// Inches (and feet for big things).
    In,
}

impl Units {
    /// The key stored in the settings file. Never change one (compatibility rule C3).
    pub fn key(self) -> &'static str {
        match self {
            Units::Mm => "mm",
            Units::In => "in",
        }
    }

    /// The units stored under `key`, if it is one.
    pub fn from_key(key: &str) -> Option<Units> {
        [Units::Mm, Units::In].into_iter().find(|u| u.key() == key)
    }

    /// A person's height or a big size: "1.68 m" or "5 ft 6 in".
    pub fn height(self, mm: f64) -> String {
        match self {
            Units::Mm => format!("{:.2} m", mm / 1000.0),
            Units::In => {
                let inches = (mm / mk_units::mm_per(mk_units::Unit::In)).round() as i64;
                format!("{} ft {} in", inches / 12, inches % 12)
            }
        }
    }
}

/// Something that comes in a later phase: what, when, and one sentence on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Later {
    /// What it is ("Round over").
    pub what: &'static str,
    /// The roadmap phase that builds it ("P1", "P3").
    pub phase: &'static str,
    /// One sentence for the person.
    pub note: &'static str,
}

/// What a command does in the shell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    /// The opening screen.
    Home,
    /// Open SolidCraft's example number n.
    OpenExample(usize),
    /// Frame the design alone.
    FitView,
    /// Frame the design, its stage and Sancho (the opening view).
    WholeView,
    /// Look from one side, or the corner.
    LookFrom(StandardView),
    /// Show or hide Sancho.
    Sancho,
    /// Take the last step back.
    Undo,
    /// Put the last undone step back.
    Redo,
    /// Go to an environment's tab.
    Tab(Env),
    /// Change the look.
    SetLook(Look),
    /// Show lengths in millimetres or inches.
    SetUnits(Units),
    /// Open command search.
    Search,
    /// Settings.
    Settings,
    /// About MakerCAD.
    About,
    /// Save the open design as a project file.
    Save,
    /// Not built yet.
    Later(Later),
}

/// A command as search and the toolbox show it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Command {
    /// Its icon id (doc 18).
    pub icon: &'static str,
    /// Maker words: what a person calls it.
    pub name: &'static str,
    /// CAD words, for people coming from other CAD programs ("" when the same).
    pub cad: &'static str,
    /// Other words people use for it, separated by spaces.
    pub also: &'static str,
    /// Where it lives: the toolbox group or environment ("View", "Make", "Stock & Cut").
    pub place: &'static str,
    /// Its shortcut ("Ctrl+Z"), or "".
    pub key: &'static str,
    /// It works on the open design (search says so when none is open).
    pub needs_design: bool,
    /// What it does.
    pub act: Act,
}

impl Command {
    /// Whether it works now (not a later phase's).
    pub fn ready(&self) -> bool {
        !matches!(self.act, Act::Later(_))
    }
}

const fn cmd(
    icon: &'static str,
    name: &'static str,
    cad: &'static str,
    also: &'static str,
    place: &'static str,
    act: Act,
) -> Command {
    Command {
        icon,
        name,
        cad,
        also,
        place,
        key: "",
        needs_design: false,
        act,
    }
}

const fn on_design(mut c: Command) -> Command {
    c.needs_design = true;
    c
}

const fn key(mut c: Command, key: &'static str) -> Command {
    c.key = key;
    c
}

const fn later(
    icon: &'static str,
    name: &'static str,
    cad: &'static str,
    also: &'static str,
    place: &'static str,
    phase: &'static str,
    note: &'static str,
) -> Command {
    cmd(
        icon,
        name,
        cad,
        also,
        place,
        Act::Later(Later {
            what: name,
            phase,
            note,
        }),
    )
}

/// The modelling tools' note: they are the next step of P1.
const TOOLS_NEXT: &str = "The modelling tools are the next part of P1, built on SolidCraft's \
                          own commands.";
const STOCK_P3: &str = "Stock & Cut is P3: parts cut from the boards and sheets on your rack.";

/// Every command, in the order an empty search lists them. `examples` are SolidCraft's samples
/// (name, one line), opened by [`Act::OpenExample`].
pub fn all(examples: &[(&'static str, &'static str)]) -> Vec<Command> {
    let mut v = vec![
        // The view.
        on_design(cmd(
            "view.fit",
            "Fit the view",
            "Zoom to fit",
            "frame zoom all whole fill window see part",
            "View",
            Act::FitView,
        )),
        on_design(cmd(
            "view.zoom",
            "Show everything",
            "Home view",
            "home stage bench sancho everything whole scene reset",
            "View",
            Act::WholeView,
        )),
        on_design(cmd(
            "view.iso",
            "Look from the corner",
            "Isometric view",
            "iso isometric corner 3d angle",
            "View",
            Act::LookFrom(StandardView::Iso),
        )),
        on_design(cmd(
            "view.front",
            "Look from the front",
            "Front view",
            "front elevation straight on",
            "View",
            Act::LookFrom(StandardView::Front),
        )),
        on_design(cmd(
            "view.top",
            "Look from the top",
            "Top view",
            "top plan above down overhead bird",
            "View",
            Act::LookFrom(StandardView::Top),
        )),
        on_design(cmd(
            "view.right",
            "Look from the right",
            "Right view",
            "right side elevation",
            "View",
            Act::LookFrom(StandardView::Right),
        )),
        on_design(cmd(
            "app.sancho",
            "Show or hide Sancho",
            "Scale figure",
            "sancho person human figure man woman people size scale hide show",
            "View",
            Act::Sancho,
        )),
        // Changes.
        key(
            on_design(cmd(
                "app.undo",
                "Undo",
                "",
                "back oops revert take mistake",
                "Edit",
                Act::Undo,
            )),
            "Ctrl+Z",
        ),
        key(
            on_design(cmd(
                "app.redo",
                "Redo",
                "",
                "again forward",
                "Edit",
                Act::Redo,
            )),
            "Ctrl+Y",
        ),
        // The project.
        key(
            on_design(cmd(
                "app.save",
                "Save the project",
                "Save",
                "keep store write file disk project maker",
                "Project",
                Act::Save,
            )),
            "Ctrl+S",
        ),
        // The modelling tools (next in P1).
        later(
            "sketch.new",
            "Start a sketch",
            "Create sketch",
            "draw outline drawing profile shape",
            "Sketch",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "sketch.rect",
            "Rectangle",
            "",
            "box square rect oblong",
            "Sketch",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "sketch.circle",
            "Circle",
            "",
            "round disc ring",
            "Sketch",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "sketch.line",
            "Line",
            "",
            "straight edge",
            "Sketch",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "sketch.dimension",
            "Size",
            "Dimension",
            "length width height set size measure",
            "Sketch",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.extrude",
            "Pull up",
            "Extrude",
            "push pull raise thicker thinner taller shorter height thickness",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.hole",
            "Drill a hole",
            "Hole",
            "drill bore screw bolt pilot countersink counterbore",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.round",
            "Round over",
            "Fillet",
            "round corners edges soften radius smooth",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.bevel",
            "Bevel",
            "Chamfer",
            "chamfer angle corner edge",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.cut",
            "Cut away",
            "Extrude cut",
            "cut slot notch groove pocket remove subtract trench",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.hollow",
            "Hollow out",
            "Shell",
            "hollow tray box walls",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.repeat",
            "Repeat in a row",
            "Pattern",
            "array copies row grid many",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.split",
            "Split",
            "Split body",
            "divide halves two",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "make.join",
            "Join",
            "Combine",
            "glue merge union together",
            "Make",
            "P1",
            TOOLS_NEXT,
        ),
        later(
            "view.measure",
            "Measure",
            "Measure",
            "distance tape how long far ruler",
            "View",
            "P1",
            TOOLS_NEXT,
        ),
    ];
    for (i, &(name, about)) in examples.iter().enumerate() {
        v.push(cmd(
            "app.open",
            name,
            "Open example",
            about,
            "Examples",
            Act::OpenExample(i),
        ));
    }
    v.extend([
        // The environments.
        cmd(
            "env.design",
            "Design",
            "Modelling",
            "model sketch solid shape",
            "Environments",
            Act::Tab(Env::Design),
        ),
        cmd(
            "env.stock",
            "Stock & Cut",
            "Nesting",
            "cut list board lumber plywood sheet waste layout",
            "Environments",
            Act::Tab(Env::Stock),
        ),
        cmd(
            "env.plan",
            "Plan",
            "Schedule",
            "steps order when calendar time",
            "Environments",
            Act::Tab(Env::Plan),
        ),
        cmd(
            "env.build",
            "Build",
            "Build guide",
            "guide instructions assemble steps bench",
            "Environments",
            Act::Tab(Env::Build),
        ),
        cmd(
            "env.space",
            "Space",
            "Room",
            "room scan wall alcove fits",
            "Environments",
            Act::Tab(Env::Space),
        ),
        cmd(
            "env.publish",
            "Share",
            "Publish",
            "publish export log post bundle",
            "Environments",
            Act::Tab(Env::Share),
        ),
        // Stock & Cut's commands (P3).
        later(
            "stock.board",
            "Board from stock",
            "",
            "lumber plank wood 2x4 1x6 board",
            "Stock & Cut",
            "P3",
            STOCK_P3,
        ),
        later(
            "stock.layout",
            "Cut layout",
            "Nesting",
            "cut list nest saw plan",
            "Stock & Cut",
            "P3",
            STOCK_P3,
        ),
        later(
            "stock.shopping",
            "Shopping list",
            "Bill of materials",
            "buy store hardware materials bom",
            "Stock & Cut",
            "P3",
            STOCK_P3,
        ),
        later(
            "stock.offcut",
            "Keep the offcuts",
            "",
            "scrap leftover offcut waste",
            "Stock & Cut",
            "P3",
            STOCK_P3,
        ),
        // The app.
        cmd(
            "app.home",
            "Go to the opening screen",
            "Home",
            "home start projects close leave",
            "App",
            Act::Home,
        ),
        cmd(
            "app.look",
            "Daydream look",
            "Theme",
            "light warm paper sky colours colors theme",
            "Settings",
            Act::SetLook(Look::Daydream),
        ),
        cmd(
            "app.look",
            "Pastel look",
            "Theme",
            "lilac soft light colours colors theme",
            "Settings",
            Act::SetLook(Look::Pastel),
        ),
        cmd(
            "app.look",
            "Night Workshop look",
            "Dark theme",
            "dark night evening black colours colors theme",
            "Settings",
            Act::SetLook(Look::NightWorkshop),
        ),
        cmd(
            "app.units",
            "Use millimetres",
            "Metric units",
            "mm metric cm metre meter millimeter units",
            "Settings",
            Act::SetUnits(Units::Mm),
        ),
        cmd(
            "app.units",
            "Use inches",
            "Imperial units",
            "inch in imperial feet foot ft fractions units",
            "Settings",
            Act::SetUnits(Units::In),
        ),
        cmd(
            "app.settings",
            "Settings",
            "Preferences",
            "options look graphics backend",
            "App",
            Act::Settings,
        ),
        cmd(
            "app.help",
            "About MakerCAD",
            "Help",
            "help version licence license graphics adapter",
            "App",
            Act::About,
        ),
    ]);
    v
}

/// Words that say nothing about which command is meant.
const STOP: &[&str] = &[
    "a", "an", "the", "to", "for", "of", "it", "its", "this", "that", "my", "me", "i", "please",
    "can", "could", "you", "how", "do", "does", "some", "in", "on", "at", "with", "and", "or",
    "into", "onto", "be", "is", "are", "want", "would", "like", "let", "lets", "s",
];

/// The query's words, lower case, without the words that say nothing.
pub fn query_words(query: &str) -> Vec<String> {
    words(query)
        .filter(|w| !STOP.contains(&w.as_str()))
        .collect()
}

fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
}

/// A word without a plural "s" ("holes" and "hole" are the same word).
fn stem(w: &str) -> &str {
    if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") {
        &w[..w.len() - 1]
    } else {
        w
    }
}

/// Endings that make another form of the same word ("thin" + "ner", "round" + "ed").
const ENDINGS: &[&str] = &[
    "s", "es", "ed", "d", "er", "r", "est", "ing", "ly", "y", "ier", "iest", "ner", "ter", "ger",
    "der", "mer", "ning", "ting", "ping",
];

/// How well query word `q` matches command word `t`, 0 to 1.
fn likeness(q: &str, t: &str) -> f32 {
    let (q, t) = (stem(q), stem(t));
    if q == t {
        1.0
    } else if t.starts_with(q) {
        // Typing the start of a word.
        if q.len() >= 2 {
            0.8
        } else {
            0.4
        }
    } else if t.len() >= 3 && q.starts_with(t) && ENDINGS.contains(&&q[t.len()..]) {
        // "thinner" for "thin", "rounded" for "round"; not "fillet" for "fill".
        0.6
    } else if q.len() >= 4 && one_typo(q, t) {
        0.5
    } else {
        0.0
    }
}

/// `a` and `b` differ by one letter changed, added or removed.
fn one_typo(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (short, long) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    match long.len() - short.len() {
        0 => {
            short
                .iter()
                .zip(long.iter())
                .filter(|(x, y)| x != y)
                .count()
                == 1
        }
        1 => (0..long.len()).any(|skip| {
            long.iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, c)| c)
                .eq(short.iter())
        }),
        _ => false,
    }
}

/// How well `command` answers the query words, 0 for not at all.
pub fn score(query: &[String], command: &Command) -> f32 {
    if query.is_empty() {
        return 0.0;
    }
    let fields = [
        (command.name, 3.0),
        (command.cad, 2.0),
        (command.also, 1.5),
        (command.place, 1.0),
    ];
    let mut total = 0.0;
    let mut matched = 0usize;
    for q in query {
        let best = fields
            .iter()
            .flat_map(|(text, weight)| words(text).map(move |t| (t, *weight)))
            .map(|(t, weight)| weight * likeness(q, &t))
            .fold(0.0_f32, f32::max);
        if best > 0.0 {
            matched += 1;
        }
        total += best;
    }
    if matched == 0 {
        return 0.0;
    }
    // The name starting with what was typed is what a person means most often.
    let typed = query.join(" ");
    if command.name.to_lowercase().starts_with(&typed) {
        total += 2.0;
    }
    total * matched as f32 / query.len() as f32
}

/// A result is shown only if it scores at least this share of the best one.
const KEEP_FROM_BEST: f32 = 0.3;

/// The commands that answer `query`, best first (indices into `commands`), at most `limit`.
/// An empty query lists the commands in their own order.
pub fn search(query: &str, commands: &[Command], limit: usize) -> Vec<usize> {
    let q = query_words(query);
    if q.is_empty() {
        return (0..commands.len().min(limit)).collect();
    }
    let mut hits: Vec<(f32, usize)> = commands
        .iter()
        .enumerate()
        .map(|(i, c)| (score(&q, c), i))
        .filter(|(s, _)| *s > 0.0)
        .collect();
    // Best first; equal scores keep the list's order (the more useful command first).
    hits.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    // Far weaker matches are noise ("hole" is one typo from "home" and "whole").
    let floor = hits.first().map_or(0.0, |h| h.0 * KEEP_FROM_BEST);
    hits.into_iter()
        .take_while(|h| h.0 >= floor)
        .take(limit)
        .map(|(_, i)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLES: &[(&str, &str)] = &[
        (
            "Sample Plate",
            "Parametric plate with fillets, holes and a bored boss",
        ),
        (
            "Sheet Metal Bracket",
            "Base flange with edge flanges and a hem on a 2 mm rule",
        ),
    ];

    fn first(query: &str) -> &'static str {
        let all = all(EXAMPLES);
        let hits = search(query, &all, 8);
        hits.first().map_or("(nothing)", |&i| all[i].name)
    }

    #[test]
    fn plain_words_find_the_command() {
        // The worked examples of doc 11 §11.4 and the UI pass 2 screens.
        for (query, expected) in [
            ("put a hole for a screw", "Drill a hole"),
            ("make it thinner", "Pull up"),
            ("cut a slot", "Cut away"),
            ("fit", "Fit the view"),
            ("undo", "Undo"),
            ("top", "Look from the top"),
            ("dark", "Night Workshop look"),
            ("inch", "Use inches"),
            ("inches", "Use inches"),
            ("sancho", "Show or hide Sancho"),
            ("bracket", "Sheet Metal Bracket"),
            ("stock", "Stock & Cut"),
        ] {
            assert_eq!(first(query), expected, "{query:?}");
        }
    }

    #[test]
    fn far_weaker_matches_are_left_out() {
        let all = all(EXAMPLES);
        let names: Vec<&str> = search("put a hole for a screw", &all, 8)
            .into_iter()
            .map(|i| all[i].name)
            .collect();
        assert_eq!(names.first(), Some(&"Drill a hole"));
        for noise in [
            "Go to the opening screen",
            "Fit the view",
            "Show everything",
        ] {
            assert!(!names.contains(&noise), "{noise} in {names:?}");
        }
    }

    #[test]
    fn cad_words_and_typos_find_it_too() {
        assert_eq!(first("fillet"), "Round over");
        let all = all(EXAMPLES);
        let fit = all.iter().position(|c| c.name == "Fit the view");
        assert!(
            !search("fillet", &all, 8).contains(&fit.expect("listed")),
            "\"fillet\" is not a form of \"fill\""
        );
        assert_eq!(first("rounded"), "Round over");
        assert_eq!(first("extrude"), "Pull up");
        assert_eq!(first("chamfer"), "Bevel");
        assert_eq!(first("dril"), "Drill a hole");
        assert_eq!(first("drlil"), "(nothing)");
        assert_eq!(first("drill a hoel"), "Drill a hole");
    }

    #[test]
    fn nonsense_finds_nothing_and_nothing_typed_lists_everything_in_order() {
        let all = all(EXAMPLES);
        assert!(search("xyzzy", &all, 8).is_empty());
        assert_eq!(
            search("the a of", &all, 8).len(),
            8,
            "only stop words: the list in order"
        );
        assert_eq!(search("", &all, 3), vec![0, 1, 2]);
        assert_eq!(search("", &all, 1000).len(), all.len());
    }

    #[test]
    fn every_command_has_an_icon_a_name_and_a_place() {
        for c in all(EXAMPLES) {
            assert!(c.icon.contains('.'), "{c:?}");
            assert!(!c.name.is_empty() && !c.place.is_empty(), "{c:?}");
            if let Act::Later(l) = c.act {
                assert_eq!(l.what, c.name);
                assert!(l.phase.starts_with('P') && l.note.ends_with('.'), "{c:?}");
                assert!(!c.ready());
            }
        }
    }

    #[test]
    fn typos_are_one_letter() {
        assert!(!one_typo("hoel", "hole"), "a swap is two changes");
        assert!(one_typo("dril", "drill"));
        assert!(one_typo("drill", "dril"));
        assert!(one_typo("bevel", "bevil"));
        assert!(!one_typo("bevel", "level2"));
    }

    #[test]
    fn units_read_and_write_their_keys_and_heights() {
        for u in [Units::Mm, Units::In] {
            assert_eq!(Units::from_key(u.key()), Some(u));
        }
        assert_eq!(Units::from_key("cubits"), None);
        assert_eq!(Units::Mm.height(1680.0), "1.68 m");
        assert_eq!(Units::In.height(1680.0), "5 ft 6 in");
        assert_eq!(Units::In.height(1829.0), "6 ft 0 in");
    }
}
