use std::f64::consts::PI;

use approx_collections::FloatPool;
use cga2d::{Blade3, Dipole, Multivector, circle, point};

use crate::{puzzle_generation::load_puzzle_and_def_from_file, *};

pub fn get_circ() -> Blade3 {
    circle(point(0.0, 1.0), 1.0)
}

pub fn get_circ2() -> Blade3 {
    circle(point(1.0, 0.0), 1.0)
}

pub fn get_arc() -> arc::Arc {
    arc::Arc {
        circle: get_circ(),
        boundary: Some(get_circ() & get_circ2()),
    }
}

pub fn get_blade2() -> Blade2 {
    get_circ() & get_circ2()
}

pub fn get_circ3() -> Blade3 {
    circle(point(0.0, 0.0), 1.0)
}

#[test]
pub fn prec_test() {
    for a in get_arc().cut_by_circle(get_circ3()).unwrap() {
        for c in a {
            c.debug();
        }
    }
}

#[test]
pub fn prec_test2() {
    let mut arc = get_arc();
    for _ in 0..10000 {
        let [x, y] = match arc.boundary.unwrap().unpack_with_prec(PRECISION) {
            Dipole::Real(r) => r,
            _ => panic!("LOL"),
        };
        arc = arc::Arc {
            circle: arc.circle,
            boundary: Some(x.to_blade().rescale_oriented() ^ y.to_blade().rescale_oriented()),
        }
    }
    arc.debug();
}

#[test]
pub fn simple_buffer_test() {
    let mut puzzle = load_puzzle_and_def_from_file("Puzzles/Definitions/55stars.kdl").unwrap();
    let turns = ["A", "B"];
    for i in 0..500 {
        for turn in turns {
            if !puzzle.turn_id(turn, false, 1).unwrap() {
                panic!("");
            }
        }
    }
    // dbg!(puzzle.intern_3.len());
    // dbg!(&puzzle.intern_3);
    // puzzle.turn_id("A", false, 1);
    // dbg!(puzzle.intern_3.len());
    // for i in 0..10000 {
    //     if !puzzle.turn_id("A", false, 1).unwrap() || !puzzle.turn_id("B", false, 1).unwrap() {
    //         panic!("{i}");
    //     };
    //     if i % 100 == 0 {
    //         dbg!(puzzle.intern_2.len());
    //         dbg!(puzzle.intern_3.len());
    //     }
    // }
}

#[test]
pub fn rotate_test() {
    let rot = puzzle_generation::basic_turn(get_circ(), PI)
        .unwrap()
        .rotation;
    let mut blade2 = get_blade2();
    let mut flool = FloatPool::new(POOL_PRECISION);
    blade2 = blade2.rescale_oriented();
    dbg!(blade2);
    for _ in 0..1000 {
        blade2 = rot.sandwich(blade2);
        blade2 = blade2.rescale_oriented();
        flool.intern_in_place(&mut blade2);
    }
    dbg!(flool.len());
    dbg!(flool);
    dbg!(blade2);
}
