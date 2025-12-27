use std::{
    fmt::Debug,
    ops::{Add, Mul, Neg, Sub},
};

use approx_collections::{ApproxEq, ApproxEqZero, ApproxHash};

use crate::PRECISION;

pub type Point = C64;
pub type Scalar = f64;

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
    pub fn mag_sq(&self) -> Scalar {
        (self.im * self.im) + (self.re * self.re)
    }
    pub fn dist_sq(&self, other: Self) -> Scalar {
        (*self - other).mag_sq()
    }
    pub fn mag(&self) -> Scalar {
        self.mag_sq().sqrt()
    }
    pub fn dist(&self, other: Self) -> Scalar {
        self.dist_sq(other).sqrt()
    }
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }
    pub fn normalize(&self) -> Option<Self> {
        let mag = self.mag();
        if mag.approx_eq_zero(PRECISION) {
            return None;
        }
        Some(Self {
            re: self.re / mag,
            im: self.im / mag,
        })
    }
    pub fn from_angle(angle: Scalar) -> Self {
        Self {
            re: angle.cos(),
            im: angle.sin(),
        }
    }
    pub fn rotate_about(&self, cent: Self, angle: Scalar) -> Self {
        let rot = Self::from_angle(angle);
        cent + (rot * (*self - cent))
    }
    pub fn angle(&self) -> Scalar {
        self.im.atan2(self.re)
    }
}
