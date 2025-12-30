use hyperpuzzlescript::{CustomValue, TypeOf};

use crate::complex::complex_circle::ComplexCircle;

use approx_collections::ApproxEq;
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
        Some(self.approx_eq(other.downcast_ref()?, crate::PRECISION))
    }
    fn field_get(
        &self,
        _self_span: hyperpuzzlescript::Span,
        (field, _field_span): hyperpuzzlescript::Spanned<&str>,
    ) -> hyperpuzzlescript::Result<Option<hyperpuzzlescript::ValueData>> {
        Ok(match field {
            "c" => Some(self.center.into()),
            "r" => Some(self.r().into()),
            _ => None,
        })
    }
}
