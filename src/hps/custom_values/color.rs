use hyperpuzzlescript::{Builtins, CustomValue, FullDiagnostic, TypeOf};

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
        (*self).into()
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _is_repr: bool) -> std::fmt::Result {
        write!(f, "test")
    }

    fn eq(&self, other: &hyperpuzzlescript::BoxDynValue) -> Option<bool> {
        Some(*self == *other.downcast_ref()?)
    }
}

pub fn color_builtins(b: &mut Builtins) -> Result<(), FullDiagnostic> {
    b.set("red", Color::Red)?;
    b.set("green", Color::Green)?;
    b.set("blue", Color::Blue)?;
    b.set("yellow", Color::Yellow)?;
    b.set("purple", Color::Purple)?;
    b.set("gray", Color::Gray)?;
    b.set("black", Color::Black)?;
    b.set("brown", Color::Brown)?;
    b.set("cyan", Color::Cyan)?;
    b.set("white", Color::White)?;
    b.set("dark_blue", Color::DarkBlue)?;
    b.set("dark_green", Color::DarkGreen)?;
    b.set("dark_gray", Color::DarkRed)?;
    b.set("dark_red", Color::DarkRed)?;
    b.set("light_blue", Color::LightBlue)?;
    b.set("light_gray", Color::LightGray)?;
    b.set("light_green", Color::LightGreen)?;
    b.set("light_red", Color::LightRed)?;
    b.set("light_yellow", Color::LightYellow)?;
    b.set("khaki", Color::Khaki)?;
    b.set("gold", Color::Gold)?;
    b.set("magenta", Color::Magenta)?;
    b.set("orange", Color::Orange)?;
    Ok(())
}
