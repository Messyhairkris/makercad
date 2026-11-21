//! Integration tests for `cutstock::lay_out`, written from the contract alone.

use mk_nest::{lay_out, BoardPlan, Layout, Part, Stock};

fn mk_stock(id: &str, length: f64) -> Stock {
    Stock {
        id: id.to_string(),
        length,
    }
}

fn mk_part(id: &str, length: f64, qty: u32) -> Part {
    Part {
        id: id.to_string(),
        length,
        qty,
    }
}

fn mk_plan(stock_id: &str, cuts: &[&str], kerf_total: f64, offcut: f64) -> BoardPlan {
    BoardPlan {
        stock_id: stock_id.to_string(),
        cuts: cuts.iter().map(|c| c.to_string()).collect(),
        kerf_total,
        offcut,
    }
}

fn board_length(stock: &[Stock], id: &str) -> f64 {
    let mut found = f64::NAN;
    for s in stock {
        if s.id == id {
            found = s.length;
        }
    }
    found
}

fn piece_length(parts: &[Part], id: &str) -> f64 {
    let mut found = f64::NAN;
    for p in parts {
        if p.id == id {
            found = p.length;
        }
    }
    found
}

// Rule 6: sum(cuts) + kerf_total + offcut == board length, within 1e-6.
fn check_invariant(layout: &Layout, stock: &[Stock], parts: &[Part]) {
    for b in &layout.boards {
        let mut sum = 0.0;
        for id in &b.cuts {
            sum += piece_length(parts, id);
        }
        let total = sum + b.kerf_total + b.offcut;
        let len = board_length(stock, &b.stock_id);
        assert!(
            (total - len).abs() < 1e-6,
            "board {} breaks rule 6: {} vs {}",
            b.stock_id,
            total,
            len
        );
    }
}

#[test]
fn example_a_15_boards_rule_3_exact() {
    // 18 pieces > 12, so rule 3 exactly: 4 posts, 4 boards of two rails, 3 of two sides.
    let stock: Vec<Stock> = (1..=15)
        .map(|i| mk_stock(&format!("b{}", i), 36.0))
        .collect();
    let parts = vec![
        mk_part("post", 33.0, 4),
        mk_part("rail", 15.0, 8),
        mk_part("side", 12.0, 6),
    ];
    let got = lay_out(&stock, &parts, 0.125, 6.0);
    let mut boards = Vec::new();
    for i in 1..=4 {
        boards.push(mk_plan(&format!("b{}", i), &["post"], 0.125, 2.875));
    }
    for i in 5..=8 {
        boards.push(mk_plan(&format!("b{}", i), &["rail", "rail"], 0.25, 5.75));
    }
    for i in 9..=11 {
        boards.push(mk_plan(&format!("b{}", i), &["side", "side"], 0.25, 11.75));
    }
    let want = Layout {
        boards,
        unplaced: Vec::new(),
        parts_length: 324.0,
        boards_length: 396.0,
        true_waste: 36.75,
        usable_offcuts: vec![11.75, 11.75, 11.75],
    };
    assert_eq!(got, want);
}

#[test]
fn example_b_piece_too_long_is_unplaced() {
    let stock = vec![mk_stock("a", 24.0), mk_stock("b", 48.0)];
    let parts = vec![mk_part("x", 60.0, 1), mk_part("y", 30.0, 1)];
    let got = lay_out(&stock, &parts, 0.0, 0.0);
    let want = Layout {
        boards: vec![mk_plan("b", &["y"], 0.0, 18.0)],
        unplaced: vec!["x".to_string()],
        parts_length: 30.0,
        boards_length: 48.0,
        true_waste: 0.0,
        usable_offcuts: vec![18.0],
    };
    assert_eq!(got, want);
}

#[test]
fn example_c_piece_uses_board_up_no_kerf() {
    // r - p == 0, so k = min(0.125, 0) = 0: no saw cut follows a piece that fills the board.
    let stock = vec![mk_stock("a", 30.0)];
    let parts = vec![mk_part("p", 30.0, 1)];
    let got = lay_out(&stock, &parts, 0.125, 6.0);
    let want = Layout {
        boards: vec![mk_plan("a", &["p"], 0.0, 0.0)],
        unplaced: Vec::new(),
        parts_length: 30.0,
        boards_length: 30.0,
        true_waste: 0.0,
        usable_offcuts: Vec::new(),
    };
    assert_eq!(got, want);
}

