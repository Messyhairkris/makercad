//! Contract tests for mk_project::intent_card, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_intent_card.md. They were the hidden tests of campaign mk-core-31.

//! Hidden tests for `mk_project::intent_card`, from `crew/contracts/mk_intent_card.md`.

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

/// The item with this id, or a failed test.
fn item_of<'a>(card: &'a Card, id: &str) -> &'a Item {
    card.items
        .iter()
        .find(|item| item.id == id)
        .expect("the card has this item id")
}

fn detail_of(card: &Card, id: &str) -> Option<String> {
    item_of(card, id).detail.clone()
}

fn state_of(card: &Card, id: &str) -> ItemState {
    item_of(card, id).state
}

fn ids_of(card: &Card) -> Vec<String> {
    outstanding(card)
        .iter()
        .map(|item| item.id.clone())
        .collect()
}

fn empty_card() -> Card {
    Card {
        intent_missing: false,
        items: Vec::new(),
    }
}

/// Every fact known (worked example 14).
fn facts_all_known() -> Facts {
    Facts {
        name: Some("Shelf".to_string()),
        goal: Some("Build a shelf".to_string()),
        fit_known: true,
        tools: 2,
        materials: 1,
        deadline: Some("Dec 12".to_string()),
        no_deadline: false,
        budget: Some("£50".to_string()),
        inspiration: 1,
    }
}

// A1

#[test]
fn a1_is_blank_for_empty_and_whitespace_only_text() {
    assert!(is_blank(""));
    assert!(is_blank(" "));
    assert!(is_blank("  \t\n"));
}

#[test]
fn a1_is_not_blank_when_any_visible_character_is_there() {
    assert!(!is_blank("x"));
    assert!(!is_blank(" x "));
    assert!(!is_blank("\n\tdresser\t"));
}

// A2

