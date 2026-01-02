use std::{
    sync::{Arc, Mutex},
    usize,
};

use hyperpuzzlescript::{Builtins, CustomValue, Error, EvalCtx, FullDiagnostic, TypeOf, hps_fns};

use crate::{
    complex::complex_circle::OrientedCircle,
    hps::custom_values::hpspuzzledata::HPSPuzzleData,
    puzzle::{color::Color, turn::OrderedTurn},
};

// pub struct HPSPuzzleConstructor(Box<dyn FnMut(HPSPuzzle) -> ()>);

#[derive(Clone, Debug)]
pub struct HPSPuzzle(pub Arc<Mutex<HPSPuzzleData>>);

impl TypeOf for HPSPuzzle {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Puzzle")
    }
}

impl Default for HPSPuzzle {
    fn default() -> Self {
        Self::new()
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

    fn eq(&self, _other: &hyperpuzzlescript::BoxDynValue) -> Option<bool> {
        None
    }
}

pub fn puzzle_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
        fn add_circle(ctx: EvalCtx, disk: OrientedCircle) -> () {
            puzzle(ctx).add_disk(disk.circ);
        }
        fn add_circles(ctx: EvalCtx, disks: Vec<OrientedCircle>) -> () {
            let mut p = puzzle(ctx);
            for disk in disks {
                p.add_disk(disk.circ);
            }
        }
        fn add_turn(ctx: EvalCtx, turn: OrderedTurn, name: String) -> () {
            puzzle(ctx).turns.insert(name, turn);
        }
        fn add_turn(ctx: EvalCtx, turn: OrderedTurn) -> () {
            let s = ctx.caller_span;
            let mut p = puzzle(ctx);
            let name = p.next_turn_name().ok_or(
                Error::User("No more automatic names left. Please manually assign name!".into())
                    .at(s),
            )?;
            p.turns.insert(name, turn);
        }
        fn add_turns(ctx: EvalCtx, turns: Vec<OrderedTurn>, names: Vec<String>) -> () {
            let s = ctx.caller_span;
            let mut p = puzzle(ctx);
            if turns.len() != names.len() {
                return Err(Error::User("Inequal number of turns and names passed.".into()).at(s));
            }
            for i in 0..(turns.len()) {
                p.turns.insert(names[i].clone(), turns[i]);
            }
        }
        fn add_turns(ctx: EvalCtx, turns: Vec<OrderedTurn>) -> () {
            let s = ctx.caller_span;
            let mut p = puzzle(ctx);
            for t in turns {
                let name = p.next_turn_name().ok_or(
                    Error::User(
                        "No more automatic names left. Please manually assign name!".into(),
                    )
                    .at(s),
                )?;
                p.turns.insert(name, t);
            }
        }
        fn cut(ctx: EvalCtx, cut: Vec<OrderedTurn>) -> () {
            let s = ctx.caller_span;
            puzzle(ctx)
                .cut(&cut)
                .or(Err(Error::Internal("Internal error when cutting!").at(s)))?;
        }
        fn cut(ctx: EvalCtx, region: Vec<OrientedCircle>, cut: Vec<OrderedTurn>) -> () {
            let s = ctx.caller_span;
            puzzle(ctx)
                .cut_region(&region, &cut)
                .or(Err(Error::Internal("Internal error when cutting!").at(s)))?;
        }
        fn turn(ctx: EvalCtx, turns: Vec<OrderedTurn>) -> () {
            let s = ctx.caller_span;
            let mut p = puzzle(ctx);
            for t in turns {
                p.turn(t, true)
                    .or(Err(Error::Internal("Internal error when turning!").at(s)))?;
            }
        }
        fn turn(ctx: EvalCtx, turn: OrderedTurn) -> () {
            let s = ctx.caller_span;
            puzzle(ctx)
                .turn(turn, true)
                .or(Err(Error::Internal("Internal error when turning!").at(s)))?;
        }
        fn undo(ctx: EvalCtx) -> () {
            let s = ctx.caller_span;
            puzzle(ctx)
                .undo()
                .or(Err(Error::Internal("Internal error when undoing!").at(s)))?;
        }
        fn undo(ctx: EvalCtx, num: usize) -> () {
            let s = ctx.caller_span;
            puzzle(ctx)
                .undo_num(num)
                .or(Err(Error::Internal("Internal error when undoing!").at(s)))?;
        }
        fn undo_all(ctx: EvalCtx) -> () {
            let s = ctx.caller_span;
            puzzle(ctx)
                .undo_all()
                .or(Err(Error::Internal("Internal error when undoing!").at(s)))?;
        }
        fn color(ctx: EvalCtx, region: Vec<OrientedCircle>, color: Color) -> () {
            puzzle(ctx).color(&region, color);
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