#[test]
fn example_d_rule_4_uses_two_boards() {
    // First-fit decreasing needs 3 boards; rule 4 must find 2 (4+3+3 twice).
    let stock = vec![
        mk_stock("a", 10.0),
        mk_stock("b", 10.0),
        mk_stock("c", 10.0),
        mk_stock("d", 10.0),
    ];
    let parts = vec![mk_part("a", 4.0, 2), mk_part("b", 3.0, 4)];
    let got = lay_out(&stock, &parts, 0.0, 0.0);
    assert_eq!(got.boards.len(), 2);
    assert_eq!(got.unplaced, Vec::<String>::new());
    assert_eq!(got.parts_length, 20.0);
    assert_eq!(got.boards_length, 20.0);
    assert_eq!(got.true_waste, 0.0);
    assert_eq!(got.usable_offcuts, Vec::<f64>::new());
    for b in &got.boards {
        assert_eq!(b.cuts.len(), 3);
        let a_count = b.cuts.iter().filter(|c| c.as_str() == "a").count();
        let b_count = b.cuts.iter().filter(|c| c.as_str() == "b").count();
        assert_eq!((a_count, b_count), (1, 2));
        assert_eq!(b.kerf_total, 0.0);
        assert_eq!(b.offcut, 0.0);
    }
}

#[test]
fn example_e_stock_runs_out() {
    // Rule 3 left pieces over, so rule 4 does not apply: one board, two unplaced.
    let stock = vec![mk_stock("a", 10.0)];
    let parts = vec![mk_part("p", 6.0, 3)];
    let got = lay_out(&stock, &parts, 0.0, 6.0);
    let want = Layout {
        boards: vec![mk_plan("a", &["p"], 0.0, 4.0)],
        unplaced: vec!["p".to_string(), "p".to_string()],
        parts_length: 6.0,
        boards_length: 10.0,
        true_waste: 4.0,
        usable_offcuts: Vec::new(),
    };
    assert_eq!(got, want);
}

#[test]
fn example_f_kerf_eats_into_the_last_gap() {
    // 10 - 4.95 - 0.125 = 4.925 left; the second 4.95 does not fit it.
    let stock = vec![mk_stock("a", 10.0)];
    let parts = vec![mk_part("p", 4.95, 2)];
    let got = lay_out(&stock, &parts, 0.125, 6.0);
    assert_eq!(got.boards.len(), 1);
    let b = &got.boards[0];
    assert_eq!(b.stock_id, "a");
    assert_eq!(b.cuts, vec!["p".to_string()]);
    assert!((b.kerf_total - 0.125).abs() < 1e-9);
    assert!((b.offcut - 4.925).abs() < 1e-9);
    assert_eq!(got.unplaced, vec!["p".to_string()]);
    assert!((got.parts_length - 4.95).abs() < 1e-9);
    assert_eq!(got.boards_length, 10.0);
    assert!((got.true_waste - 5.05).abs() < 1e-9);
    assert_eq!(got.usable_offcuts, Vec::<f64>::new());
}

#[test]
fn rule_1_zero_and_negative_length_parts_are_ignored() {
    // Only "p" is a piece; "z" (0) and "n" (-3) are neither placed nor unplaced.
    let stock = vec![mk_stock("a", 10.0)];
    let parts = vec![
        mk_part("z", 0.0, 2),
        mk_part("p", 5.0, 1),
        mk_part("n", -3.0, 1),
    ];
    let got = lay_out(&stock, &parts, 0.0, 6.0);
    let want = Layout {
        boards: vec![mk_plan("a", &["p"], 0.0, 5.0)],
        unplaced: Vec::new(),
        parts_length: 5.0,
        boards_length: 10.0,
        true_waste: 5.0,
        usable_offcuts: Vec::new(),
    };
    assert_eq!(got, want);
}

#[test]
fn rule_1_empty_parts_list_gives_an_empty_layout() {
    let stock = vec![mk_stock("a", 10.0), mk_stock("b", 12.0)];
    let got = lay_out(&stock, &[], 0.125, 6.0);
    let want = Layout {
        boards: Vec::new(),
        unplaced: Vec::new(),
        parts_length: 0.0,
        boards_length: 0.0,
        true_waste: 0.0,
        usable_offcuts: Vec::new(),
    };
    assert_eq!(got, want);
}

#[test]
fn rule_3_stable_order_for_equal_lengths() {
    // 13 pieces of 5 > 12, so rule 3 exactly; equal lengths keep their order in `parts`.
    let stock: Vec<Stock> = (1..=7)
        .map(|i| mk_stock(&format!("b{}", i), 10.0))
        .collect();
    let ids = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m",
    ];
    let parts: Vec<Part> = ids.iter().map(|id| mk_part(id, 5.0, 1)).collect();
    let got = lay_out(&stock, &parts, 0.0, 6.0);
    let want = Layout {
        boards: vec![
            mk_plan("b1", &["a", "b"], 0.0, 0.0),
            mk_plan("b2", &["c", "d"], 0.0, 0.0),
            mk_plan("b3", &["e", "f"], 0.0, 0.0),
            mk_plan("b4", &["g", "h"], 0.0, 0.0),
            mk_plan("b5", &["i", "j"], 0.0, 0.0),
            mk_plan("b6", &["k", "l"], 0.0, 0.0),
            mk_plan("b7", &["m"], 0.0, 5.0),
        ],
        unplaced: Vec::new(),
        parts_length: 65.0,
        boards_length: 70.0,
        true_waste: 5.0,
        usable_offcuts: Vec::new(),
    };
    assert_eq!(got, want);
}

