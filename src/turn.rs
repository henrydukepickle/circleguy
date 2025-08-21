use cga2d::*;
#[derive(Debug, Clone, Copy)]
pub struct Turn {
    pub circle: Blade3,
    pub rotation: Rotoflector,
}
fn rotor_pow(i: isize, rot: Rotoflector) -> Rotoflector {
    if i == 0 {
        return Rotoflector::ident();
    }
    if i > 0 {
        return rotor_pow(i - 1, rot) * rot;
    } else {
        return rotor_pow(-i, rot).rev();
    }
}
auto_ops::impl_op!(*|a: isize, b: Turn| -> Turn {
    Turn {
        circle: b.circle,
        rotation: rotor_pow(a, b.rotation),
    }
});
impl Turn {
    pub fn inverse(&self) -> Turn {
        Turn {
            circle: self.circle,
            rotation: self.rotation.rev(),
        }
    }
}
