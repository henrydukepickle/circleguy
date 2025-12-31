use hyperpuzzlescript::{Builtins, CustomValue, FullDiagnostic, TypeOf, builtins};

use crate::puzzle::color::Color;

impl TypeOf for Color {
    fn hps_ty() -> hyperpuzzlescript::Type {
        hyperpuzzlescript::Type::Custom("Color")
    }
}
impl CustomValue for Color {
    fn type_name(&self) -> &'static str {
        "Color"
    }

    fn clone_dyn(&self) -> hyperpuzzlescript::BoxDynValue {
        self.clone().into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_repr: bool) -> std::fmt::Result {
        write!(f, "test")
    }

    fn eq(&self, other: &hyperpuzzlescript::BoxDynValue) -> Option<bool> {
        Some(*self == *other.downcast_ref()?)
    }
}

pub fn color_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set("red", Color::Red)?;
    b.set("green", Color::Green)?;
    b.set("blue", Color::Blue)
}
