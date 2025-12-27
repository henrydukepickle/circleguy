use std::{cmp::Ordering, f64::consts::PI, ops::Neg};

use approx_collections::{ApproxEq, ApproxEqZero};

use crate::{PRECISION, complex::c64::Point, complex::c64::Scalar};

#[derive(PartialEq, Clone, Copy, Debug)]
///enum for whether one object (usually a circle) 'contains' another object (i.e., a point)
pub enum Contains {
    Inside,
    Outside,
    Border,
}

pub type Circle = ComplexCircle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexCircle {
    pub center: Point,
    pub r_sq: Scalar,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    CW,
    CCW,
}

impl ComplexCircle {
    pub fn rad(&self) -> Scalar {
        self.r_sq.sqrt()
    }
    pub fn contains(&self, point: Point) -> Contains {
        let d = self.center.dist_sq(point);
        if d.approx_eq(&self.r_sq, PRECISION) {
            Contains::Border
        } else if d < self.r_sq {
            Contains::Inside
        } else {
            Contains::Outside
        }
    }
    pub fn intersect_circle(&self, circ: Self) -> Vec<Point> {
        let d = self.center.dist(circ.center);
        if d.approx_eq_zero(PRECISION) {
            vec![]
        } else if d.approx_eq(&(self.rad() + circ.rad()), PRECISION) {
            vec![self.center + (self.rad() * (circ.center - self.center).normalize().unwrap())]
        } else if (d + self.rad()).approx_eq(&circ.rad(), PRECISION) {
            vec![self.center + (self.rad() * (self.center - circ.center).normalize().unwrap())]
        } else if self.rad().approx_eq(&(circ.rad() + d), PRECISION) {
            vec![self.center + (self.rad() * (circ.center - self.center).normalize().unwrap())]
        } else if (d > (self.rad() + circ.rad()))
            || (self.rad() > d + circ.rad())
            || (circ.rad() > d + self.rad())
        {
            vec![]
        } else {
            let angle = ((circ.r_sq - (self.r_sq + self.center.dist_sq(circ.center)))
                / (-2.0 * self.rad() * d))
                .acos();
            let point =
                self.center + (self.rad() * (circ.center - self.center).normalize().unwrap());
            vec![
                point.rotate_about(self.center, angle),
                point.rotate_about(self.center, -angle),
            ]
        }
    }
    pub fn rotate_about(&self, cent: Point, angle: Scalar) -> Self {
        Self {
            center: self.center.rotate_about(cent, angle),
            r_sq: self.r_sq,
        }
    }
    //points approx eq to start are minimal.
    pub fn comp_points_on_circle(
        &self,
        base: Point,
        a: Point,
        b: Point,
        ori: Orientation,
    ) -> Ordering {
        if a.approx_eq(&b, PRECISION) {
            return Ordering::Equal;
        }
        if a.approx_eq(&base, PRECISION) {
            return Ordering::Less;
        }
        if b.approx_eq(&base, PRECISION) {
            return Ordering::Greater;
        }
        let angle_a =
            ((a - self.center).angle() - (base - self.center).angle()).rem_euclid(2.0 * PI);
        let angle_b =
            ((b - self.center).angle() - (base - self.center).angle()).rem_euclid(2.0 * PI);
        if ori == Orientation::CCW {
            angle_a.total_cmp(&angle_b)
        } else {
            angle_b.total_cmp(&angle_a)
        }
    }
}

impl ApproxEq for ComplexCircle {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.center.approx_eq(&other.center, prec) && self.r_sq.approx_eq(&other.r_sq, prec)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientedCircle {
    pub circ: Circle,
    pub ori: Contains,
}

pub fn inside_bounds(bounds: &Vec<OrientedCircle>, point: Point) -> Contains {
    let mut border = false;
    for circ in bounds {
        let cont = circ.circ.contains(point);
        if cont == Contains::Border {
            border = true;
        } else if cont != circ.ori {
            return Contains::Outside;
        }
    }
    if border {
        Contains::Border
    } else {
        Contains::Inside
    }
}

impl OrientedCircle {
    pub fn contains(&self, point: Point) -> bool {
        self.circ.contains(point) == self.ori
    }
}

impl Neg for OrientedCircle {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            circ: self.circ,
            ori: match self.ori {
                Contains::Border => Contains::Border,
                Contains::Inside => Contains::Outside,
                Contains::Outside => Contains::Inside,
            },
        }
    }
}
