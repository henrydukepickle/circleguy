use std::ops::Mul;

use approx_collections::ApproxEq;

use crate::complex::{
    c64::{C64, Scalar},
    point::Point,
    vector::Vector,
};
#[derive(Clone, Debug, Copy)]
pub struct Rotation(pub C64); //rotation around the origin.

impl Rotation {
    ///constructs a rotation from an angle
    pub fn from_angle(angle: Scalar) -> Self {
        Self(C64::from_angle(angle))
    }
    ///complex conjugate the rotation
    pub fn conj(&self) -> Self {
        Self(self.0.conj())
    }
    pub fn angle(&self) -> Scalar {
        self.0.angle()
    }
}

impl ApproxEq for Rotation {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.0.approx_eq(&other.0, prec)
    }
}

impl Mul<Point> for Rotation {
    type Output = Point;
    fn mul(self, rhs: Point) -> Self::Output {
        Point(self.0 * rhs.0)
    }
}

impl Mul<Vector> for Rotation {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Self::Output {
        Vector(self.0 * rhs.0)
    }
}
