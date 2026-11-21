//! Lays parts out on boards (one-dimensional cutting stock), the way a maker cuts them.

use std::cmp::Ordering;

/// One board on hand.
#[derive(Debug, Clone, PartialEq)]
pub struct Stock {
    /// The caller's label for the board.
    pub id: String,
    /// The board's length, in the caller's unit.
    pub length: f64,
}

/// A part to cut, `qty` times.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// The caller's label for the part.
    pub id: String,
    /// The length of one piece of this part.
    pub length: f64,
    /// How many pieces of this part are wanted.
    pub qty: u32,
}

/// What happens to one board.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardPlan {
    /// The `Stock::id` of the board.
    pub stock_id: String,
    /// The part ids cut from it, in cutting order from one end.
    pub cuts: Vec<String>,
    /// Length lost to the saw on this board.
    pub kerf_total: f64,
    /// What is left at the far end: length - sum of cuts - kerf_total.
    pub offcut: f64,
}

/// The whole layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// Only boards that receive at least one piece.
    pub boards: Vec<BoardPlan>,
    /// One part id per piece that could not be placed, in the order pieces were tried.
    pub unplaced: Vec<String>,
    /// Sum of the lengths of the placed pieces.
    pub parts_length: f64,
    /// Sum of the lengths of the boards in `boards`.
    pub boards_length: f64,
    /// All kerf, plus every offcut shorter than `usable_min`.
    pub true_waste: f64,
    /// Offcuts of at least `usable_min` (and above 0), longest first.
    pub usable_offcuts: Vec<f64>,
}

/// One piece to place: a part id and its length.
#[derive(Debug, Clone)]
struct Piece {
    id: String,
    length: f64,
}

/// A board that has been started while rule 3 runs.
struct OpenBoard {
    stock_index: usize,
    remaining: f64,
    cuts: Vec<String>,
    cuts_len: f64,
    kerf_total: f64,
}

/// Lays `parts` out on `stock` with a saw kerf of `kerf`; offcuts of at least `usable_min`
/// are worth keeping.
pub fn lay_out(stock: &[Stock], parts: &[Part], kerf: f64, usable_min: f64) -> Layout {
    // Rule 1: a piece whose length is not above 0 (NaN included) is ignored.
    let mut pieces: Vec<Piece> = Vec::new();
    for part in parts {
        if part.length > 0.0 {
            for _ in 0..part.qty {
                pieces.push(Piece {
                    id: part.id.clone(),
                    length: part.length,
                });
            }
        }
    }
    // Rule 3: longest first, stable, so equal lengths keep their order in `parts`.
    pieces.sort_by(|a, b| b.length.partial_cmp(&a.length).unwrap_or(Ordering::Equal));

    let mut open: Vec<OpenBoard> = Vec::new();
    let mut used: Vec<bool> = vec![false; stock.len()];
    let mut unplaced: Vec<String> = Vec::new();

    for piece in &pieces {
        let mut placed = false;
        // Rule 3: the first board already in use, in the order boards were started.
        for board in open.iter_mut() {
            if piece.length <= board.remaining + 1e-9 {
                let saw = kerf.min(board.remaining - piece.length).max(0.0);
                board.remaining = board.remaining - piece.length - saw;
                board.cuts_len += piece.length;
                board.kerf_total += saw;
                board.cuts.push(piece.id.clone());
                placed = true;
                break;
            }
        }
        if placed {
            continue;
        }
        // Rule 3: otherwise the first unused board in `stock` order that is long enough.
        for (index, board) in stock.iter().enumerate() {
            if used[index] {
                continue;
            }
            if piece.length <= board.length + 1e-9 {
                let saw = kerf.min(board.length - piece.length).max(0.0);
                used[index] = true;
                open.push(OpenBoard {
                    stock_index: index,
                    remaining: board.length - piece.length - saw,
                    cuts: vec![piece.id.clone()],
                    cuts_len: piece.length,
                    kerf_total: saw,
                });
                placed = true;
                break;
            }
        }
        if !placed {
            unplaced.push(piece.id.clone());
        }
    }

    // Rule 4: at most 12 pieces, all of them placed, and rule 3 used more than one board.
    if pieces.len() <= 12 && unplaced.is_empty() && open.len() > 1 {
        if let Some(solution) = search_fewer(&pieces, stock, kerf, open.len()) {
            let mut boards: Vec<BoardPlan> = Vec::new();
            let mut parts_length = 0.0;
            let mut boards_length = 0.0;
            for (stock_index, cut_indices, kerf_total) in solution {
                let length = stock.get(stock_index).map(|s| s.length).unwrap_or(0.0);
                let stock_id = stock
                    .get(stock_index)
                    .map(|s| s.id.clone())
                    .unwrap_or_default();
                let mut cuts: Vec<String> = Vec::new();
                let mut sum = 0.0;
                for &index in &cut_indices {
                    if let Some(piece) = pieces.get(index) {
                        cuts.push(piece.id.clone());
                        sum += piece.length;
                    }
                }
                parts_length += sum;
                boards_length += length;
                boards.push(BoardPlan {
                    stock_id,
                    cuts,
                    kerf_total,
                    offcut: length - sum - kerf_total,
                });
            }
            return finish(boards, unplaced, parts_length, boards_length, usable_min);
        }
    }

    let mut boards: Vec<BoardPlan> = Vec::new();
    let mut parts_length = 0.0;
    let mut boards_length = 0.0;
    for board in &open {
        let length = stock
            .get(board.stock_index)
            .map(|s| s.length)
            .unwrap_or(0.0);
        let stock_id = stock
            .get(board.stock_index)
            .map(|s| s.id.clone())
            .unwrap_or_default();
        parts_length += board.cuts_len;
        boards_length += length;
        boards.push(BoardPlan {
            stock_id,
            cuts: board.cuts.clone(),
            kerf_total: board.kerf_total,
            offcut: length - board.cuts_len - board.kerf_total,
        });
    }
    finish(boards, unplaced, parts_length, boards_length, usable_min)
}

