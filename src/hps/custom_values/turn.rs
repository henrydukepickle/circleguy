use std::f64::consts::PI;

use egui::Order;
use hyperpuzzlescript::{
    BUILTIN_SPAN, Builtins, CustomValue, FullDiagnostic, List, ListOf, TypeOf, hps_fns,
};

use crate::{
    complex::{
        complex_circle::{Contains, OrientedCircle},
        rotation::Rotation,
    },
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
    fn field_get(
        &self,
        _self_span: hyperpuzzlescript::Span,
        (field, _field_span): hyperpuzzlescript::Spanned<&str>,
    ) -> hyperpuzzlescript::Result<Option<hyperpuzzlescript::ValueData>> {
        Ok(match field {
            "circ" | "c" | "circle" => Some(
                OrientedCircle {
                    circ: self.turn.circle.into(),
                    ori: Contains::Inside,
                }
                .into(),
            ),
            "order" | "ord" | "num" => Some(self.order.into()),
            _ => None,
        })
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
        fn inverse(turn: OrderedTurn) -> OrderedTurn {
            turn.inverse()
        }
        fn mult(turn: OrderedTurn, mult: i64) -> OrderedTurn {
            turn.mult(mult as isize)
        }
        fn sym(ctx: EvalCtx, t: OrderedTurn) -> ListOf<OrderedTurn> {
            let mut orders = ListOf::new();
            for i in 0..(t.order) {
                orders.push((t.mult(i as isize), ctx.caller_span));
            }
            orders
        }
        fn inverse(ctx: EvalCtx, turns: ListOf<OrderedTurn>) -> ListOf<OrderedTurn> {
            flip_turn_seq(turns.into_iter().map(|x| x.0).collect())
                .into_iter()
                .map(|x| (x, ctx.caller_span))
                .collect()
        }
        fn mult(ctx: EvalCtx, turns: ListOf<OrderedTurn>, num: i64) -> ListOf<OrderedTurn> {
            mult_turn_seq(turns.into_iter().map(|x| x.0).collect(), num)
                .into_iter()
                .map(|x| (x, ctx.caller_span))
                .collect()
        }
    ])?;
    b.set_custom_ty::<OrderedTurn>()
}

fn flip_turn_seq(turn_seq: Vec<OrderedTurn>) -> Vec<OrderedTurn> {
    turn_seq.iter().rev().map(|x| x.inverse()).collect()
}

fn mult_turn_seq(turn_seq: Vec<OrderedTurn>, num: i64) -> Vec<OrderedTurn> {
    if num == 0 {
        Vec::new()
    } else if num > 0 {
        let mut turns = mult_turn_seq(turn_seq.clone(), num - 1);
        turns.extend(turn_seq);
        turns
    } else {
        mult_turn_seq(flip_turn_seq(turn_seq), -num)
    }
}
