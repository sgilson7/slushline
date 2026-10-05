//! A pilot returns an `Input` and nothing else (D14): it reads the world and
//! cannot change it, so it can do nothing a player cannot.

use pilot::{Kind, Pilot};
use sim::{Input, World};

#[test]
fn a_pilot_returns_an_input_and_nothing_else() {
    // `Pilot::input` takes `&World`, so the borrow checker already forbids a
    // change; this also checks the world is equal before and after, over a
    // whole mission, and that the input carries only spout bits.
    let m = content::missions::mission("m_three").unwrap();
    let mut w = World::new(m.setup(1, 1));
    for kind in [Kind::Idle, Kind::Timer, Kind::Yardstick { seed: 3, error: pilot::YARDSTICK_ERROR }] {
        let mut p = Pilot::new(kind);
        let mut held = 0;
        for _ in 0..1_500 {
            let before = w.clone();
            let i: Input = p.input(&w, 0);
            assert_eq!(w, before, "a pilot changed the world");
            assert_eq!(i.0 & !0x0F, 0, "a pilot set a bit that is not a spout");
            held += (i.0 != 0) as u32;
            w.step([i, Input::NONE]);
        }
        let _ = held;
    }
}

#[test]
fn the_idle_pilot_holds_nothing_and_the_timer_pours() {
    let m = content::missions::mission("m_first_pour").unwrap();
    let w = World::new(m.setup(1, 1));
    let mut idle = Pilot::new(Kind::Idle);
    let mut timer = Pilot::new(Kind::Timer);
    let (mut a, mut b) = (w.clone(), w);
    let (mut held_a, mut held_b) = (0, 0);
    for _ in 0..2_000 {
        let i = idle.input(&a, 0);
        held_a += (i.0 != 0) as u32;
        a.step([i, Input::NONE]);
        let j = timer.input(&b, 0);
        held_b += (j.0 != 0) as u32;
        b.step([j, Input::NONE]);
    }
    assert_eq!(held_a, 0);
    assert!(held_b > 100, "the timer held a key for {held_b} ticks");
}
