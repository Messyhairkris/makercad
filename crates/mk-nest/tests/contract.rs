//! Contract tests for mk_nest, written by Claude from crew/contracts/mk_cutstock.md (ATLAS-api-harness).
//! They were the hidden tests the Crew's module had to pass in campaign mk-core-1; the contract is
//! final, so they are public here. New rules get new hidden tests before they come here.

use mk_nest::{lay_out, BoardPlan, Layout, Part, Stock};

fn stock(n: usize, len: f64) -> Vec<Stock> {
    (1..=n)
        .map(|i| Stock {
            id: format!("b{i}"),
            length: len,
        })
        .collect()
}

fn part(id: &str, length: f64, qty: u32) -> Part {
    Part {
        id: id.into(),
        length,
        qty,
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// Every board obeys rule 2 and rule 6, and the totals obey rule 5.
fn check_valid(stock: &[Stock], parts: &[Part], kerf: f64, usable_min: f64, l: &Layout) {
    let piece_len = |id: &str| parts.iter().find(|p| p.id == id).map(|p| p.length).unwrap();
    let mut used_ids = Vec::new();
    let mut kerf_sum = 0.0;
    let mut short_offcuts = 0.0;
    for b in &l.boards {
        assert!(!b.cuts.is_empty(), "a listed board has no cuts: {b:?}");
        assert!(
            !used_ids.contains(&b.stock_id),
            "board {} used twice",
            b.stock_id
        );
        used_ids.push(b.stock_id.clone());
        let len = stock
            .iter()
            .find(|s| s.id == b.stock_id)
            .expect("known board")
            .length;
        let mut r = len;
        let mut k_total = 0.0;
        for c in &b.cuts {
            let p = piece_len(c);
            assert!(
                p <= r + 1e-9,
                "{c} ({p}) does not fit {r} left on {}",
                b.stock_id
            );
            let k = kerf.min(r - p).max(0.0);
            r = r - p - k;
            k_total += k;
        }
        assert!(
            close(b.kerf_total, k_total),
            "kerf on {}: {} vs {k_total}",
            b.stock_id,
            b.kerf_total
        );
        assert!(
            close(b.offcut, r),
            "offcut on {}: {} vs {r}",
            b.stock_id,
            b.offcut
        );
        kerf_sum += b.kerf_total;
        if b.offcut < usable_min {
            short_offcuts += b.offcut;
        }
    }
    let placed: f64 = l
        .boards
        .iter()
        .flat_map(|b| &b.cuts)
        .map(|c| piece_len(c))
        .sum();
    assert!(
        close(l.parts_length, placed),
        "parts_length {} vs {placed}",
        l.parts_length
    );
    let boards_len: f64 = used_ids
        .iter()
        .map(|id| stock.iter().find(|s| &s.id == id).unwrap().length)
        .sum();
    assert!(close(l.boards_length, boards_len));
    assert!(
        close(l.true_waste, kerf_sum + short_offcuts),
        "true_waste {}",
        l.true_waste
    );
    let mut want: Vec<f64> = l
        .boards
        .iter()
        .map(|b| b.offcut)
        .filter(|&o| o >= usable_min && o > 0.0)
        .collect();
    want.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(l.usable_offcuts.len(), want.len());
    for (a, b) in l.usable_offcuts.iter().zip(&want) {
        assert!(
            close(*a, *b),
            "usable_offcuts {:?} vs {want:?}",
            l.usable_offcuts
        );
    }
    let pieces: usize = parts
        .iter()
        .filter(|p| p.length > 0.0)
        .map(|p| p.qty as usize)
        .sum();
    let placed_count: usize = l.boards.iter().map(|b| b.cuts.len()).sum();
    assert_eq!(
        placed_count + l.unplaced.len(),
        pieces,
        "every piece placed or unplaced"
    );
}

/// Smallest number of boards of one length that hold every piece (brute force).
fn min_boards(len: f64, pieces: &[f64], kerf: f64) -> usize {
    fn go(i: usize, pieces: &[f64], rem: &mut Vec<f64>, kerf: f64, best: &mut usize, len: f64) {
        if rem.len() >= *best {
            return;
        }
        if i == pieces.len() {
            *best = rem.len();
            return;
        }
        let p = pieces[i];
        for j in 0..rem.len() {
            let r = rem[j];
            if p <= r + 1e-9 {
                rem[j] = r - p - kerf.min(r - p).max(0.0);
                go(i + 1, pieces, rem, kerf, best, len);
                rem[j] = r;
            }
        }
        if p <= len + 1e-9 {
            rem.push(len - p - kerf.min(len - p).max(0.0));
            go(i + 1, pieces, rem, kerf, best, len);
            rem.pop();
        }
    }
    let mut sorted = pieces.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let mut best = usize::MAX;
    go(0, &sorted, &mut Vec::new(), kerf, &mut best, len);
    best
}

fn plan(id: &str, cuts: &[&str], kerf_total: f64, offcut: f64) -> BoardPlan {
    BoardPlan {
        stock_id: id.into(),
        cuts: cuts.iter().map(|c| c.to_string()).collect(),
        kerf_total,
        offcut,
    }
}

fn same_plans(got: &[BoardPlan], want: &[BoardPlan]) {
    assert_eq!(got.len(), want.len(), "boards: {got:?}");
    for (g, w) in got.iter().zip(want) {
        assert_eq!((&g.stock_id, &g.cuts), (&w.stock_id, &w.cuts), "{got:?}");
        assert!(
            close(g.kerf_total, w.kerf_total) && close(g.offcut, w.offcut),
            "{g:?} vs {w:?}"
        );
    }
}

#[test]
fn example_a_the_fifteen_boards() {
    let s = stock(15, 36.0);
    let p = vec![
        part("post", 33.0, 4),
        part("rail", 15.0, 8),
        part("side", 12.0, 6),
    ];
    let l = lay_out(&s, &p, 0.125, 6.0);
    check_valid(&s, &p, 0.125, 6.0, &l);
    let mut want = Vec::new();
    for i in 1..=4 {
        want.push(plan(&format!("b{i}"), &["post"], 0.125, 2.875));
    }
    for i in 5..=8 {
        want.push(plan(&format!("b{i}"), &["rail", "rail"], 0.25, 5.75));
    }
    for i in 9..=11 {
        want.push(plan(&format!("b{i}"), &["side", "side"], 0.25, 11.75));
    }
    same_plans(&l.boards, &want);
    assert!(l.unplaced.is_empty());
    assert!(close(l.parts_length, 324.0));
    assert!(close(l.boards_length, 396.0));
    assert!(close(l.true_waste, 36.75));
    assert_eq!(l.usable_offcuts.len(), 3);
    assert!(l.usable_offcuts.iter().all(|&o| close(o, 11.75)));
}

#[test]
fn example_b_a_piece_longer_than_every_board_is_unplaced() {
    let s = vec![
        Stock {
            id: "a".into(),
            length: 24.0,
        },
        Stock {
            id: "b".into(),
            length: 48.0,
        },
    ];
    let p = vec![part("x", 60.0, 1), part("y", 30.0, 1)];
    let l = lay_out(&s, &p, 0.0, 6.0);
    check_valid(&s, &p, 0.0, 6.0, &l);
    assert_eq!(l.unplaced, vec!["x".to_string()]);
    same_plans(&l.boards, &[plan("b", &["y"], 0.0, 18.0)]);
}

#[test]
fn example_c_a_piece_that_uses_the_board_up_takes_no_kerf() {
    let s = vec![Stock {
        id: "a".into(),
        length: 30.0,
    }];
    let p = vec![part("p", 30.0, 1)];
    let l = lay_out(&s, &p, 0.125, 6.0);
    same_plans(&l.boards, &[plan("a", &["p"], 0.0, 0.0)]);
    assert!(close(l.true_waste, 0.0));
}

#[test]
fn example_d_the_exact_search_beats_first_fit() {
    let s = stock(4, 10.0);
    let p = vec![part("a", 4.0, 2), part("b", 3.0, 4)];
    let l = lay_out(&s, &p, 0.0, 6.0);
    check_valid(&s, &p, 0.0, 6.0, &l);
    assert_eq!(l.boards.len(), 2, "{:?}", l.boards);
    assert!(l.unplaced.is_empty());
    assert!(close(l.true_waste, 0.0));
}

#[test]
fn example_e_stock_runs_out() {
    let s = vec![Stock {
        id: "a".into(),
        length: 10.0,
    }];
    let p = vec![part("p", 6.0, 3)];
    let l = lay_out(&s, &p, 0.0, 6.0);
    check_valid(&s, &p, 0.0, 6.0, &l);
    same_plans(&l.boards, &[plan("a", &["p"], 0.0, 4.0)]);
    assert_eq!(l.unplaced, vec!["p".to_string(), "p".to_string()]);
}

#[test]
fn example_f_kerf_eats_into_the_last_gap() {
    let s = vec![Stock {
        id: "a".into(),
        length: 10.0,
    }];
    let p = vec![part("p", 4.95, 2)];
    let l = lay_out(&s, &p, 0.125, 6.0);
    check_valid(&s, &p, 0.125, 6.0, &l);
    assert_eq!(l.boards.len(), 1);
    assert_eq!(l.unplaced, vec!["p".to_string()]);
}

#[test]
fn first_fit_starts_the_first_unused_board_in_stock_order_that_is_long_enough() {
    let s = vec![
        Stock {
            id: "short".into(),
            length: 10.0,
        },
        Stock {
            id: "long1".into(),
            length: 40.0,
        },
        Stock {
            id: "long2".into(),
            length: 40.0,
        },
    ];
    // 13 pieces: more than 12, so first-fit decreasing exactly.
    let p = vec![part("big", 30.0, 1), part("small", 2.0, 12)];
    let l = lay_out(&s, &p, 0.0, 1.0);
    check_valid(&s, &p, 0.0, 1.0, &l);
    assert_eq!(l.boards[0].stock_id, "long1");
    assert_eq!(l.boards[0].cuts[0], "big");
    // the small pieces fill long1's remaining 10 first (5 pieces), then start "short"
    assert_eq!(l.boards[0].cuts.len(), 6);
    assert_eq!(l.boards[1].stock_id, "short");
    assert_eq!(l.boards[1].cuts.len(), 5);
    assert_eq!(l.boards[2].stock_id, "long2");
    assert_eq!(l.boards[2].cuts.len(), 2);
}

#[test]
fn equal_lengths_keep_the_order_of_the_parts_list() {
    let s = stock(3, 100.0);
    let p = vec![part("first", 20.0, 7), part("second", 20.0, 7)];
    let l = lay_out(&s, &p, 0.0, 1.0);
    let order: Vec<&String> = l.boards.iter().flat_map(|b| &b.cuts).collect();
    let firsts = order.iter().take(7).all(|c| c.as_str() == "first");
    assert!(firsts, "{order:?}");
}

#[test]
fn zero_and_negative_parts_are_ignored() {
    let s = stock(2, 10.0);
    let p = vec![
        part("ok", 5.0, 1),
        part("zero", 0.0, 3),
        part("neg", -2.0, 2),
    ];
    let l = lay_out(&s, &p, 0.0, 1.0);
    assert!(l.unplaced.is_empty());
    let all: Vec<&String> = l.boards.iter().flat_map(|b| &b.cuts).collect();
    assert_eq!(all, vec!["ok"]);
}

#[test]
fn nothing_to_cut_uses_no_boards() {
    let l = lay_out(&stock(3, 10.0), &[], 0.125, 1.0);
    assert!(l.boards.is_empty() && l.unplaced.is_empty());
    assert!(close(l.parts_length, 0.0) && close(l.boards_length, 0.0) && close(l.true_waste, 0.0));
}

#[test]
fn small_jobs_use_the_fewest_boards_possible() {
    // deterministic pseudo-random cases; every one checked against brute force
    let mut seed: u64 = 0x2545F4914F6CDD1D;
    let mut next = |m: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % m
    };
    for case in 0..40 {
        let n_parts = 2 + next(4) as usize;
        let mut parts = Vec::new();
        let mut lens = Vec::new();
        let mut total = 0usize;
        for k in 0..n_parts {
            let len = 1.0 + next(9) as f64;
            let qty = 1 + next(3) as u32;
            if total + qty as usize > 10 {
                break;
            }
            total += qty as usize;
            parts.push(part(&format!("p{k}"), len, qty));
            for _ in 0..qty {
                lens.push(len);
            }
        }
        let kerf = if case % 2 == 0 { 0.0 } else { 0.25 };
        let s = stock(12, 10.0);
        let l = lay_out(&s, &parts, kerf, 1.0);
        check_valid(&s, &parts, kerf, 1.0, &l);
        assert!(l.unplaced.is_empty(), "case {case}: {parts:?}");
        let best = min_boards(10.0, &lens, kerf);
        assert_eq!(l.boards.len(), best, "case {case}: {parts:?} kerf {kerf}");
    }
}

#[test]
fn twelve_pieces_finish_quickly() {
    let s = stock(12, 100.0);
    let p = vec![
        part("a", 46.0, 3),
        part("b", 31.0, 3),
        part("c", 23.0, 3),
        part("d", 17.0, 3),
    ];
    let t = std::time::Instant::now();
    let l = lay_out(&s, &p, 0.5, 5.0);
    assert!(t.elapsed().as_millis() < 2000, "took {:?}", t.elapsed());
    check_valid(&s, &p, 0.5, 5.0, &l);
    let lens: Vec<f64> = p
        .iter()
        .flat_map(|x| vec![x.length; x.qty as usize])
        .collect();
    assert_eq!(l.boards.len(), min_boards(100.0, &lens, 0.5));
}

#[test]
fn nan_lengths_fit_nothing_and_never_crash() {
    let s = vec![
        Stock {
            id: "a".into(),
            length: 10.0,
        },
        Stock {
            id: "n".into(),
            length: f64::NAN,
        },
    ];
    let p = vec![part("x", 5.0, 1), part("bad", f64::NAN, 1)];
    let l = lay_out(&s, &p, 0.0, 1.0);
    assert!(l.boards.iter().all(|b| b.stock_id == "a"));
}

#[test]
fn the_exact_search_counts_kerf_like_first_fit_does() {
    // Example D on 10.5 boards with a 0.25 kerf: first fit needs 3 boards, the search
    // finds {4,3,3} twice, where the last 3 uses the board up and takes no kerf.
    // A piece as long as a board takes no kerf either, also when the search opens a board.
    let s = stock(5, 10.5);
    let p = vec![part("c", 10.5, 1), part("a", 4.0, 2), part("b", 3.0, 4)];
    let l = lay_out(&s, &p, 0.25, 6.0);
    check_valid(&s, &p, 0.25, 6.0, &l);
    assert_eq!(l.boards.len(), 3, "{:?}", l.boards);
    for b in &l.boards {
        let want = if b.cuts == ["c"] { 0.0 } else { 0.5 };
        assert!(close(b.kerf_total, want), "{b:?}");
        assert!(close(b.offcut, 0.0), "{b:?}");
    }
    assert!(close(l.true_waste, 1.0));
}

#[test]
fn the_exact_search_may_need_the_first_board() {
    // Only b1 + b2 (10 + 10) can hold 20 in two boards; first fit spills onto b3.
    let s = vec![
        Stock {
            id: "b1".into(),
            length: 10.0,
        },
        Stock {
            id: "b2".into(),
            length: 10.0,
        },
        Stock {
            id: "b3".into(),
            length: 7.0,
        },
    ];
    let p = vec![
        part("p5", 5.0, 1),
        part("p4", 4.0, 1),
        part("p3", 3.0, 3),
        part("p2", 2.0, 1),
    ];
    let l = lay_out(&s, &p, 0.0, 1.0);
    check_valid(&s, &p, 0.0, 1.0, &l);
    assert_eq!(l.boards.len(), 2, "{:?}", l.boards);
    let mut ids: Vec<&str> = l.boards.iter().map(|b| b.stock_id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, ["b1", "b2"]);
}
