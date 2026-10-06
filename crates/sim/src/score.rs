//! The score: one rule for fill and shares (PLANNING-BRIEF 0.4; D11).
//!
//! ```text
//! score = ⌊ 100 / C · Σᵢ min(nᵢ, C·tᵢ/T) ⌋
//! ```
//!
//! A unit counts while its flavor is still under its share. Units past a
//! share, and units of a flavor the order did not ask for, take up room and
//! count for nothing. Integers throughout. *Rejected:* averaging a fill score
//! and a ratio score, under which a cup of one flavor scores 75 (H3).

use crate::balance::MAX_SCORE;
use crate::setup::Order;
use serde::{Deserialize, Serialize};

/// What cost a cup the most points, in units of room.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Cause {
    /// Nothing: the cup scored full marks.
    None,
    /// Room left empty.
    Empty,
    /// A flavor past its share.
    Over,
    /// A flavor the order did not ask for.
    Wrong,
    /// A layered order's slush out of its layer.
    Layer,
}

impl Cause {
    /// The copy key's last part: `results.cause.<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Cause::None => "none",
            Cause::Empty => "empty",
            Cause::Over => "over",
            Cause::Wrong => "wrong",
            Cause::Layer => "layer",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CupScore {
    pub score: u32,
    /// Units that counted.
    pub counted: u32,
    /// Units in the cup.
    pub total: u32,
    /// Room left empty, units past their share, units not ordered.
    pub empty: u32,
    pub over: u32,
    pub wrong: u32,
    /// In a layered order, units of an ordered flavor outside its layer.
    pub misplaced: u32,
    /// The largest of the three; ties go empty, then over, then wrong.
    pub cause: Cause,
    /// The flavor that went over its share most, or was not ordered most,
    /// for the result's sentence (`results.cup.over`, `.wrong`).
    pub worst: Option<u8>,
    /// Units in the cup as a percentage of its capacity, floored.
    pub fill_pct: u32,
}

/// Score one cup. `counts[f]` is how many units of flavor `f` it holds.
pub fn score(counts: &[u32], order: &Order, capacity: u32) -> CupScore {
    let c = capacity.max(1);
    let (mut counted, mut over, mut wrong, mut total) = (0u32, 0u32, 0u32, 0u32);
    let (mut worst_over, mut worst_wrong) = ((0u32, None), (0u32, None));
    for (f, &n) in counts.iter().enumerate() {
        total += n;
        let ordered = order.parts.iter().any(|p| p.0 as usize == f && p.1 > 0);
        if ordered {
            let share = order.share(f as u8, c);
            counted += n.min(share);
            let past = n.saturating_sub(share);
            over += past;
            if past > worst_over.0 {
                worst_over = (past, Some(f as u8));
            }
        } else {
            wrong += n;
            if n > worst_wrong.0 {
                worst_wrong = (n, Some(f as u8));
            }
        }
    }
    let empty = c.saturating_sub(total);
    let score = (MAX_SCORE as u64 * counted as u64 / c as u64) as u32;
    let cause = if score >= MAX_SCORE {
        Cause::None
    } else if empty >= over && empty >= wrong && empty > 0 {
        Cause::Empty
    } else if over >= wrong && over > 0 {
        Cause::Over
    } else if wrong > 0 {
        Cause::Wrong
    } else {
        Cause::Empty
    };
    let worst = match cause {
        Cause::Over => worst_over.1,
        Cause::Wrong => worst_wrong.1,
        _ => None,
    };
    CupScore { score, counted, total, empty, over, wrong, misplaced: 0, cause, worst, fill_pct: (100 * total as u64 / c as u64) as u32 }
}

/// A mission's score: the average of its cups' scores, floored (0.4).
pub fn average(scores: &[u32]) -> u32 {
    if scores.is_empty() {
        return 0;
    }
    (scores.iter().map(|&s| s as u64).sum::<u64>() / scores.len() as u64) as u32
}

/// A layered order (2026-10-06): the parts are layers from the bottom up,
/// each as tall as its share. `ranked` is the cup's units from the lowest
/// up. A unit counts when its flavor is the layer it sits in; one of an
/// ordered flavor in another layer is misplaced, one of a flavor not
/// ordered is wrong, and units past the capacity count for nothing.
///
/// Hand case: capacity 120, cola under lemon, 1 to 1. Sixty cola below
/// sixty lemon is 100; the same 120 units with the lemon underneath is 0;
/// thirty cola, then sixty lemon, then thirty cola is 25 + 25 = 50.
pub fn score_layered(counts: &[u32], ranked: &[u8], order: &Order, capacity: u32) -> CupScore {
    let c = capacity.max(1);
    let mut bands = Vec::new();
    for &(f, _) in &order.parts {
        bands.extend(std::iter::repeat_n(f, order.share(f, c) as usize));
    }
    let (mut counted, mut misplaced, mut wrong) = (0u32, 0u32, 0u32);
    let mut worst = vec![0u32; counts.len()];
    for (k, &f) in ranked.iter().enumerate().take(c as usize) {
        let ordered = order.parts.iter().any(|p| p.0 == f);
        if bands.get(k) == Some(&f) {
            counted += 1;
        } else if ordered {
            misplaced += 1;
            worst[f as usize] += 1;
        } else {
            wrong += 1;
        }
    }
    let total: u32 = counts.iter().sum();
    let over = total.saturating_sub(c);
    let empty = c.saturating_sub(total);
    let score = (MAX_SCORE as u64 * counted as u64 / c as u64) as u32;
    let cause = if score >= MAX_SCORE {
        Cause::None
    } else if misplaced >= empty && misplaced >= wrong && misplaced >= over && misplaced > 0 {
        Cause::Layer
    } else if empty >= over && empty >= wrong && empty > 0 {
        Cause::Empty
    } else if over >= wrong && over > 0 {
        Cause::Over
    } else if wrong > 0 {
        Cause::Wrong
    } else {
        Cause::Empty
    };
    let worst = (cause == Cause::Layer).then(|| worst.iter().enumerate().max_by_key(|w| *w.1).map(|w| w.0 as u8)).flatten();
    CupScore { score, counted, total, empty, over, wrong, misplaced, cause, worst, fill_pct: (100 * total as u64 / c as u64) as u32 }
}
