//! The same inputs give the same world, every time and on every machine
//! (D3, D4). The native half is here; the browser half is the gate's "native
//! checksum equals wasm checksum" check.

use content::setup::{line, order, setup_of};
use sim::rng::Rng;
use sim::{Input, World};

fn mission(seed: u64) -> World {
    let orders = (0..6).map(|k| if k % 2 == 0 { order(&[("cola", 1), ("lemon", 1)]) } else { order(&[("cherry", 1)]) }).collect();
    World::new(setup_of(seed, sim::balance::DEFAULT_TUNING, line(&["cola", "cherry", "lemon"], "regular", "steady", orders)))
}

fn random_input(r: &mut Rng) -> Input {
    Input(r.below(16) as u16)
}

#[test]
fn two_worlds_fed_the_same_inputs_agree_for_ten_thousand_ticks() {
    for seed in [1u64, 42] {
        let (mut a, mut b) = (mission(seed), mission(seed));
        let mut r = Rng::new(seed ^ 0x55);
        let mut held = Input::NONE;
        for t in 0..10_000 {
            // Inputs change every few ticks, as hands do.
            if t % 7 == 0 {
                held = random_input(&mut r);
            }
            a.step([held, Input::NONE]);
            b.step([held, Input::NONE]);
            if t % 500 == 0 {
                assert_eq!(a.checksum(), b.checksum(), "seed {seed}: the worlds parted by tick {t}");
            }
        }
        assert!(a.lines[0].as_ref().unwrap().emitted > 500, "the run poured too little to mean anything");
        assert_eq!(a, b, "seed {seed}: equal checksums but different worlds");
    }
}

#[test]
fn the_checksum_would_actually_catch_a_divergence() {
    let mut a = mission(3);
    for _ in 0..200 {
        a.step([Input(Input::SPOUT[0]), Input::NONE]);
    }
    let mut b = a.clone();
    assert_eq!(a.checksum(), b.checksum());
    b.units[5].p.x.0 += 1;
    assert_ne!(a.checksum(), b.checksum(), "one raw unit in one unit's place must change the checksum");
    let mut c = a.clone();
    c.lines[0].as_mut().unwrap().valves[0].acc.0 += 1;
    assert_ne!(a.checksum(), c.checksum(), "and so must one raw unit in a valve");
}
