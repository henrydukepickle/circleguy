use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use approx_collections::{ApproxEq, FloatPool};
use hyperpuzzlescript::{
    BoxDynValue, CustomValue, EvalCtx, List, Scope, TypeOf, Value, ValueData, hps_fns,
};

use crate::{
    PRECISION,
    complex::{c64::C64, complex_circle::ComplexCircle},
    hps::{
        builtins,
        custom_values::{
            self,
            hpspuzzle::{HPSPuzzle, HPSPuzzleData},
        },
    },
    puzzle::puzzle::Puzzle,
    ui::puzzle_generation::make_basic_puzzle,
};

pub fn parse_hps(hps: &str) -> Option<Puzzle> {
    dbg!(hps);
    let mut rt = hyperpuzzlescript::Runtime::new();
    let mut scope = Scope::default();
    scope.special.puz = Value {
        data: HPSPuzzle::new().into(),
        span: 
    };
    rt.builtins = Arc::new(scope);
    let state = Arc::new(Mutex::new(Vec::new()));
    let state2 = state.clone();
    // Add base built-ins.
    rt.with_builtins(hyperpuzzlescript::builtins::define_base_in)
        .expect("error defining HPS built-ins");
    rt.modules.add_file(&PathBuf::from("lol"), hps);
    rt.with_builtins(builtins::circleguy_builtins).unwrap();
    rt.exec_all_files();
    let p = make_basic_puzzle((*state.lock().unwrap()).clone()).unwrap();
    Some(Puzzle {
        name: String::new(),
        authors: Vec::new(),
        pieces: p.unwrap(),
        base_turns: HashMap::new(),
        turn_orders: HashMap::new(),
        anim_left: 0.0,
        scramble: None,
        animation_offset: None,
        stack: Vec::new(),
        intern: FloatPool::new(PRECISION),
        depth: 500,
        solved_state: None,
        solved: false,
        def: String::new(),
    })
}
