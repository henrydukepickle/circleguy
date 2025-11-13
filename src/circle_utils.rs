use crate::{LOW_PRECISION, PRECISION};
use approx_collections::*;
use cga2d::*;
use std::cmp::Ordering;
#[derive(PartialEq, Clone, Copy, Debug)]
///enum for whether one object (usually a circle) 'contains' another object (i.e., a point)
pub enum Contains {
    Inside,
    Outside,
    Border,
}
///given two circles, tells whether the border of c2 lies inside the border of c1. if they intersect, returns Border.
///aeq circles will return Border
pub fn circ_border_inside_circ(c1: Blade3, c2: Blade3) -> Contains {
    if let Dipole::Real(_) = (c1 & c2).unpack_with_prec(PRECISION) {
        //if they intersect, return border
        return Contains::Border;
    }
    for point in [NI, NO] {
        //we need two arbitrary points here
        let val = !(point ^ (c1 & c2) ^ !c2); //this is a bit involved, theres a diagram at https://enkimute.github.io/ganja.js/examples/coffeeshop.html#bYku6UFbSK
        if contains_from_metric(val) != Contains::Border {
            //if the test yields a 0 value, it is inconclusive and we use the second arbitrary point. otherwise we return
            return contains_from_metric(val);
        }
    }
    return Contains::Border; //if both tests are inconclusive, we default to border
}
///check if a circle contains a point, using CGA
///
///depends on the orientation of both the point and the circle
pub fn circle_contains(circ: Blade3, point: Blade1) -> Contains {
    contains_from_metric_prec(
        //wedge the point with the circle and take the sign of the dual
        !(circ.rescale_oriented() ^ point.rescale_oriented()),
        LOW_PRECISION,
    )
}
///gives the euclidian orientation of a circle -- arbitrarily chosen that circles that contain 'NI' are 'Outside' circles
pub fn circle_orientation_euclid(circ: Blade3) -> Contains {
    circle_contains(-circ, NI)
}
///compares two points along a circle, given a base point--essentially, traveling along a circle (counterclockwise) from base,
///which point do you encounter first?
///
///a point equal to the base is minimal
///
///undefined/arbitrary when A is approximately B
pub fn comp_points_on_circle(base: Blade1, a: Blade1, b: Blade1, circ: Blade3) -> Ordering {
    if a.approx_eq(&base, LOW_PRECISION) {
        return Ordering::Less;
    }
    if b.approx_eq(&base, LOW_PRECISION) {
        return Ordering::Greater;
    }
    (((base ^ b) ^ a) << circ).approx_cmp_zero(PRECISION) //make a new circle in the same place and check its orientation against that of circ
}
///returns whether or not c1 excludes c2, i.e., the inside of c2 is entirely contained in c1
pub fn circle_excludes(c1: Blade3, c2: Blade3) -> bool {
    circ_border_inside_circ(c1, c2) == Contains::Outside //return true iff the border of c2 lies in c1 and their orientations are the same
        && circle_orientation_euclid(c1) == circle_orientation_euclid(c2)
}
///helper function that takes a metric (an f64) and reads its sign, and gives a Contains. useful since a lot of this is read off of scalars from
///producting CGA blades in certain ways.
///
///arbitrarily chosen that positive values yield 'inside' and negative yield 'outside'
///
///uses PRECISION
pub fn contains_from_metric(metric: f64) -> Contains {
    contains_from_metric_prec(metric, PRECISION)
}
///helper function that takes a metric (an f64) and reads its sign, and gives a Contains. useful since a lot of this is read off of scalars from
///producting CGA blades in certain ways.
///
///arbitrarily chosen that positive values yield 'inside' and negative yield 'outside'
///
///uses the precision you give it
pub fn contains_from_metric_prec(metric: f64, prec: Precision) -> Contains {
    match metric.approx_cmp_zero(prec) {
        Ordering::Greater => Contains::Inside,
        Ordering::Less => Contains::Outside,
        Ordering::Equal => Contains::Border,
    }
}
