use std::{
    path::PathBuf,
    sync::Arc,
};

use hyperpuzzlescript::{
    Runtime,
    Scope,
};

use crate::{
    hps::{
        builtins,
        custom_values::hpspuzzle::HPSPuzzle,
    },
    puzzle::puzzle::Puzzle,
};

pub fn parse_hps(hps: &str, _rt: &mut Runtime) -> Option<Puzzle> {
    let mut rt = hyperpuzzlescript::Runtime::new();
    let puzzle = HPSPuzzle::new();
    let scope = Scope::default();
    rt.builtins = Arc::new(scope);
    // Add base built-ins.
    rt.with_builtins(hyperpuzzlescript::builtins::define_base_in)
        .expect("error defining HPS built-ins");
    rt.modules.add_file(&PathBuf::from("lol"), hps);
    rt.with_builtins(builtins::circleguy_builtins).unwrap();
    rt.exec_all_files();
    let data = puzzle.0.lock().unwrap().clone();
    // Some(Puzzle {
    //     name: String::new(),
    //     authors: Vec::new(),
    //     pieces: p.unwrap(),
    //     base_turns: HashMap::new(),
    //     turn_orders: HashMap::new(),
    //     anim_left: 0.0,
    //     scramble: None,
    //     animation_offset: None,
    //     stack: Vec::new(),
    //     intern: FloatPool::new(PRECISION),
    //     depth: 500,
    //     solved_state: None,
    //     solved: false,
    //     def: String::new(),
    // })
    Some(Puzzle::new(data.to_puzzle_data()))
}
