use cga2d::*;
#[derive(Debug, Clone, Copy)]
///stores a turn as a circle the turn is inside, and a rotoflector the turn applies
pub struct Turn {
    pub circle: Blade3,
    pub rotation: Rotoflector,
}
///take a rotoflector to a power (for the sake of compound turns)
fn rotor_pow(i: isize, rot: Rotoflector) -> Rotoflector {
    if i == 0 {
        Rotoflector::ident()
    } else if i > 0 {
        rotor_pow(i - 1, rot) * rot
    } else {
        rotor_pow(-i, rot).rev()
    }
}
///implement turn multiplication for convenience
auto_ops::impl_op!(*|a: isize, b: Turn| -> Turn {
    Turn {
        circle: b.circle,
        rotation: rotor_pow(a, b.rotation),
    }
});
impl Turn {
    ///the inverse of a turn
    pub fn inverse(&self) -> Turn {
        Turn {
            circle: self.circle,
            rotation: self.rotation.rev(),
        }
    }
}
