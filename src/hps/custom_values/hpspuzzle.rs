use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    usize,
};

use approx_collections::FloatPool;
use egui::mutex::MutexGuard;
use hyperpuzzlescript::{Builtins, CustomValue, EvalCtx, FullDiagnostic, ListOf, TypeOf, hps_fns};

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
        fn add_circle(ctx: EvalCtx, disk: OrientedCircle) -> () {
            puzzle(ctx).add_disk(disk.circ);
        }
        fn add_circles(ctx: EvalCtx, disks: ListOf<OrientedCircle>) -> () {
            let mut p = puzzle(ctx);
            for disk in disks {
                p.add_disk(disk.0.circ);
            }
        }
        fn add_turn(ctx: EvalCtx, turn: OrderedTurn, name: String) -> () {
            puzzle(ctx).turns.insert(name, turn);
        }
        fn add_turn(ctx: EvalCtx, turn: OrderedTurn) -> () {
            let mut p = puzzle(ctx);
            let name = p.next_turn_name().unwrap();
            p.turns.insert(name, turn);
        }
        fn add_turns(ctx: EvalCtx, turns: ListOf<OrderedTurn>, names: ListOf<String>) -> () {
            let mut p = puzzle(ctx);
            for i in 0..(turns.len()) {
                p.turns.insert(names[i].0.clone(), turns[i].0);
            }
        }
        fn add_turns(ctx: EvalCtx, turns: ListOf<OrderedTurn>) -> () {
            let mut p = puzzle(ctx);
            for t in turns {
                let name = p.next_turn_name().unwrap();
                p.turns.insert(name, t.0);
            }
        }
        fn cut(ctx: EvalCtx, cut: ListOf<OrderedTurn>) -> () {
            puzzle(ctx).cut(&cut.iter().map(|x| x.0).collect());
        }
        fn turn(ctx: EvalCtx, turns: ListOf<OrderedTurn>) -> () {
            let mut p = puzzle(ctx);
            for t in turns {
                p.turn(t.0, true);
            }
        }
        fn turn(ctx: EvalCtx, turn: OrderedTurn) -> () {
            puzzle(ctx).turn(turn, true);
        }
        fn undo(ctx: EvalCtx) -> () {
            puzzle(ctx).undo();
        }
        fn undo(ctx: EvalCtx, num: usize) -> () {
            puzzle(ctx).undo_num(num);
        }
        fn undo_all(ctx: EvalCtx) -> () {
            puzzle(ctx).undo_all();
        }
        fn color(ctx: EvalCtx, region: ListOf<OrientedCircle>, color: Color) -> () {
            puzzle(ctx).color(&region.iter().map(|x| x.0).collect(), color);
        }
    ])
}

fn puzzle<'a>(ctx: &'a mut EvalCtx) -> std::sync::MutexGuard<'a, HPSPuzzleData> {
    ctx.scope
        .special
        .puz
        .as_ref::<HPSPuzzle>()
        .unwrap()
        .0
        .lock()
        .unwrap()
}
