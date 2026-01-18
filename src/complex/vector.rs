use std::ops::{Add, Mul, Neg, Sub};

use approx_collections::{ApproxEq, ApproxEqZero};

use crate::{
    PRECISION,
    complex::c64::{C64, Scalar},
};

#[derive(Clone, Debug, Copy)]
pub struct Vector(pub C64);

impl ApproxEq for Vector {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.0.approx_eq(&other.0, prec)
    }
}

impl Vector {
    pub fn mag_sq(&self) -> Scalar {
        self.0.mag_sq()
    }
    pub fn mag(&self) -> Scalar {
        self.0.mag()
    }
    ///normalization. if the number is approx 0, returns None
    pub fn normalize(&self) -> Option<Self> {
        let mag = self.mag();
        if mag.approx_eq_zero(PRECISION) {
            return None;
        }
        Some(Self((1. / mag) * self.0))
    }
    pub fn angle(&self) -> Scalar {
        self.0.angle()
    }
}

impl Mul<Vector> for Scalar {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector(self * rhs.0)
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}
