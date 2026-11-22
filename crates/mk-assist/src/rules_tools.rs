//! Sancho's tools rule pack (doc 07 section 7.5): steps against the maker's tools.
//!
//! Written by Kimi (the Crew, campaign mk-core-19, 2026-11-22) from the contract
//! crew/contracts/mk_rules_tools.md (ATLAS-api-harness), which Claude finished from Gordon's brief.

use crate::findings::{Finding, Kind, Location, Severity, Source};

/// Rule id for a capability no usable tool supplies.
pub const RULE_MISSING: &str = "tools.missing";
/// Rule id for a capability available only away from home.
pub const RULE_AWAY: &str = "tools.away";
/// Rule id for tools whose accuracy exceeds a step's tolerance.
pub const RULE_ACCURACY: &str = "tools.accuracy";
/// Rule id for loud tools planned during quiet hours.
pub const RULE_NOISE: &str = "tools.noise";

/// The rule ids in the order they are tried for each step and capability.
pub const RULES: [&str; 4] = [RULE_MISSING, RULE_AWAY, RULE_ACCURACY, RULE_NOISE];

/// How the maker has a tool (doc 04 section 4.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    /// The tool is at home and owned by the maker.
    Owned,
    /// The tool is shared and available at home.
    Shared,
    /// The tool is available at a makerspace.
    Makerspace,
    /// The tool can be rented.
    Rentable,
    /// The tool is wanted but not currently available.
    Wishlist,
}

/// A tool's noise level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Noise {
    /// Quiet operation.
    Quiet,
    /// Moderate noise.
    Moderate,
    /// Loud operation.
    Loud,
}

/// A stretch of one day: day 0 is Monday .. 6 Sunday; minutes after midnight, start < end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot {
    /// Day of the week, from 0 (Monday) through 6 (Sunday).
    pub day: u8,
    /// Start time in minutes after midnight.
    pub start_minute: u16,
    /// End time in minutes after midnight.
    pub end_minute: u16,
}

/// One of the maker's tools.
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    /// The tool record's identifier.
    pub id: String,
    /// Capabilities as the tool record writes them: "cut.straight", "cut.straight:rough".
    pub supplies: Vec<String>,
    /// How the maker has the tool.
    pub access: Access,
    /// The tool's accuracy in millimetres, when known.
    pub accuracy_mm: Option<f64>,
    /// The tool's noise level.
    pub noise: Noise,
}

/// One step of the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// The step's identifier.
    pub id: u32,
    /// Capabilities required by the step.
    pub needs: Vec<String>,
    /// The step's tolerance in millimetres, when given.
    pub tolerance_mm: Option<f64>,
    /// The step's planned time, when given.
    pub planned: Option<Slot>,
}

/// What the tools pack reads.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ToolFacts {
    /// Plan steps, in evaluation order.
    pub steps: Vec<Step>,
    /// Tool records, in input order.
    pub tools: Vec<Tool>,
    /// Slots during which the maker asked for quiet.
    pub quiet_hours: Vec<Slot>,
}

/// Returns the text before the first ':', or all of `supply` when there is no ':'.
pub fn family(supply: &str) -> &str {
    match supply.split_once(':') {
        Some((name, _)) => name,
        None => supply,
    }
}

/// Returns whether the access mode makes a tool usable.
pub fn usable(access: Access) -> bool {
    matches!(
        access,
        Access::Owned | Access::Shared | Access::Makerspace | Access::Rentable
    )
}

/// Returns whether two slots share a day and strictly overlap in minutes.
pub fn overlaps(a: Slot, b: Slot) -> bool {
    a.day == b.day && a.start_minute < b.end_minute && b.start_minute < a.end_minute
}

fn finding(
    rule: &str,
    step_id: u32,
    kind: Kind,
    severity: Severity,
    words: (String, String, Option<&str>),
    facts: Vec<(String, String)>,
) -> Finding {
    let (what, why, fix) = words;
    Finding {
        kind,
        severity,
        source: Source::Rule(rule.to_string()),
        location: Location::Step(step_id),
        what,
        why,
        fix: fix.map(str::to_string),
        facts,
    }
}

