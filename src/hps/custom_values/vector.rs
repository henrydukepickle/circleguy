use hyperpuzzlescript::{CustomValue, TypeOf};

use crate::complex::{complex_circle::ComplexCircle, point::Point, vector::Vector};
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
            "x" => Some(self.0.re.into()),
            "y" => Some(self.0.im.into()),
            _ => None,
        })
    }
}
