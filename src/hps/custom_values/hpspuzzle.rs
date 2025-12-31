use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    usize,
};

use approx_collections::FloatPool;
use hyperpuzzlescript::{Builtins, CustomValue, FullDiagnostic, ListOf, TypeOf, hps_fns};

use crate::{
    complex::complex_circle::OrientedCircle,
    hps::custom_values::hpspuzzledata::HPSPuzzleData,
    puzzle::{
        color::Color,
        piece::Piece,
        puzzle::Puzzle,
        turn::{OrderedTurn, Turn},
    },
};

// pub struct HPSPuzzleConstructor(Box<dyn FnMut(HPSPuzzle) -> ()>);

#[derive(Clone, Debug)]
pub struct HPSPuzzle(pub Arc<Mutex<HPSPuzzleData>>);

impl TypeOf for HPSPuzzle {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Puzzle")
    }
}

impl HPSPuzzle {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HPSPuzzleData::new())))
    }
}

impl CustomValue for HPSPuzzle {
    fn type_name(&self) -> &'static str {
        "Puzzle"
    }

    fn clone_dyn(&self) -> hyperpuzzlescript::BoxDynValue {
        self.clone().into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_repr: bool) -> std::fmt::Result {
        if is_repr {
            write!(f, "hello repr")
        } else {
            write!(f, "hello display")
        }
    }

    fn eq(&self, other: &hyperpuzzlescript::BoxDynValue) -> Option<bool> {
        None
    }
}

pub fn puzzle_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
        fn add_circle(puz: HPSPuzzle, disk: OrientedCircle) -> () {
            puz.0.lock().unwrap().add_disk(disk.circ);
        }
        fn add_circles(puz: HPSPuzzle, disks: ListOf<OrientedCircle>) -> () {
            let mut p = puz.0.lock().unwrap();
            for disk in disks {
                p.add_disk(disk.0.circ);
            }
        }
        fn add_turn(puz: HPSPuzzle, turn: OrderedTurn, name: String) -> () {
            puz.0.lock().unwrap().turns.insert(name, turn);
        }
        fn add_turn(puz: HPSPuzzle, turn: OrderedTurn) -> () {
            let mut p = puz.0.lock().unwrap();
            let name = p.next_turn_name().unwrap();
            p.turns.insert(name, turn);
        }
        fn add_turns(puz: HPSPuzzle, turns: ListOf<OrderedTurn>, names: ListOf<String>) -> () {
            let mut p = puz.0.lock().unwrap();
            for i in 0..(turns.len()) {
                p.turns.insert(names[i].0.clone(), turns[i].0);
            }
        }
        fn add_turns(puz: HPSPuzzle, turns: ListOf<OrderedTurn>) -> () {
            let mut p = puz.0.lock().unwrap();
            for t in turns {
                let name = p.next_turn_name().unwrap();
                p.turns.insert(name, t.0);
            }
        }
        fn cut(puz: HPSPuzzle, cut: ListOf<OrderedTurn>) -> () {
            puz.0
                .lock()
                .unwrap()
                .cut(&cut.iter().map(|x| x.0).collect());
        }
        fn turn(puz: HPSPuzzle, turns: ListOf<OrderedTurn>) -> () {
            let mut p = puz.0.lock().unwrap();
            for t in turns {
                p.turn(t.0, true);
            }
        }
        fn turn(puz: HPSPuzzle, turn: OrderedTurn) -> () {
            puz.0.lock().unwrap().turn(turn, true);
        }
        fn undo(puz: HPSPuzzle) -> () {
            puz.0.lock().unwrap().undo();
        }
        fn undo(puz: HPSPuzzle, num: usize) -> () {
            puz.0.lock().unwrap().undo_num(num);
        }
        fn undo_all(puz: HPSPuzzle) -> () {
            puz.0.lock().unwrap().undo_all();
        }
        fn color(puz: HPSPuzzle, region: ListOf<OrientedCircle>, color: Color) -> () {
            puz.0
                .lock()
                .unwrap()
                .color(&region.iter().map(|x| x.0).collect(), color);
        }
        fn name(puz: HPSPuzzle, name: String) -> () {
            puz.0.lock().unwrap().name = name;
        }
        fn authors(puz: HPSPuzzle, authors: ListOf<String>) -> () {
            puz.0.lock().unwrap().authors = authors.iter().map(|x| x.0.clone()).collect();
        }
    ])
}
