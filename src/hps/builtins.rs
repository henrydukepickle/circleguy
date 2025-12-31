use hyperpuzzlescript::{Builtins, FullDiagnostic};

use crate::hps::custom_values::circle::circle_builtins;
use crate::hps::custom_values::color::color_builtins;
use crate::hps::custom_values::hpspuzzle::puzzle_builtins;
use crate::hps::custom_values::point::point_builtins;
use crate::hps::custom_values::turn::turn_builtins;
use crate::hps::custom_values::vector::vector_builtins;

pub fn circleguy_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    color_builtins(b)?;
    puzzle_builtins(b)?;
    vector_builtins(b)?;
    turn_builtins(b)?;
    point_builtins(b)?;
    circle_builtins(b)?;
    Ok(())
}
