//! Tests for mk_project::intent_card written by Deepak (the Crew, campaign mk-core-31) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_project::intent_card` (contract v1, rules A1-A11).

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_project::intent_card::{
    build, headline, is_blank, next_outstanding, outstanding, progress, Card, Facts, Item,
    ItemState, DETAIL_FIT, DETAIL_NO_DEADLINE, HEADLINE_ALL_SET, HEADLINE_INTENT_MISSING,
    HEADLINE_MANY_OPEN, HEADLINE_ONE_OPEN, IDS, ID_BUDGET, ID_DEADLINE, ID_FIT, ID_GOAL,
    ID_INSPIRATION, ID_MATERIALS, ID_NAME, ID_TOOLS,
};

fn item(id: &str, label: &str, state: ItemState, detail: Option<&str>) -> Item {
    Item {
        id: id.to_string(),
        label: label.to_string(),
        state,
        detail: detail.map(str::to_string),
    }
}

fn done(id: &str, label: &str, detail: &str) -> Item {
    item(id, label, ItemState::Done, Some(detail))
}

fn open(id: &str, label: &str) -> Item {
    item(id, label, ItemState::Outstanding, None)
}

fn find<'a>(card: &'a Card, id: &str) -> &'a Item {
    card.items
        .iter()
        .find(|it| it.id == id)
        .expect("card holds the item")
}

fn open_ids(card: &Card) -> Vec<&str> {
    outstanding(card).iter().map(|it| it.id.as_str()).collect()
}

fn full_facts() -> Facts {
    Facts {
        name: Some("Shelf".to_string()),
        goal: Some("Build a shelf".to_string()),
        fit_known: true,
        tools: 2,
        materials: 1,
        deadline: Some("Dec 12".to_string()),
        no_deadline: false,
        budget: Some("\u{a3}50".to_string()),
        inspiration: 1,
    }
}

// ---------------------------------------------------------------- rules A1-A11

#[test]
fn rule_a1_is_blank() {
    assert!(is_blank(""));
    assert!(is_blank("  \t\n"));
    assert!(is_blank(" \u{a0} "));
    assert!(!is_blank(" x "));
    assert!(!is_blank("x"));
    assert!(!is_blank("\u{200b}"));
}