#[test]
fn rule_3_more_than_12_pieces_uses_first_fit() {
    // 13 pieces of 1: b1 takes ten, b2 takes the last three, offcut 7.
    // Rule 5: offcut 7 >= usable_min 6, so it is usable and not waste.
    let stock = vec![mk_stock("b1", 10.0), mk_stock("b2", 10.0)];
    let parts: Vec<Part> = (1..=13)
        .map(|i| mk_part(&format!("q{}", i), 1.0, 1))
        .collect();
    let got = lay_out(&stock, &parts, 0.0, 6.0);
    let first: Vec<String> = (1..=10).map(|i| format!("q{}", i)).collect();
    let second: Vec<String> = (11..=13).map(|i| format!("q{}", i)).collect();
    let want = Layout {
        boards: vec![
            BoardPlan {
                stock_id: "b1".to_string(),
                cuts: first,
                kerf_total: 0.0,
                offcut: 0.0,
            },
            BoardPlan {
                stock_id: "b2".to_string(),
                cuts: second,
                kerf_total: 0.0,
                offcut: 7.0,
            },
        ],
        unplaced: Vec::new(),
        parts_length: 13.0,
        boards_length: 20.0,
        true_waste: 0.0,
        usable_offcuts: vec![7.0],
    };
    assert_eq!(got, want);
}

#[test]
fn rule_2_kerf_is_clamped_to_the_remaining_gap() {
    // r - p = 0.05 < kerf 0.125, so k = 0.05 and the board is used up exactly.
    let stock = vec![mk_stock("a", 10.0)];
    let parts = vec![mk_part("p", 9.95, 1)];
    let got = lay_out(&stock, &parts, 0.125, 6.0);
    assert_eq!(got.boards.len(), 1);
    let b = &got.boards[0];
    assert_eq!(b.cuts, vec!["p".to_string()]);
    assert!((b.kerf_total - 0.05).abs() < 1e-9);
    assert!(b.offcut.abs() < 1e-9);
    assert!((got.true_waste - 0.05).abs() < 1e-9);
    assert_eq!(got.usable_offcuts, Vec::<f64>::new());
}

#[test]
fn rule_5_usable_offcuts_longest_first_and_true_waste() {
    // 13 pieces of 5 on boards 10..15: offcuts 0, 1, 2, 3, 4, 0.
    let stock = vec![
        mk_stock("b1", 10.0),
        mk_stock("b2", 11.0),
        mk_stock("b3", 12.0),
        mk_stock("b4", 13.0),
        mk_stock("b5", 14.0),
        mk_stock("b6", 15.0),
    ];
    let parts: Vec<Part> = (1..=13)
        .map(|i| mk_part(&format!("p{}", i), 5.0, 1))
        .collect();
    let got = lay_out(&stock, &parts, 0.0, 2.0);
    let want = Layout {
        boards: vec![
            mk_plan("b1", &["p1", "p2"], 0.0, 0.0),
            mk_plan("b2", &["p3", "p4"], 0.0, 1.0),
            mk_plan("b3", &["p5", "p6"], 0.0, 2.0),
            mk_plan("b4", &["p7", "p8"], 0.0, 3.0),
            mk_plan("b5", &["p9", "p10"], 0.0, 4.0),
            mk_plan("b6", &["p11", "p12", "p13"], 0.0, 0.0),
        ],
        unplaced: Vec::new(),
        parts_length: 65.0,
        boards_length: 75.0,
        true_waste: 1.0,
        usable_offcuts: vec![4.0, 3.0, 2.0],
    };
    assert_eq!(got, want);
}

#[test]
fn rule_6_invariant_holds_for_every_board() {
    let stock_a: Vec<Stock> = (1..=15)
        .map(|i| mk_stock(&format!("b{}", i), 36.0))
        .collect();
    let parts_a = vec![
        mk_part("post", 33.0, 4),
        mk_part("rail", 15.0, 8),
        mk_part("side", 12.0, 6),
    ];
    let layout_a = lay_out(&stock_a, &parts_a, 0.125, 6.0);
    check_invariant(&layout_a, &stock_a, &parts_a);

    let stock_d = vec![
        mk_stock("a", 10.0),
        mk_stock("b", 10.0),
        mk_stock("c", 10.0),
        mk_stock("d", 10.0),
    ];
    let parts_d = vec![mk_part("a", 4.0, 2), mk_part("b", 3.0, 4)];
    let layout_d = lay_out(&stock_d, &parts_d, 0.0, 0.0);
    check_invariant(&layout_d, &stock_d, &parts_d);

    let stock_f = vec![mk_stock("a", 10.0)];
    let parts_f = vec![mk_part("p", 4.95, 2)];
    let layout_f = lay_out(&stock_f, &parts_f, 0.125, 6.0);
    check_invariant(&layout_f, &stock_f, &parts_f);
}
