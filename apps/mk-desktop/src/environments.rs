//! The environments (doc 06): the tabs along the top of the workspace, and the others the
//! "+ environment" menu lists. Only Design works in P1; every other one says what it will do and
//! in which phase of the roadmap (doc 14) it comes, so the shell shows the whole program's shape.

/// A tab in the environment bar (`docs/design/ui-v2`, doc 11 §11.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Env {
    /// Sketch and Solid together: SolidCraft's modelling (P1).
    Design,
    /// Stock & Cut (doc 08).
    Stock,
    /// Plan (doc 10).
    Plan,
    /// Build: the step-by-step guide at the bench (doc 10).
    Build,
    /// Space & Scan (doc 09).
    Space,
    /// Share: the build log and bundles (doc 10).
    Share,
}

/// What an environment does and when it comes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The roadmap phase that builds it ("P3").
    pub phase: &'static str,
    /// One sentence on what it will do, in a maker's words.
    pub does: &'static str,
}

impl Env {
    /// The tabs, in the bar's order.
    pub const TABS: [Env; 6] = [
        Env::Design,
        Env::Stock,
        Env::Plan,
        Env::Build,
        Env::Space,
        Env::Share,
    ];

    /// The tab's name.
    pub fn name(self) -> &'static str {
        match self {
            Env::Design => "Design",
            Env::Stock => "Stock & Cut",
            Env::Plan => "Plan",
            Env::Build => "Build",
            Env::Space => "Space",
            Env::Share => "Share",
        }
    }

    /// Its key (for `--tab`).
    pub fn key(self) -> &'static str {
        match self {
            Env::Design => "design",
            Env::Stock => "stock",
            Env::Plan => "plan",
            Env::Build => "build",
            Env::Space => "space",
            Env::Share => "share",
        }
    }

    /// The tab with this key, if it is one.
    pub fn from_key(key: &str) -> Option<Env> {
        Env::TABS.into_iter().find(|e| e.key() == key)
    }

    /// The tab's icon id (doc 18).
    pub fn icon(self) -> &'static str {
        match self {
            Env::Design => "env.design",
            Env::Stock => "env.stock",
            Env::Plan => "env.plan",
            Env::Build => "env.build",
            Env::Space => "env.space",
            Env::Share => "env.publish",
        }
    }

    /// `None` when it works now; else what it will do and when.
    pub fn plan(self) -> Option<Plan> {
        let plan = |phase, does| Some(Plan { phase, does });
        match self {
            Env::Design => None,
            Env::Stock => plan(
                "P3",
                "Every part gets the board or sheet it is cut from. You see the cut layout, the \
                 waste, the offcuts worth keeping and a shopping list, from what is on your rack.",
            ),
            Env::Plan => plan(
                "P5",
                "The steps to build it, in a sensible order, with glue and paint drying times \
                 and the evenings you have free.",
            ),
            Env::Build => plan(
                "P5",
                "The build guide at the bench: one step at a time, with the parts, tools and \
                 sizes for that step, and printable templates.",
            ),
            Env::Space => plan(
                "P6",
                "The room the thing goes in, from a tape measure or a phone scan, so you can \
                 check it fits before you cut anything.",
            ),
            Env::Share => plan(
                "P9",
                "A build log with photos and notes, and a bundle someone else can build from, \
                 with a privacy check first.",
            ),
        }
    }
}

/// An environment the "+ environment" menu lists (doc 06's table), not yet a tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoreEnv {
    /// Its name.
    pub name: &'static str,
    /// Its icon id (set `env2` of the Crew's icons).
    pub icon: &'static str,
    /// What it does and when it comes.
    pub plan: Plan,
}

/// Doc 06's other environments, in the roadmap's order.
pub const MORE: [MoreEnv; 11] = [
    more(
        "Journal",
        "app.journal",
        "P2",
        "Why you made each choice, the questions still open, and notes from each session.",
    ),
    more(
        "Quick Build",
        "env.quick",
        "P4",
        "Push and pull shapes straight off the floor, the quick way to rough out furniture.",
    ),
    more(
        "Drawings",
        "env.drawings",
        "P5",
        "Paper drawings with sizes, to print or hand to someone.",
    ),
    more(
        "Projector",
        "env.projector",
        "P5",
        "Shine the cut lines onto your board or fabric at true size.",
    ),
    more(
        "Fold & Package",
        "env.fold",
        "P7",
        "Boxes, cartons and papercraft: fold lines, flaps and a flat layout to cut.",
    ),
    more(
        "Soft Goods",
        "env.soft",
        "P8",
        "Sewing patterns from a 3D shape, with seams and allowances.",
    ),
    more(
        "Motion",
        "env.motion",
        "P8",
        "Hinges and moving parts: check a fold-away bed clears the wall as it swings.",
    ),
    more(
        "Laser, plotter, vinyl",
        "env.laser",
        "P10",
        "Lay out flat parts for a laser cutter or a plotter, with power and speed for your material.",
    ),
    more(
        "3D print prep",
        "env.print",
        "P10",
        "Hand a part to a slicer, oriented and ready to print.",
    ),
    more(
        "CNC",
        "env.cnc",
        "P10",
        "Toolpaths for a CNC router: profiles, pockets and holes cut from your sheets.",
    ),
    more(
        "Render",
        "env.render",
        "P10",
        "A nice picture of the finished thing, for the log or for a client.",
    ),
];

const fn more(
    name: &'static str,
    icon: &'static str,
    phase: &'static str,
    does: &'static str,
) -> MoreEnv {
    MoreEnv {
        name,
        icon,
        plan: Plan { phase, does },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_design_works_in_p1_and_every_other_tab_says_when_it_comes() {
        for env in Env::TABS {
            match env.plan() {
                None => assert_eq!(env, Env::Design),
                Some(p) => {
                    assert!(p.phase.starts_with('P'), "{env:?}");
                    assert!(p.does.ends_with('.'), "{env:?}: {}", p.does);
                }
            }
        }
    }

    #[test]
    fn every_tab_round_trips_through_its_key() {
        for env in Env::TABS {
            assert_eq!(Env::from_key(env.key()), Some(env));
        }
        assert_eq!(Env::from_key("kitchen"), None);
    }

    #[test]
    fn every_environment_has_its_own_icon_and_name() {
        let mut icons: Vec<&str> = Env::TABS.iter().map(|e| e.icon()).collect();
        icons.extend(MORE.iter().map(|m| m.icon));
        let mut names: Vec<&str> = Env::TABS.iter().map(|e| e.name()).collect();
        names.extend(MORE.iter().map(|m| m.name));
        for list in [&mut icons, &mut names] {
            let n = list.len();
            list.sort_unstable();
            list.dedup();
            assert_eq!(list.len(), n, "a duplicate in {list:?}");
        }
    }
}