#[test]
fn rule_a2_text_known_and_trimmed() {
    let facts = Facts {
        name: Some("  Alcove dresser  ".to_string()),
        goal: Some("  Build  a dresser \n".to_string()),
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(find(&card, ID_NAME).state, ItemState::Done);
    assert_eq!(
        find(&card, ID_NAME).detail,
        Some("Alcove dresser".to_string())
    );
    // Inner whitespace is unchanged by the trim.
    assert_eq!(
        find(&card, ID_GOAL).detail,
        Some("Build  a dresser".to_string())
    );
    // None is never known.
    let none_card = build(&Facts::default());
    assert_eq!(find(&none_card, ID_NAME).state, ItemState::Outstanding);
    assert_eq!(find(&none_card, ID_NAME).detail, None);
}

#[test]
fn rule_a3_count_known_above_zero() {
    let zero = build(&Facts::default());
    assert_eq!(find(&zero, ID_TOOLS).state, ItemState::Outstanding);
    assert_eq!(find(&zero, ID_MATERIALS).state, ItemState::Outstanding);
    assert_eq!(find(&zero, ID_INSPIRATION).state, ItemState::Outstanding);
    let one = build(&Facts {
        tools: 1,
        materials: 1,
        inspiration: 1,
        ..Facts::default()
    });
    assert_eq!(find(&one, ID_TOOLS).state, ItemState::Done);
    assert_eq!(find(&one, ID_MATERIALS).state, ItemState::Done);
    assert_eq!(find(&one, ID_INSPIRATION).state, ItemState::Done);
}

#[test]
fn rule_a4_fit_and_deadline_flags() {
    let fit = build(&Facts {
        fit_known: true,
        ..Facts::default()
    });
    assert_eq!(find(&fit, ID_FIT).state, ItemState::Done);
    assert_eq!(find(&fit, ID_FIT).detail, Some(DETAIL_FIT.to_string()));
    let no_fit = build(&Facts::default());
    assert_eq!(find(&no_fit, ID_FIT).state, ItemState::Outstanding);

    let no_deadline = build(&Facts {
        no_deadline: true,
        ..Facts::default()
    });
    assert_eq!(find(&no_deadline, ID_DEADLINE).state, ItemState::Done);
    assert_eq!(
        find(&no_deadline, ID_DEADLINE).detail,
        Some(DETAIL_NO_DEADLINE.to_string())
    );

    // Blank deadline text with no_deadline false is not known.
    let blank = build(&Facts {
        deadline: Some("   ".to_string()),
        ..Facts::default()
    });
    assert_eq!(find(&blank, ID_DEADLINE).state, ItemState::Outstanding);
    assert_eq!(find(&blank, ID_DEADLINE).detail, None);
}

#[test]
fn rule_a5_build_writes_eight_items_in_card_order() {
    let card = build(&Facts::default());
    assert!(card.intent_missing);
    assert_eq!(
        card.items,
        vec![
            open(ID_NAME, "Name"),
            open(ID_GOAL, "What to make"),
            open(ID_FIT, "Where it fits"),
            open(ID_TOOLS, "Your tools"),
            open(ID_MATERIALS, "Your materials"),
            open(ID_DEADLINE, "When you need it"),
            open(ID_BUDGET, "Budget"),
            open(ID_INSPIRATION, "What inspires you"),
        ]
    );
    let ids: Vec<&str> = card.items.iter().map(|it| it.id.as_str()).collect();
    assert_eq!(ids, IDS.to_vec());
}

#[test]
fn rule_a6_done_details() {
    let card = build(&full_facts());
    assert_eq!(
        card.items,
        vec![
            done(ID_NAME, "Name", "Shelf"),
            done(ID_GOAL, "What to make", "Build a shelf"),
            done(ID_FIT, "Where it fits", DETAIL_FIT),
            done(ID_TOOLS, "Your tools", "2 tools"),
            done(ID_MATERIALS, "Your materials", "1 material"),
            done(ID_DEADLINE, "When you need it", "Dec 12"),
            done(ID_BUDGET, "Budget", "\u{a3}50"),
            done(ID_INSPIRATION, "What inspires you", "1 inspiration"),
        ]
    );
}

#[test]
fn rule_a7_count_detail() {
    let card = build(&Facts {
        tools: 3,
        materials: 1,
        inspiration: 2,
        ..Facts::default()
    });
    assert_eq!(find(&card, ID_TOOLS).detail, Some("3 tools".to_string()));
    assert_eq!(
        find(&card, ID_MATERIALS).detail,
        Some("1 material".to_string())
    );
    assert_eq!(
        find(&card, ID_INSPIRATION).detail,
        Some("2 inspirations".to_string())
    );
    let other = build(&Facts {
        tools: 1,
        materials: 2,
        inspiration: 1,
        ..Facts::default()
    });
    assert_eq!(find(&other, ID_TOOLS).detail, Some("1 tool".to_string()));
    assert_eq!(
        find(&other, ID_MATERIALS).detail,
        Some("2 materials".to_string())
    );
    assert_eq!(
        find(&other, ID_INSPIRATION).detail,
        Some("1 inspiration".to_string())
    );
}

#[test]
fn rule_a8_state_and_detail_agree() {
    let card = build(&full_facts());
    for it in &card.items {
        assert_eq!(it.state, ItemState::Done);
        assert!(it.detail.is_some());
    }
    let empty = build(&Facts::default());
    for it in &empty.items {
        assert_eq!(it.state, ItemState::Outstanding);
        assert_eq!(it.detail, None);
    }
}

#[test]
fn rule_a9_outstanding_next_and_progress() {
    let card = build(&Facts {
        fit_known: false,
        budget: None,
        ..full_facts()
    });
    assert_eq!(open_ids(&card), vec!["fit", "budget"]);
    assert_eq!(
        next_outstanding(&card).map(|it| it.id.as_str()),
        Some("fit")
    );
    assert_eq!(progress(&card), (6, 8));
}

#[test]
fn rule_a10_headline_first_match_wins() {
    // (1) intent_missing wins even when every item is done.
    let missing = Card {
        intent_missing: true,
        items: vec![done(ID_NAME, "Name", "Shelf")],
    };
    assert_eq!(headline(&missing), HEADLINE_INTENT_MISSING);
    // (2) nothing outstanding.
    let all_set = build(&full_facts());
    assert_eq!(headline(&all_set), HEADLINE_ALL_SET);
    // (3) exactly one outstanding.
    let one = build(&Facts {
        budget: None,
        ..full_facts()
    });
    assert_eq!(headline(&one), HEADLINE_ONE_OPEN);
    // (4) two or more, with {n} replaced by the count.
    assert_eq!(HEADLINE_MANY_OPEN, "{n} questions still open");
    let two = build(&Facts {
        budget: None,
        fit_known: false,
        ..full_facts()
    });
    assert_eq!(headline(&two), "2 questions still open");
    let three = build(&Facts {
        budget: None,
        fit_known: false,
        inspiration: 0,
        ..full_facts()
    });
    assert_eq!(headline(&three), "3 questions still open");
}

#[test]
fn rule_a11_empty_card_never_panics() {
    let card = Card {
        intent_missing: false,
        items: vec![],
    };
    assert!(outstanding(&card).is_empty());
    assert_eq!(next_outstanding(&card), None);
    assert_eq!(progress(&card), (0, 0));
    assert_eq!(headline(&card), HEADLINE_ALL_SET);
    let missing = Card {
        intent_missing: true,
        items: vec![],
    };
    assert_eq!(headline(&missing), HEADLINE_INTENT_MISSING);
}

#[test]
fn rule_a11_unknown_id_never_panics() {
    let card = Card {
        intent_missing: false,
        items: vec![
            done("mystery", "Mystery", "something"),
            open("other", "Other"),
        ],
    };
    assert_eq!(open_ids(&card), vec!["other"]);
    assert_eq!(
        next_outstanding(&card).map(|it| it.id.as_str()),
        Some("other")
    );
    assert_eq!(progress(&card), (1, 2));
    assert_eq!(headline(&card), HEADLINE_ONE_OPEN);
}

#[test]
fn rule_a11_huge_counts_never_panic() {
    let card = build(&Facts {
        tools: usize::MAX,
        materials: usize::MAX,
        inspiration: usize::MAX,
        ..Facts::default()
    });
    assert_eq!(
        find(&card, ID_TOOLS).detail,
        Some(format!("{} tools", usize::MAX))
    );
    assert_eq!(
        find(&card, ID_MATERIALS).detail,
        Some(format!("{} materials", usize::MAX))
    );
    assert_eq!(
        find(&card, ID_INSPIRATION).detail,
        Some(format!("{} inspirations", usize::MAX))
    );
}

// ------------------------------------------------------------ worked examples

#[test]
fn example_1_nothing_given() {
    let card = build(&Facts::default());
    assert!(card.intent_missing);
    assert_eq!(
        card.items,
        vec![
            open(ID_NAME, "Name"),
            open(ID_GOAL, "What to make"),
            open(ID_FIT, "Where it fits"),
            open(ID_TOOLS, "Your tools"),
            open(ID_MATERIALS, "Your materials"),
            open(ID_DEADLINE, "When you need it"),
            open(ID_BUDGET, "Budget"),
            open(ID_INSPIRATION, "What inspires you"),
        ]
    );
    assert_eq!(progress(&card), (0, 8));
    assert_eq!(
        next_outstanding(&card).map(|it| it.id.as_str()),
        Some("name")
    );
    assert_eq!(
        headline(&card),
        "Intent missing: tell Sancho what you want to make"
    );
}

#[test]
fn example_2_name_only() {
    let card = build(&Facts {
        name: Some("Alcove dresser".to_string()),
        ..Facts::default()
    });
    assert_eq!(
        find(&card, ID_NAME),
        &done(ID_NAME, "Name", "Alcove dresser")
    );
    assert_eq!(progress(&card), (1, 8));
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn example_3_name_with_surrounding_spaces() {
    let facts = Facts {
        name: Some("  Alcove dresser  ".to_string()),
        ..Facts::default()
    };
    assert!(!is_blank(facts.name.as_deref().unwrap()));
    let card = build(&facts);
    assert_eq!(
        find(&card, ID_NAME),
        &done(ID_NAME, "Name", "Alcove dresser")
    );
}

#[test]
fn example_4_goal_with_newline() {
    let card = build(&Facts {
        goal: Some("  Build a dresser\n".to_string()),
        ..Facts::default()
    });
    assert!(!card.intent_missing);
    assert_eq!(
        find(&card, ID_GOAL),
        &done(ID_GOAL, "What to make", "Build a dresser")
    );
    assert_eq!(
        open_ids(&card),
        vec![
            "name",
            "fit",
            "tools",
            "materials",
            "deadline",
            "budget",
            "inspiration"
        ]
    );
    assert_eq!(headline(&card), "7 questions still open");
}

#[test]
fn example_5_blank_goal() {
    let card = build(&Facts {
        goal: Some("   ".to_string()),
        ..Facts::default()
    });
    assert!(card.intent_missing);
    assert_eq!(find(&card, ID_GOAL), &open(ID_GOAL, "What to make"));
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn example_6_tools_and_materials() {
    let card = build(&Facts {
        tools: 3,
        materials: 1,
        ..Facts::default()
    });
    assert_eq!(
        find(&card, ID_TOOLS),
        &done(ID_TOOLS, "Your tools", "3 tools")
    );
    assert_eq!(
        find(&card, ID_MATERIALS),
        &done(ID_MATERIALS, "Your materials", "1 material")
    );
    assert_eq!(progress(&card), (2, 8));
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn example_7_one_tool() {
    let card = build(&Facts {
        tools: 1,
        ..Facts::default()
    });
    assert_eq!(
        find(&card, ID_TOOLS),
        &done(ID_TOOLS, "Your tools", "1 tool")
    );
}

#[test]
fn example_8_deadline_text() {
    let card = build(&Facts {
        deadline: Some("Dec 12".to_string()),
        ..Facts::default()
    });
    assert_eq!(
        find(&card, ID_DEADLINE),
        &done(ID_DEADLINE, "When you need it", "Dec 12")
    );
}

#[test]
fn example_9_no_deadline_flag() {
    let card = build(&Facts {
        no_deadline: true,
        ..Facts::default()
    });
    assert_eq!(
        find(&card, ID_DEADLINE),
        &done(ID_DEADLINE, "When you need it", "No deadline")
    );
}

#[test]
fn example_10_deadline_text_beats_no_deadline_flag() {
    let facts = Facts {
        deadline: Some(" Dec 12 ".to_string()),
        no_deadline: true,
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(
        find(&card, ID_DEADLINE),
        &done(ID_DEADLINE, "When you need it", "Dec 12")
    );
}

#[test]
fn example_11_budget_text_then_blank() {
    let given = build(&Facts {
        budget: Some("\u{a3}200".to_string()),
        ..Facts::default()
    });
    assert_eq!(
        find(&given, ID_BUDGET),
        &done(ID_BUDGET, "Budget", "\u{a3}200")
    );
    let blank = build(&Facts {
        budget: Some("  ".to_string()),
        ..Facts::default()
    });
    assert_eq!(find(&blank, ID_BUDGET), &open(ID_BUDGET, "Budget"));
}

#[test]
fn example_12_inspiration_counts() {
    let two = build(&Facts {
        inspiration: 2,
        ..Facts::default()
    });
    assert_eq!(
        find(&two, ID_INSPIRATION),
        &done(ID_INSPIRATION, "What inspires you", "2 inspirations")
    );
    let one = build(&Facts {
        inspiration: 1,
        ..Facts::default()
    });
    assert_eq!(
        find(&one, ID_INSPIRATION),
        &done(ID_INSPIRATION, "What inspires you", "1 inspiration")
    );
}

#[test]
fn example_13_fit_flag() {
    let known = build(&Facts {
        fit_known: true,
        ..Facts::default()
    });
    assert_eq!(
        find(&known, ID_FIT),
        &done(ID_FIT, "Where it fits", "Size known")
    );
    let unknown = build(&Facts {
        fit_known: false,
        ..Facts::default()
    });
    assert_eq!(find(&unknown, ID_FIT), &open(ID_FIT, "Where it fits"));
}

#[test]
fn example_14_everything_given() {
    let card = build(&full_facts());
    assert!(!card.intent_missing);
    assert_eq!(
        card.items,
        vec![
            done(ID_NAME, "Name", "Shelf"),
            done(ID_GOAL, "What to make", "Build a shelf"),
            done(ID_FIT, "Where it fits", "Size known"),
            done(ID_TOOLS, "Your tools", "2 tools"),
            done(ID_MATERIALS, "Your materials", "1 material"),
            done(ID_DEADLINE, "When you need it", "Dec 12"),
            done(ID_BUDGET, "Budget", "\u{a3}50"),
            done(ID_INSPIRATION, "What inspires you", "1 inspiration"),
        ]
    );
    assert_eq!(progress(&card), (8, 8));
    assert!(outstanding(&card).is_empty());
    assert_eq!(next_outstanding(&card), None);
    assert_eq!(headline(&card), "All set");
}

#[test]
fn example_15_budget_missing() {
    let card = build(&Facts {
        budget: None,
        ..full_facts()
    });
    assert_eq!(open_ids(&card), vec!["budget"]);
    assert_eq!(
        next_outstanding(&card).map(|it| it.id.as_str()),
        Some("budget")
    );
    assert_eq!(progress(&card), (7, 8));
    assert_eq!(headline(&card), "1 question still open");
}

#[test]
fn example_16_fit_and_budget_missing() {
    let card = build(&Facts {
        fit_known: false,
        budget: None,
        ..full_facts()
    });
    assert_eq!(open_ids(&card), vec!["fit", "budget"]);
    assert_eq!(
        next_outstanding(&card).map(|it| it.id.as_str()),
        Some("fit")
    );
    assert_eq!(progress(&card), (6, 8));
    assert_eq!(headline(&card), "2 questions still open");
}

#[test]
fn example_17_goal_missing() {
    let card = build(&Facts {
        goal: None,
        ..full_facts()
    });
    assert!(card.intent_missing);
    assert_eq!(open_ids(&card), vec!["goal"]);
    assert_eq!(progress(&card), (7, 8));
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn example_18_card_not_from_build() {
    let card = Card {
        intent_missing: false,
        items: vec![],
    };
    assert!(outstanding(&card).is_empty());
    assert_eq!(next_outstanding(&card), None);
    assert_eq!(progress(&card), (0, 0));
    assert_eq!(headline(&card), "All set");
}
