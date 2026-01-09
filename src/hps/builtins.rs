use std::sync::Arc;

use hyperpuzzlescript::{Builtins, Error, FnValue, FullDiagnostic, Runtime, hps_fns};

use crate::hps::custom_values::circle::circle_builtins;
use crate::hps::custom_values::color::color_builtins;
use crate::hps::custom_values::hpspuzzle::puzzle_builtins;
use crate::hps::custom_values::point::point_builtins;
use crate::hps::custom_values::turn::turn_builtins;
use crate::hps::custom_values::vector::vector_builtins;
use crate::hps::data_storer::data_storer::{PuzzleLoadingData, PuzzlesMap};
use hyperpuzzlescript::builtins::*;

pub fn circleguy_hps_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    assertions::define_in(b)?;
    collections::define_in(b)?;
    math::define_in(b)?;
    operators::define_in(b)?;
    output::define_in(b)?;
    strings::define_in(b)?;
    types::define_in(b)?;
    Ok(())
}

pub fn circleguy_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    color_builtins(b)?;
    puzzle_builtins(b)?;
    vector_builtins(b)?;
    turn_builtins(b)?;
    point_builtins(b)?;
    circle_builtins(b)?;
    Ok(())
}
pub fn loading_builtins(
    rt: &mut Runtime,
    puzzles: PuzzlesMap,
    exp: bool,
) -> Result<(), FullDiagnostic> {
    rt.with_builtins(|b| {
        b.set_fns(hps_fns![
            #[kwargs(name: String, authors: Vec<String>, scramble: usize = 500, (build, span): Arc<FnValue>, experimental: bool = false)]
            fn add_puzzle(ctx: EvalCtx) -> () {
                if !experimental || exp {
                    let mut p = puzzles.lock().unwrap();
                    if p.contains_key(&name) {
                        return Err(Error::User("Error: duplicate puzzle names!".into()).at(ctx.caller_span));
                    }
                    p.insert(
                        name.clone(),
                        PuzzleLoadingData {
                            name,
                            authors,
                            scramble: scramble as usize,
                            constructor: (build, span),
                        },
                    );
                }
            }
        ])
    })
}