#[test]
fn a2_blank_text_is_not_known() {
    let facts = Facts {
        name: Some("   ".to_string()),
        goal: Some("\t\n".to_string()),
        deadline: Some(" ".to_string()),
        budget: Some("".to_string()),
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(state_of(&card, ID_NAME), ItemState::Outstanding);
    assert_eq!(state_of(&card, ID_GOAL), ItemState::Outstanding);
    assert_eq!(state_of(&card, ID_DEADLINE), ItemState::Outstanding);
    assert_eq!(state_of(&card, ID_BUDGET), ItemState::Outstanding);
}

#[test]
fn a2_known_text_is_trimmed() {
    let facts = Facts {
        name: Some("  Alcove dresser  ".to_string()),
        goal: Some("  Build a dresser\n".to_string()),
        budget: Some("\t£200 ".to_string()),
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(
        detail_of(&card, ID_NAME),
        Some("Alcove dresser".to_string())
    );
    assert_eq!(
        detail_of(&card, ID_GOAL),
        Some("Build a dresser".to_string())
    );
    assert_eq!(detail_of(&card, ID_BUDGET), Some("£200".to_string()));
}

// A3

#[test]
fn a3_zero_counts_are_not_known() {
    let facts = Facts {
        tools: 0,
        materials: 0,
        inspiration: 0,
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(state_of(&card, ID_TOOLS), ItemState::Outstanding);
    assert_eq!(state_of(&card, ID_MATERIALS), ItemState::Outstanding);
    assert_eq!(state_of(&card, ID_INSPIRATION), ItemState::Outstanding);
    assert_eq!(detail_of(&card, ID_TOOLS), None);
}

// A4

#[test]
fn a4_fit_flag_and_no_deadline_make_items_done() {
    let facts = Facts {
        fit_known: true,
        no_deadline: true,
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(state_of(&card, ID_FIT), ItemState::Done);
    assert_eq!(detail_of(&card, ID_FIT), Some(DETAIL_FIT.to_string()));
    assert_eq!(state_of(&card, ID_DEADLINE), ItemState::Done);
    assert_eq!(
        detail_of(&card, ID_DEADLINE),
        Some(DETAIL_NO_DEADLINE.to_string())
    );
}

#[test]
fn a4_unset_flags_leave_items_outstanding() {
    let card = build(&Facts::default());
    assert_eq!(state_of(&card, ID_FIT), ItemState::Outstanding);
    assert_eq!(state_of(&card, ID_DEADLINE), ItemState::Outstanding);
}

// A5

#[test]
fn a5_build_writes_eight_items_in_ids_order_with_labels() {
    let card = build(&Facts::default());
    assert_eq!(card.items.len(), 8);
    let ids: Vec<&str> = card.items.iter().map(|item| item.id.as_str()).collect();
    assert_eq!(ids, IDS.to_vec());
    let labels: Vec<&str> = card.items.iter().map(|item| item.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "Name",
            "What to make",
            "Where it fits",
            "Your tools",
            "Your materials",
            "When you need it",
            "Budget",
            "What inspires you",
        ]
    );
}

#[test]
fn a5_intent_missing_reads_only_the_goal() {
    let missing = build(&Facts::default());
    assert!(missing.intent_missing);
    let blank = build(&Facts {
        goal: Some("  \n".to_string()),
        ..Facts::default()
    });
    assert!(blank.intent_missing);
    let named_but_no_goal = build(&Facts {
        name: Some("Shelf".to_string()),
        ..Facts::default()
    });
    assert!(named_but_no_goal.intent_missing);
    let given = build(&Facts {
        goal: Some("Build a shelf".to_string()),
        ..Facts::default()
    });
    assert!(!given.intent_missing);
}

// A6

#[test]
fn a6_every_done_detail_is_the_specified_words() {
    let card = build(&facts_all_known());
    assert_eq!(detail_of(&card, ID_NAME), Some("Shelf".to_string()));
    assert_eq!(detail_of(&card, ID_GOAL), Some("Build a shelf".to_string()));
    assert_eq!(detail_of(&card, ID_FIT), Some(DETAIL_FIT.to_string()));
    assert_eq!(detail_of(&card, ID_TOOLS), Some("2 tools".to_string()));
    assert_eq!(
        detail_of(&card, ID_MATERIALS),
        Some("1 material".to_string())
    );
    assert_eq!(detail_of(&card, ID_DEADLINE), Some("Dec 12".to_string()));
    assert_eq!(detail_of(&card, ID_BUDGET), Some("£50".to_string()));
    assert_eq!(
        detail_of(&card, ID_INSPIRATION),
        Some("1 inspiration".to_string())
    );
}

#[test]
fn a6_deadline_text_wins_over_no_deadline() {
    let facts = Facts {
        deadline: Some("  Dec 12 ".to_string()),
        no_deadline: true,
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(detail_of(&card, ID_DEADLINE), Some("Dec 12".to_string()));
}

// A7

#[test]
fn a7_count_detail_is_number_space_noun_with_singular_at_one() {
    let one = build(&Facts {
        tools: 1,
        materials: 1,
        inspiration: 1,
        ..Facts::default()
    });
    assert_eq!(detail_of(&one, ID_TOOLS), Some("1 tool".to_string()));
    assert_eq!(
        detail_of(&one, ID_MATERIALS),
        Some("1 material".to_string())
    );
    assert_eq!(
        detail_of(&one, ID_INSPIRATION),
        Some("1 inspiration".to_string())
    );

    let many = build(&Facts {
        tools: 3,
        materials: 4,
        inspiration: 2,
        ..Facts::default()
    });
    assert_eq!(detail_of(&many, ID_TOOLS), Some("3 tools".to_string()));
    assert_eq!(
        detail_of(&many, ID_MATERIALS),
        Some("4 materials".to_string())
    );
    assert_eq!(
        detail_of(&many, ID_INSPIRATION),
        Some("2 inspirations".to_string())
    );
}

// A8

#[test]
fn a8_open_items_have_no_detail_and_done_items_have_one() {
    let card = build(&Facts {
        tools: 2,
        ..Facts::default()
    });
    for item in &card.items {
        match item.state {
            ItemState::Done => assert!(item.detail.is_some()),
            ItemState::Outstanding => assert!(item.detail.is_none()),
        }
    }
    assert_eq!(card.items.len(), 8);
}

// A9

#[test]
fn a9_outstanding_keeps_card_order() {
    let card = build(&Facts {
        goal: Some("Build a shelf".to_string()),
        tools: 1,
        ..Facts::default()
    });
    assert_eq!(
        ids_of(&card),
        vec![
            "name".to_string(),
            "fit".to_string(),
            "materials".to_string(),
            "deadline".to_string(),
            "budget".to_string(),
            "inspiration".to_string(),
        ]
    );
}

#[test]
fn a9_next_outstanding_is_the_first_open_item() {
    let card = build(&Facts {
        name: Some("Shelf".to_string()),
        goal: Some("Build a shelf".to_string()),
        ..Facts::default()
    });
    let next = next_outstanding(&card).expect("an open item");
    assert_eq!(next.id, ID_FIT);
    assert_eq!(next.state, ItemState::Outstanding);
}

#[test]
fn a9_progress_counts_done_items_and_all_items() {
    let none_done = build(&Facts::default());
    assert_eq!(progress(&none_done), (0, 8));
    let all_done = build(&facts_all_known());
    assert_eq!(progress(&all_done), (8, 8));
    let name_only = build(&Facts {
        name: Some("Shelf".to_string()),
        ..Facts::default()
    });
    assert_eq!(progress(&name_only), (1, 8));
}

// A10

#[test]
fn a10_headline_intent_missing_comes_first() {
    let card = build(&Facts::default());
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
    let open = build(&Facts {
        goal: Some("  ".to_string()),
        name: Some("Shelf".to_string()),
        ..Facts::default()
    });
    assert_eq!(headline(&open), HEADLINE_INTENT_MISSING);
}

#[test]
fn a10_headline_all_set_when_nothing_is_outstanding() {
    let card = build(&facts_all_known());
    assert!(outstanding(&card).is_empty());
    assert_eq!(headline(&card), HEADLINE_ALL_SET);
}

#[test]
fn a10_headline_one_question_still_open() {
    let facts = Facts {
        budget: None,
        ..facts_all_known()
    };
    let card = build(&facts);
    assert_eq!(headline(&card), HEADLINE_ONE_OPEN);
}

#[test]
fn a10_headline_many_questions_still_open() {
    let facts = Facts {
        goal: Some("Build a shelf".to_string()),
        ..Facts::default()
    };
    let card = build(&facts);
    assert_eq!(headline(&card), "7 questions still open");
    let two = Facts {
        budget: None,
        fit_known: false,
        ..facts_all_known()
    };
    assert_eq!(headline(&build(&two)), "2 questions still open");
    assert_eq!(
        HEADLINE_MANY_OPEN.replace("{n}", "2"),
        "2 questions still open"
    );
}

// A11

#[test]
fn a11_empty_item_list_is_quiet() {
    let card = empty_card();
    assert!(outstanding(&card).is_empty());
    assert!(next_outstanding(&card).is_none());
    assert_eq!(progress(&card), (0, 0));
    assert_eq!(headline(&card), HEADLINE_ALL_SET);
}

#[test]
fn a11_empty_item_list_with_missing_intent_stays_missing() {
    let card = Card {
        intent_missing: true,
        items: Vec::new(),
    };
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

// Worked examples, one test per row of the contract's table.

#[test]
fn row1_empty_facts() {
    let card = build(&Facts::default());
    assert!(card.intent_missing);
    assert_eq!(card.items.len(), 8);
    assert!(card
        .items
        .iter()
        .all(|item| item.state == ItemState::Outstanding));
    assert!(card.items.iter().all(|item| item.detail.is_none()));
    assert_eq!(progress(&card), (0, 8));
    assert_eq!(next_outstanding(&card).expect("name is open").id, ID_NAME);
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn row2_name_only() {
    let card = build(&Facts {
        name: Some("Alcove dresser".to_string()),
        ..Facts::default()
    });
    assert_eq!(
        detail_of(&card, ID_NAME),
        Some("Alcove dresser".to_string())
    );
    assert_eq!(progress(&card), (1, 8));
    assert!(card.intent_missing);
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn row3_padded_name_is_trimmed() {
    let card = build(&Facts {
        name: Some("  Alcove dresser  ".to_string()),
        ..Facts::default()
    });
    assert!(!is_blank("  Alcove dresser  "));
    assert_eq!(
        detail_of(&card, ID_NAME),
        Some("Alcove dresser".to_string())
    );
    assert_eq!(state_of(&card, ID_NAME), ItemState::Done);
}

#[test]
fn row4_goal_only() {
    let card = build(&Facts {
        goal: Some("  Build a dresser\n".to_string()),
        ..Facts::default()
    });
    assert!(!card.intent_missing);
    assert_eq!(
        detail_of(&card, ID_GOAL),
        Some("Build a dresser".to_string())
    );
    assert_eq!(outstanding(&card).len(), 7);
    assert_eq!(headline(&card), "7 questions still open");
}

#[test]
fn row5_blank_goal_is_missing() {
    let card = build(&Facts {
        goal: Some("   ".to_string()),
        ..Facts::default()
    });
    assert!(card.intent_missing);
    assert_eq!(state_of(&card, ID_GOAL), ItemState::Outstanding);
    assert_eq!(detail_of(&card, ID_GOAL), None);
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn row6_three_tools_and_one_material() {
    let card = build(&Facts {
        tools: 3,
        materials: 1,
        ..Facts::default()
    });
    assert_eq!(detail_of(&card, ID_TOOLS), Some("3 tools".to_string()));
    assert_eq!(
        detail_of(&card, ID_MATERIALS),
        Some("1 material".to_string())
    );
    assert_eq!(progress(&card), (2, 8));
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn row7_one_tool_is_singular() {
    let card = build(&Facts {
        tools: 1,
        ..Facts::default()
    });
    assert_eq!(detail_of(&card, ID_TOOLS), Some("1 tool".to_string()));
}

#[test]
fn row8_deadline_text() {
    let card = build(&Facts {
        deadline: Some("Dec 12".to_string()),
        ..Facts::default()
    });
    assert_eq!(state_of(&card, ID_DEADLINE), ItemState::Done);
    assert_eq!(detail_of(&card, ID_DEADLINE), Some("Dec 12".to_string()));
}

#[test]
fn row9_no_deadline_flag() {
    let card = build(&Facts {
        no_deadline: true,
        ..Facts::default()
    });
    assert_eq!(state_of(&card, ID_DEADLINE), ItemState::Done);
    assert_eq!(
        detail_of(&card, ID_DEADLINE),
        Some("No deadline".to_string())
    );
}

#[test]
fn row10_deadline_text_beats_the_no_deadline_flag() {
    let card = build(&Facts {
        deadline: Some(" Dec 12 ".to_string()),
        no_deadline: true,
        ..Facts::default()
    });
    assert_eq!(state_of(&card, ID_DEADLINE), ItemState::Done);
    assert_eq!(detail_of(&card, ID_DEADLINE), Some("Dec 12".to_string()));
}

#[test]
fn row11_budget_text_or_none() {
    let given = build(&Facts {
        budget: Some("£200".to_string()),
        ..Facts::default()
    });
    assert_eq!(detail_of(&given, ID_BUDGET), Some("£200".to_string()));
    let blank = build(&Facts {
        budget: Some("  ".to_string()),
        ..Facts::default()
    });
    assert_eq!(state_of(&blank, ID_BUDGET), ItemState::Outstanding);
    assert_eq!(detail_of(&blank, ID_BUDGET), None);
}

#[test]
fn row12_inspiration_count() {
    let two = build(&Facts {
        inspiration: 2,
        ..Facts::default()
    });
    assert_eq!(
        detail_of(&two, ID_INSPIRATION),
        Some("2 inspirations".to_string())
    );
    let one = build(&Facts {
        inspiration: 1,
        ..Facts::default()
    });
    assert_eq!(
        detail_of(&one, ID_INSPIRATION),
        Some("1 inspiration".to_string())
    );
}

#[test]
fn row13_fit_flag() {
    let known = build(&Facts {
        fit_known: true,
        ..Facts::default()
    });
    assert_eq!(state_of(&known, ID_FIT), ItemState::Done);
    assert_eq!(detail_of(&known, ID_FIT), Some("Size known".to_string()));
    let unknown = build(&Facts::default());
    assert_eq!(state_of(&unknown, ID_FIT), ItemState::Outstanding);
}

#[test]
fn row14_every_fact_known() {
    let card = build(&facts_all_known());
    assert!(!card.intent_missing);
    assert!(card.items.iter().all(|item| item.state == ItemState::Done));
    assert_eq!(progress(&card), (8, 8));
    assert!(outstanding(&card).is_empty());
    assert!(next_outstanding(&card).is_none());
    assert_eq!(headline(&card), HEADLINE_ALL_SET);
}

#[test]
fn row15_budget_open_only() {
    let card = build(&Facts {
        budget: None,
        ..facts_all_known()
    });
    assert_eq!(ids_of(&card), vec!["budget".to_string()]);
    assert_eq!(
        next_outstanding(&card).expect("budget is open").id,
        ID_BUDGET
    );
    assert_eq!(progress(&card), (7, 8));
    assert_eq!(headline(&card), HEADLINE_ONE_OPEN);
}

#[test]
fn row16_fit_and_budget_open() {
    let card = build(&Facts {
        fit_known: false,
        budget: None,
        ..facts_all_known()
    });
    assert_eq!(ids_of(&card), vec!["fit".to_string(), "budget".to_string()]);
    assert_eq!(next_outstanding(&card).expect("fit is open").id, ID_FIT);
    assert_eq!(headline(&card), "2 questions still open");
}

#[test]
fn row17_blank_goal_with_everything_else_known() {
    let card = build(&Facts {
        goal: None,
        ..facts_all_known()
    });
    assert!(card.intent_missing);
    assert_eq!(ids_of(&card), vec!["goal".to_string()]);
    assert_eq!(progress(&card), (7, 8));
    assert_eq!(headline(&card), HEADLINE_INTENT_MISSING);
}

#[test]
fn row18_empty_item_list() {
    let card = empty_card();
    assert!(outstanding(&card).is_empty());
    assert!(next_outstanding(&card).is_none());
    assert_eq!(progress(&card), (0, 0));
    assert_eq!(headline(&card), HEADLINE_ALL_SET);
}
