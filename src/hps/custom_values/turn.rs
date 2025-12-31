use std::f64::consts::PI;

use hyperpuzzlescript::{
    BUILTIN_SPAN, Builtins, CustomValue, FullDiagnostic, List, ListOf, TypeOf, hps_fns,
};

use crate::{
    complex::{complex_circle::OrientedCircle, rotation::Rotation},
    puzzle::turn::{OrderedTurn, Turn},
};

impl TypeOf for OrderedTurn {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Turn")
    }
}
impl CustomValue for OrderedTurn {
    fn type_name(&self) -> &'static str {
        "Turn"
    }

    fn clone_dyn(&self) -> hyperpuzzlescript::BoxDynValue {
        self.clone().into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_repr: bool) -> std::fmt::Result {
        if is_repr {
            write!(f, "NOT DONE!")
        } else {
            write!(f, "UNIMPLEMENTED!")
        }
    }

    fn eq(&self, other: &hyperpuzzlescript::BoxDynValue) -> Option<bool> {
        None
    }
}

pub fn turn_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
        fn turn(c: OrientedCircle, num: i64) -> OrderedTurn {
            OrderedTurn {
                turn: Turn {
                    circle: c.circ,
                    rot: Rotation::from_angle((-2.0 * PI) / (num as f64)),
                },
                order: num as usize,
            }
        }
        fn orders(t: OrderedTurn) -> ListOf<OrderedTurn> {
            let mut orders = ListOf::new();
            for i in 0..(t.order) {
                //WRONG SPAN BUT IDK
                orders.push((t.mult(i as isize), BUILTIN_SPAN));
            }
            orders
        }
    ])?;
    b.set_custom_ty::<OrderedTurn>()
}
