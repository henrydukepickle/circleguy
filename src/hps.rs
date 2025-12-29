use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use approx_collections::ApproxEq;
use hyperpuzzlescript::{CustomValue, EvalCtx, Scope, TypeOf, hps_fns};

use crate::{
    PRECISION,
    complex::{c64::C64, complex_circle::ComplexCircle},
    puzzle::puzzle::Puzzle,
};
impl TypeOf for ComplexCircle {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Circle")
    }
}
impl CustomValue for ComplexCircle {
    fn type_name(&self) -> &'static str {
        "Circle"
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
        Some(self.approx_eq(other.downcast_ref()?, PRECISION))
    }
    fn field_get(
        &self,
        _self_span: hyperpuzzlescript::Span,
        (field, _field_span): hyperpuzzlescript::Spanned<&str>,
    ) -> hyperpuzzlescript::Result<Option<hyperpuzzlescript::ValueData>> {
        Ok(match field {
            "cx" => Some(self.center.re.into()),
            "cy" => Some(self.center.im.into()),
            "r" => Some(self.r().into()),
            _ => None,
        })
    }
}

pub fn parse_hps(hps: &str) -> Option<Puzzle> {
    dbg!(hps);
    let mut rt = hyperpuzzlescript::Runtime::new();
    let mut scope = Scope::default();
    scope.special.puz = rt.builtins = Arc::new(scope);
    let state = Arc::new(Mutex::new(5));
    let state2 = state.clone();
    // Add base built-ins.
    rt.with_builtins(hyperpuzzlescript::builtins::define_base_in)
        .expect("error defining HPS built-ins");
    rt.modules.add_file(&PathBuf::from("lol"), hps);
    rt.with_builtins(|b| {
        b.set_fns(hps_fns![
            fn my_function(arg1: Vec<i64>) -> () {
                println!("{arg1:?}");
            }
            fn circle(cx: f64, cy: f64, r: f64) -> ComplexCircle {
                ComplexCircle {
                    center: C64 { re: cx, im: cy },
                    r_sq: r * r,
                }
            }
            fn thing(ctx: EvalCtx) -> () {
                ctx.scope.special.puz
            }
        ])?;
        b.set_fns(hps_fns![(
            "+",
            |_, a: ComplexCircle, b: f64| -> ComplexCircle {
                ComplexCircle {
                    center: a.center,
                    r_sq: a.r_sq + b.abs(),
                }
            }
        )])?;
        b.set_custom_ty::<ComplexCircle>()
    })
    .unwrap();
    rt.exec_all_files();
    dbg!(*state.lock().unwrap());
    None
}
