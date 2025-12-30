use std::ops::{Add, Sub};

use approx_collections::ApproxEq;

use crate::complex::{
    c64::{C64, Scalar},
    rotation::Rotation,
    vector::Vector,
};

#[derive(Clone, Debug, Copy)]
pub struct Point(pub C64);

impl Point {
    ///rotate the point around a center according to a given angle
    pub fn rotate_about(&self, cent: Self, angle: Scalar) -> Self {
        let rot = Rotation::from_angle(angle);
        cent + (rot * (*self - cent)) //use complex multiplication
    }
    ///distance to another point, squared
    pub fn dist_sq(&self, other: Self) -> Scalar {
        (*self - other).mag_sq()
    }
    ///distance to another point
    pub fn dist(&self, other: Self) -> Scalar {
        (*self - other).mag()
    }
}
impl ApproxEq for Point {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.0.approx_eq(&other.0, prec)
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector(self.0 - rhs.0)
    }
}

impl Add<Vector> for Point {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