/// Checks each step capability against the tools and returns findings in evaluation order.
pub fn evaluate(facts: &ToolFacts) -> Vec<Finding> {
    let mut findings = Vec::new();

    for step in &facts.steps {
        let mut checked: Vec<&str> = Vec::new();

        for need in &step.needs {
            let capability = need.as_str();
            if checked.contains(&capability) {
                continue;
            }
            checked.push(capability);

            let tools: Vec<&Tool> = facts
                .tools
                .iter()
                .filter(|tool| {
                    usable(tool.access)
                        && tool
                            .supplies
                            .iter()
                            .any(|supply| family(supply) == capability)
                })
                .collect();

            // T1: wishlist tools have already been excluded from `tools`.
            if tools.is_empty() {
                findings.push(finding(
                    RULE_MISSING,
                    step.id,
                    Kind::Inventory,
                    Severity::Warning,
                    (
                        format!("No tool for {capability}"),
                        format!(
                            concat!(
                                "Step {id} needs {capability}, and none of your tools does that ",
                                "(a wishlist tool does not count)."
                            ),
                            id = step.id,
                            capability = capability,
                        ),
                        Some("inventory.add_tool"),
                    ),
                    vec![
                        ("step".to_string(), step.id.to_string()),
                        ("capability".to_string(), capability.to_string()),
                    ],
                ));
                continue;
            }

            // T2: a makerspace anywhere in the list selects the word "makerspace".
            if tools
                .iter()
                .all(|tool| matches!(tool.access, Access::Makerspace | Access::Rentable))
            {
                let place = if tools.iter().any(|tool| tool.access == Access::Makerspace) {
                    "makerspace"
                } else {
                    "rental"
                };
                findings.push(finding(
                    RULE_AWAY,
                    step.id,
                    Kind::Tip,
                    Severity::Info,
                    (
                        format!("{capability} means a trip"),
                        format!(
                            "Step {id} needs {capability}: you have it at a {place}, not at home.",
                            id = step.id,
                        ),
                        None,
                    ),
                    vec![
                        ("step".to_string(), step.id.to_string()),
                        ("capability".to_string(), capability.to_string()),
                        ("access".to_string(), place.to_string()),
                    ],
                ));
                continue;
            }

            // T3: only finite accuracies participate; strict improvement keeps the first tie.
            if let Some(tolerance) = step.tolerance_mm {
                if tolerance.is_finite() && tolerance > 0.0 {
                    let mut best: Option<(&Tool, f64)> = None;
                    for &tool in &tools {
                        if let Some(accuracy) = tool.accuracy_mm {
                            if accuracy.is_finite() {
                                let better = match best.as_ref() {
                                    Some((_, current)) => accuracy < *current,
                                    None => true,
                                };
                                if better {
                                    best = Some((tool, accuracy));
                                }
                            }
                        }
                    }

                    if let Some((tool, accuracy)) = best {
                        if accuracy > tolerance {
                            findings.push(finding(
                                RULE_ACCURACY,
                                step.id,
                                Kind::NotSuitable,
                                Severity::Warning,
                                (
                                    format!("{} is not accurate enough", tool.id),
                                    format!(
                                        concat!(
                                            "Step {id} needs {capability} within {tolerance} mm; ",
                                            "{tool} holds about {accuracy} mm. Clamp a guide, or ",
                                            "use a finer tool."
                                        ),
                                        id = step.id,
                                        capability = capability,
                                        tolerance = tolerance,
                                        tool = tool.id,
                                        accuracy = accuracy,
                                    ),
                                    None,
                                ),
                                vec![
                                    ("step".to_string(), step.id.to_string()),
                                    ("capability".to_string(), capability.to_string()),
                                    ("tool".to_string(), tool.id.clone()),
                                    ("accuracy_mm".to_string(), accuracy.to_string()),
                                    ("tolerance_mm".to_string(), tolerance.to_string()),
                                ],
                            ));
                            continue;
                        }
                    }
                }
            }

            // T4: the missing-tool branch above guarantees a first tool.
            if let Some(planned) = step.planned {
                let during_quiet_hours = facts
                    .quiet_hours
                    .iter()
                    .any(|quiet| overlaps(planned, *quiet));
                if during_quiet_hours && tools.iter().all(|tool| tool.noise == Noise::Loud) {
                    if let Some(tool) = tools.first() {
                        findings.push(finding(
                            RULE_NOISE,
                            step.id,
                            Kind::Denied,
                            Severity::Warning,
                            (
                                format!("{} is loud in your quiet hours", tool.id),
                                format!(
                                    "Step {id} is planned when you asked for quiet, and {tool} is loud.",
                                    id = step.id,
                                    tool = tool.id,
                                ),
                                Some("plan.move_step"),
                            ),
                            vec![
                                ("step".to_string(), step.id.to_string()),
                                ("capability".to_string(), capability.to_string()),
                                ("tool".to_string(), tool.id.clone()),
                                ("day".to_string(), planned.day.to_string()),
                            ],
                        ));
                    }
                }
            }
        }
    }

    findings
}
