use hyperpuzzlescript::{Builtins, CustomValue, FullDiagnostic, TypeOf, hps_fns};

use crate::complex::{
    c64::C64,
    complex_circle::{ComplexCircle, Contains, OrientedCircle},
    point::Point,
};

use approx_collections::ApproxEq;
impl TypeOf for OrientedCircle {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Circle")
    }
}
impl CustomValue for OrientedCircle {
    fn type_name(&self) -> &'static str {
        "Circle"
    }

    fn clone_dyn(&self) -> hyperpuzzlescript::BoxDynValue {
        (*self).into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_repr: bool) -> std::fmt::Result {
        if is_repr {
            write!(f, "hello repr")
        } else {
            write!(f, "hello display")
        }
    }

    fn eq(&self, other: &hyperpuzzlescript::BoxDynValue) -> Option<bool> {
        Some(self.approx_eq(other.downcast_ref()?, crate::PRECISION))
    }
    fn field_get(
        &self,
        _self_span: hyperpuzzlescript::Span,
        (field, _field_span): hyperpuzzlescript::Spanned<&str>,
    ) -> hyperpuzzlescript::Result<Option<hyperpuzzlescript::ValueData>> {
        Ok(match field {
            "c" | "cent" | "center" => Some(self.circ.center.into()),
            "r" | "rad" | "radius" => Some(self.circ.r().into()),
            _ => None,
        })
    }
}

pub fn circle_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
        fn circle(cx: f64, cy: f64, r: f64) -> OrientedCircle {
            OrientedCircle {
                circ: ComplexCircle {
                    center: Point(C64 { re: cx, im: cy }),
                    r_sq: r * r,
                },
                ori: Contains::Inside,
            }
        }
        fn circle(c: Point, r: f64) -> OrientedCircle {
            OrientedCircle {
                circ: ComplexCircle {
                    center: c,
                    r_sq: r * r,
                },
                ori: Contains::Inside,
            }
        }
        fn rotate(c: OrientedCircle, cent: Point, angle: f64) -> OrientedCircle {
            OrientedCircle {
                circ: c.circ.rotate_about(cent, angle),
                ori: c.ori,
            }
        }
    ])?;
    b.set_fns(hps_fns![("~", |_, a: OrientedCircle| -> OrientedCircle {
        -a
    })])?;
    b.set_custom_ty::<OrientedCircle>()
}
