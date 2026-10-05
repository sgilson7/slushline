//! A key pulls a handle and does nothing else (CLAUDE.md; D8).

use content::setup::{line, order, setup_of};
use sim::{Input, World};

fn three_spouts() -> World {
    let l = line(&["cola", "cherry", "lemon"], "regular", "steady", vec![order(&[("cola", 1)]); 3]);
    World::new(setup_of(9, sim::balance::DEFAULT_TUNING, l))
}

#[test]
fn only_spout_bits_move_a_handle() {
    // Every bit but the four spout bits, held for two seconds: no handle
    // moves, nothing is poured, and the cups move exactly as with no input.
    let mut quiet = three_spouts();
    for _ in 0..120 {
        quiet.step([Input::NONE; 2]);
    }
    for bit in 4..16 {
        let mut w = three_spouts();
        for _ in 0..120 {
            w.step([Input(1 << bit), Input(1 << bit)]);
        }
        let ls = w.lines[0].as_ref().unwrap();
        assert!(ls.handles.iter().all(|h| h.opening().0 == 0), "bit {bit} moved a handle");
        assert_eq!(ls.emitted, 0, "bit {bit} poured slush");
        assert_eq!(w.lines, quiet.lines, "bit {bit} changed the line");
    }
}

#[test]
fn each_spout_bit_pulls_its_own_handle_and_no_other() {
    for i in 0..3 {
        let mut w = three_spouts();
        for _ in 0..30 {
            w.step([Input(Input::SPOUT[i]), Input::NONE]);
        }
        let ls = w.lines[0].as_ref().unwrap();
        for (k, h) in ls.handles.iter().enumerate() {
            assert_eq!(h.opening().0 > 0, k == i, "key {} and handle {}", i + 1, k + 1);
        }
    }
}

#[test]
fn no_key_moves_a_cup_or_the_belt() {
    let mut a = three_spouts();
    let mut b = three_spouts();
    for t in 0..300u32 {
        a.step([Input::NONE; 2]);
        b.step([Input((t % 16) as u16), Input::NONE]);
        let (la, lb) = (a.lines[0].as_ref().unwrap(), b.lines[0].as_ref().unwrap());
        assert_eq!(la.travel, lb.travel);
        assert!(la.cups.iter().zip(&lb.cups).all(|(x, y)| x.x == y.x), "a key moved a cup at tick {t}");
    }
}

#[test]
fn a_handle_opens_with_the_key_and_closes_after_it() {
    // D7: the valve's opening follows the handle, and the handle takes a
    // moment to close: the tail has no constant of its own.
    let mut w = three_spouts();
    let mut opening = Vec::new();
    for t in 0..60 {
        w.step([Input(if t < 30 { Input::SPOUT[0] } else { 0 }), Input::NONE]);
        opening.push(w.lines[0].as_ref().unwrap().handles[0].opening().0);
    }
    assert_eq!(opening[29], sim::fx::ONE.0, "fully open after half a second held");
    assert!(opening[1] < opening[5], "the handle opens over several ticks");
    assert!(opening[31] > 0, "the handle is still open just after the key is let go");
    assert_eq!(opening[59], 0, "and shut half a second later");
}