/// Rule 5: fills in the totals of a layout from its boards.
fn finish(
    boards: Vec<BoardPlan>,
    unplaced: Vec<String>,
    parts_length: f64,
    boards_length: f64,
    usable_min: f64,
) -> Layout {
    let mut true_waste = 0.0;
    let mut usable_offcuts: Vec<f64> = Vec::new();
    for board in &boards {
        true_waste += board.kerf_total;
        if board.offcut < usable_min {
            true_waste += board.offcut;
        } else if board.offcut > 0.0 {
            usable_offcuts.push(board.offcut);
        }
    }
    usable_offcuts.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
    Layout {
        boards,
        unplaced,
        parts_length,
        boards_length,
        true_waste,
        usable_offcuts,
    }
}

/// The state of the rule 4 search: the pieces, the boards, and the best answer so far.
struct SearchCtx<'a> {
    pieces: &'a [Piece],
    stock: &'a [Stock],
    order: &'a [usize],
    suffix: &'a [f64],
    kerf: f64,
    best_n: usize,
    best: Option<Vec<(usize, Vec<usize>, f64)>>,
}

impl<'a> SearchCtx<'a> {
    /// Rule 4: places piece `i` into an open board or into one new board.
    fn dfs(
        &mut self,
        i: usize,
        rem: &mut Vec<f64>,
        ktot: &mut Vec<f64>,
        cuts: &mut Vec<Vec<usize>>,
    ) {
        if i == self.pieces.len() {
            if rem.len() < self.best_n {
                self.best_n = rem.len();
                let mut solution = Vec::with_capacity(rem.len());
                for b in 0..rem.len() {
                    let stock_index = self.order.get(b).copied().unwrap_or(0);
                    let kerf_total = ktot.get(b).copied().unwrap_or(0.0);
                    let board_cuts = cuts.get(b).cloned().unwrap_or_default();
                    solution.push((stock_index, board_cuts, kerf_total));
                }
                self.best = Some(solution);
            }
            return;
        }
        // Stop a branch as soon as it uses as many boards as the best found.
        if rem.len() >= self.best_n {
            return;
        }
        // Bound: what is still to place must fit in what is left plus the boards we may
        // still open before reaching `best_n` boards.
        let mut room = 0.0;
        for r in rem.iter() {
            room += *r;
        }
        let last = self.best_n.saturating_sub(1);
        for b in rem.len()..last {
            if let Some(&stock_index) = self.order.get(b) {
                if let Some(board) = self.stock.get(stock_index) {
                    room += board.length;
                }
            }
        }
        if self.suffix.get(i).copied().unwrap_or(0.0) > room + 1e-9 {
            return;
        }
        let length = self.pieces.get(i).map(|p| p.length).unwrap_or(0.0);
        let mut seen: Vec<(f64, f64, f64)> = Vec::new();
        for b in 0..rem.len() {
            let r = rem.get(b).copied().unwrap_or(0.0);
            if length > r + 1e-9 {
                continue;
            }
            let stock_index = self.order.get(b).copied().unwrap_or(0);
            let board_length = self.stock.get(stock_index).map(|s| s.length).unwrap_or(0.0);
            let spent = ktot.get(b).copied().unwrap_or(0.0);
            // Two boards in the same state are the same choice.
            let same = seen.iter().any(|&(l, k, rr)| {
                (l - board_length).abs() < 1e-9 && (k - spent).abs() < 1e-9 && (rr - r).abs() < 1e-9
            });
            if same {
                continue;
            }
            seen.push((board_length, spent, r));
            let saw = self.kerf.min(r - length).max(0.0);
            if let Some(slot) = rem.get_mut(b) {
                *slot = r - length - saw;
            }
            if let Some(slot) = ktot.get_mut(b) {
                *slot = spent + saw;
            }
            if let Some(slot) = cuts.get_mut(b) {
                slot.push(i);
            }
            self.dfs(i + 1, rem, ktot, cuts);
            if let Some(slot) = cuts.get_mut(b) {
                slot.pop();
            }
            if let Some(slot) = ktot.get_mut(b) {
                *slot = spent;
            }
            if let Some(slot) = rem.get_mut(b) {
                *slot = r;
            }
        }
        // Open one new board: the next longest one still unused, because a longer board
        // holds anything a shorter one does.
        if rem.len() + 1 < self.best_n {
            if let Some(&stock_index) = self.order.get(rem.len()) {
                let board_length = self.stock.get(stock_index).map(|s| s.length).unwrap_or(0.0);
                if length <= board_length + 1e-9 {
                    let saw = self.kerf.min(board_length - length).max(0.0);
                    rem.push(board_length - length - saw);
                    ktot.push(saw);
                    cuts.push(vec![i]);
                    self.dfs(i + 1, rem, ktot, cuts);
                    cuts.pop();
                    ktot.pop();
                    rem.pop();
                }
            }
        }
    }
}

