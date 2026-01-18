use std::{cmp::Ordering, f64::consts::PI, ops::Neg};

use approx_collections::{ApproxEq, ApproxEqZero};

use crate::{
    PRECISION,
    complex::{
        c64::{C64, Scalar},
        point::Point,
        vector::Vector,
    },
};

#[derive(PartialEq, Clone, Copy, Debug)]
///enum for whether one object (usually a circle) 'contains' another object (i.e., a point)
pub enum Contains {
    Inside,
    Outside,
    Border,
}

pub type Circle = ComplexCircle;

#[derive(Debug, Clone, Copy)]
///circle implemented with complex numbers
pub struct ComplexCircle {
    pub center: Point,
    pub r_sq: Scalar,
}
#[derive(Debug, Clone, Copy, PartialEq)]
///orientation, used in arcs and some circle operations
pub enum Orientation {
    CW,
    CCW,
}

impl ComplexCircle {
    ///radius
    pub fn r(&self) -> Scalar {
        self.r_sq.sqrt()
    }
    ///see if a circle contains a point
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
    ///intersect a circle another circle. tangent points are included. circles that are approximately equal return no intersection points.
    pub fn intersect_circle(&self, circ: Self) -> Vec<Point> {
        let d = self.center.dist(circ.center);
        if d.approx_eq_zero(PRECISION) {
            vec![] //if the circles have the same center, they dont intersect
        } else if d.approx_eq(&(self.r() + circ.r()), PRECISION) {
            vec![self.center + (self.r() * (circ.center - self.center).normalize().unwrap())] //handle the three tangent cases
        } else if (d + self.r()).approx_eq(&circ.r(), PRECISION) {
            vec![self.center + (self.r() * (self.center - circ.center).normalize().unwrap())]
        } else if self.r().approx_eq(&(circ.r() + d), PRECISION) {
            vec![self.center + (self.r() * (circ.center - self.center).normalize().unwrap())]
        } else if (d > (self.r() + circ.r())) //handle the proper intersection case
            || (self.r() > d + circ.r())
            || (circ.r() > d + self.r())
        {
            vec![]
        } else {
            let angle = ((circ.r_sq - (self.r_sq + self.center.dist_sq(circ.center)))
                / (-2.0 * self.r() * d)) //find the angle of the intersection points, above the line between the circles' centers
                .acos(); //use the law of cosines
            let point = self.center + (self.r() * (circ.center - self.center).normalize().unwrap()); //get a point on the first circle, directly between the two centers
            vec![
                point.rotate_about(self.center, angle), //rotate it both ways by the above angle
                point.rotate_about(self.center, -angle),
            ]
        }
    }
    ///rotate a circle around a point according to an angle
    pub fn rotate_about(&self, cent: Point, angle: Scalar) -> Self {
        Self {
            center: self.center.rotate_about(cent, angle),
            r_sq: self.r_sq,
        }
    }
    ///compare two points on a circle, with respect to a base and an orientation.
    ///the 'lesser' one is the one first encountered when travelling along the circle, starting at base, according to the orientation
    ///points approx eq to base are minimal.
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
    pub fn right_point(&self) -> Point {
        self.center
            + Vector(C64 {
                re: self.r(),
                im: 0.0,
            })
    }
}

impl ApproxEq for ComplexCircle {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.center.approx_eq(&other.center, prec) && self.r_sq.approx_eq(&other.r_sq, prec)
    }
}

#[derive(Debug, Clone, Copy)]
///circle with an orientation, either Outside or Inside.
pub struct OrientedCircle {
    pub circ: Circle,
    pub ori: Contains,
}

///see if a point lies inside a bunch of oriented circles (a CCP representation)
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
    ///see if an oriented circle contains a point, properly.
    pub fn contains(&self, point: Point) -> bool {
        self.circ.contains(point) == self.ori
    }
}

///reverse the orientation
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

impl ApproxEq for OrientedCircle {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.circ.approx_eq(&other.circ, prec) && self.ori == other.ori
    }
}
