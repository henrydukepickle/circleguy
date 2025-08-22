use crate::PRECISION;
use approx_collections::*;
use cga2d::*;
use std::cmp::Ordering;
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Contains {
    Inside,
    Outside,
    Border,
}
pub fn circ_border_inside_circ(c1: Blade3, c2: Blade3) -> Contains {
    if let Dipole::Real(_) = (c1 & c2).unpack_with_prec(PRECISION) {
        return Contains::Border;
    }
    for point in [NI, NO] {
        let val = !(point ^ (c1 & c2) ^ !c2);
        if contains_from_metric(val) != Contains::Border {
            return contains_from_metric(val);
        }
    }
    return Contains::Border;
}
pub fn circle_contains(circ: Blade3, point: Blade1) -> Contains {
    contains_from_metric_prec(
        !(circ.rescale_oriented() ^ point.rescale_oriented()),
        Precision::new_simple(16),
    )
}
pub fn circle_orientation_euclid(circ: Blade3) -> Contains {
    circle_contains(-circ, NI)
}
//CGA MAYBE NEEDS TESTING, I THINK I GOT IT?
//undefined when A aeq B
//point aeq to base is minimal
pub fn comp_points_on_circle(base: Blade1, a: Blade1, b: Blade1, circ: Blade3) -> Ordering {
    if a.approx_eq(&base, PRECISION) {
        return Ordering::Less;
    }
    if b.approx_eq(&base, PRECISION) {
        return Ordering::Greater;
    }
    (((base ^ b) ^ a) << circ).approx_cmp_zero(PRECISION)
}
//CGA NEEDS TESTING
//returns if c1 fully excludes c2 (if the inside of c2 is fully contained in c1)
//WRONG!!! straight up doesnt work lol the math is bad
//SHOULD BE WORKING BUT NOT SURE
pub fn circle_excludes(c1: Blade3, c2: Blade3) -> bool {
    circ_border_inside_circ(c1, c2) == Contains::Outside
        && circle_orientation_euclid(c1) == circle_orientation_euclid(c2)
}
//arbitrary that > 0.0 is inside
pub fn contains_from_metric(metric: f64) -> Contains {
    match metric.approx_cmp_zero(PRECISION) {
        Ordering::Greater => Contains::Inside,
        Ordering::Less => Contains::Outside,
        Ordering::Equal => Contains::Border,
    }
}

pub fn contains_from_metric_prec(metric: f64, prec: Precision) -> Contains {
    match metric.approx_cmp_zero(prec) {
        Ordering::Greater => Contains::Inside,
        Ordering::Less => Contains::Outside,
        Ordering::Equal => Contains::Border,
    }
}
