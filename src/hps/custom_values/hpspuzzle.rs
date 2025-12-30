use std::sync::{Arc, Mutex};

use hyperpuzzlescript::{CustomValue, TypeOf};

use crate::puzzle::puzzle::Puzzle;

#[derive(Clone, Debug)]
pub struct HPSPuzzleData {
    pub puzzle: Option<Puzzle>,
}

#[derive(Clone, Debug)]
pub struct HPSPuzzle(pub Arc<Mutex<HPSPuzzleData>>);

impl TypeOf for HPSPuzzle {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Puzzle")
    }
}

impl HPSPuzzle {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HPSPuzzleData { puzzle: None })))
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
