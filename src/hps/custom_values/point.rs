use hyperpuzzlescript::{Builtins, CustomValue, FullDiagnostic, TypeOf, hps_fns};

use crate::complex::{c64::C64, complex_circle::ComplexCircle, point::Point, vector::Vector};
use approx_collections::ApproxEq;

impl TypeOf for Point {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Point")
    }
}

impl CustomValue for Point {
    fn type_name(&self) -> &'static str {
        "Point"
    }

    fn clone_dyn(&self) -> hyperpuzzlescript::BoxDynValue {
        self.clone().into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_repr: bool) -> std::fmt::Result {
        if is_repr {
            write!(f, "Point({}, {})", self.0.re, self.0.im)
        } else {
            write!(f, "({}, {})", self.0.re, self.0.im)
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
            "x" | "0" | "re" => Some(self.0.re.into()),
            "y" | "1" | "im" => Some(self.0.im.into()),
            _ => None,
        })
    }
}

pub fn point_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
        fn point(x: f64, y: f64) -> Point {
            Point(C64 { re: x, im: y })
        }
        fn rotate(point: Point, cent: Point, angle: f64) -> Point {
            point.rotate_about(cent, angle)
        }
    ])?;
    b.set_fns(hps_fns![
        ("+", |_, a: Point, b: Vector| -> Point { a + b }),
        ("+", |_, a: Vector, b: Point| -> Point { b + a })
    ])?;
    b.set_fns(hps_fns![("-", |_, a: Point, b: Point| -> Vector { a - b })])?;
    b.set_custom_ty::<Point>()
}