/// Rule 4: the smallest number of boards that holds every piece, if it beats `best_n`.
fn search_fewer(
    pieces: &[Piece],
    stock: &[Stock],
    kerf: f64,
    best_n: usize,
) -> Option<Vec<(usize, Vec<usize>, f64)>> {
    let mut order: Vec<usize> = (0..stock.len()).collect();
    order.sort_by(|&a, &b| {
        let left = stock.get(a).map(|s| s.length).unwrap_or(0.0);
        let right = stock.get(b).map(|s| s.length).unwrap_or(0.0);
        right.partial_cmp(&left).unwrap_or(Ordering::Equal)
    });
    let mut suffix: Vec<f64> = vec![0.0; pieces.len() + 1];
    for i in (0..pieces.len()).rev() {
        let length = pieces.get(i).map(|p| p.length).unwrap_or(0.0);
        let next = suffix.get(i + 1).copied().unwrap_or(0.0);
        if let Some(slot) = suffix.get_mut(i) {
            *slot = next + length;
        }
    }
    let mut ctx = SearchCtx {
        pieces,
        stock,
        order: &order,
        suffix: &suffix,
        kerf,
        best_n,
        best: None,
    };
    let mut rem: Vec<f64> = Vec::new();
    let mut ktot: Vec<f64> = Vec::new();
    let mut cuts: Vec<Vec<usize>> = Vec::new();
    ctx.dfs(0, &mut rem, &mut ktot, &mut cuts);
    ctx.best
}
