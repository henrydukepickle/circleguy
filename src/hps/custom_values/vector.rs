use hyperpuzzlescript::{Builtins, CustomValue, FullDiagnostic, TypeOf, hps_fns};

use crate::complex::{c64::C64, complex_circle::ComplexCircle, point::Point, vector::Vector};
use approx_collections::ApproxEq;

impl TypeOf for Vector {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Vector")
    }
}

impl CustomValue for Vector {
    fn type_name(&self) -> &'static str {
        "Vector"
    }

    fn clone_dyn(&self) -> hyperpuzzlescript::BoxDynValue {
        self.clone().into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_repr: bool) -> std::fmt::Result {
        if is_repr {
            write!(f, "Vector[{}, {}]", self.0.re, self.0.im)
        } else {
            write!(f, "[{}, {}]", self.0.re, self.0.im)
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

pub fn vector_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set_fns(hps_fns![
        fn vector(x: f64, y: f64) -> Vector {
            Vector(C64 { re: x, im: y })
        }
        fn mag(v: Vector) -> f64 {
            v.mag()
        }
    ])?;
    b.set_fns(hps_fns![
        ("+", |_, a: Vector, b: Vector| -> Vector { a + b }),
        ("-", |_, a: Vector, b: Vector| -> Vector { a - b }),
        ("~", |_, a: Vector| -> Vector { -a }),
        ("*", |_, a: f64, b: Vector| -> Vector { a * b })
    ])?;
    b.set_custom_ty::<Vector>()
}
