use hyperpuzzlescript::{Builtins, FullDiagnostic, hps_fns};

use crate::complex::point::Point;
use crate::complex::{c64::C64, complex_circle::ComplexCircle};
use crate::hps::custom_values::circle;

pub fn circleguy_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
            fn circle(cx: f64, cy: f64, r: f64) -> ComplexCircle {
                ComplexCircle {
                    center: Point(C64 { re: cx, im: cy }),
                    r_sq: r * r,
                }
            }
            // fn set(circs: Vec<ComplexCircle>) -> () {
            //     (*state2.lock().unwrap()) = circs.into();
            // }
        ])?;
    // b.set_fns(hps_fns![(
    //     "+",
    //     |_, a: ComplexCircle, b: f64| -> ComplexCircle {
    //         ComplexCircle {
    //             center: a.center,
    //             r_sq: a.r_sq + b.abs(),
    //         }
    //     }
    // )])?;
    b.set_custom_ty::<ComplexCircle>().unwrap();
    Ok(())
}
