use std::{
    fmt::Debug,
    ops::{Add, Mul, Neg, Sub},
};

use approx_collections::{ApproxEq, ApproxEqZero};

use crate::PRECISION;

pub type Scalar = f64;

///complex number with f64 components. used for points and rotations
#[derive(Copy, Clone, PartialEq)]
pub struct C64 {
    pub re: f64,
    pub im: f64,
}

impl Debug for C64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} + {}i", self.re, self.im))
    }
}

impl Add for C64 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl Mul for C64 {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self {
            re: (self.re * other.re) - (self.im * other.im),
            im: (self.im * other.re) + (self.re * other.im),
        }
    }
}

impl Mul<C64> for f64 {
    type Output = C64;
    fn mul(self, rhs: C64) -> Self::Output {
        C64 {
            re: rhs.re * self,
            im: rhs.im * self,
        }
    }
}

impl Neg for C64 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl Sub for C64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl ApproxEq for C64 {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.re.approx_eq(&other.re, prec) && self.im.approx_eq(&other.im, prec)
    }
}

impl C64 {
    ///magnitude squared
    pub fn mag_sq(&self) -> Scalar {
        (self.im * self.im) + (self.re * self.re)
    }
    ///magnitude
    pub fn mag(&self) -> Scalar {
        self.mag_sq().sqrt()
    }
    ///complex conjugate
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }
    ///constructs a magnitude 1 complex number from an angle. 0 rad is 1 + 0i
    pub fn from_angle(angle: Scalar) -> Self {
        Self {
            re: angle.cos(),
            im: angle.sin(),
        }
    }
    ///get the angle of a complex number
    pub fn angle(&self) -> Scalar {
        self.im.atan2(self.re)
    }
}
