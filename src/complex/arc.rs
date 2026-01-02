use approx_collections::ApproxEq;

use crate::{
    PRECISION,
    complex::{
        c64::Scalar,
        complex_circle::{Circle, Contains, Orientation},
        point::Point,
    },
};
use std::{cmp::Ordering, f64::consts::PI};

#[derive(Debug, Clone, Copy)]
///arc around a circle. can go clockwise or ccw (along the circle)
pub struct Arc {
    pub circle: Circle,
    pub start: Point,
    pub angle: Scalar,
}

impl Arc {
    ///get the endpoint of the arc
    pub fn end(&self) -> Point {
        self.start.rotate_about(self.circle.center, self.angle)
    }
    ///get the midpoint of the arc
    pub fn midpoint(&self) -> Point {
        self.start.rotate_about(self.circle.center, self.angle / 2.)
    }
    ///get an arc from the endpoints and an orientation
    pub fn from_endpoints(circ: Circle, start: Point, end: Point, ori: Orientation) -> Self {
        Self {
            circle: circ,
            start,
            angle: (match ori {
                Orientation::CCW => ((end - circ.center).angle() - (start - circ.center).angle())
                    .rem_euclid(2.0 * PI),
                Orientation::CW => {
                    -(2. * PI
                        - (((end - circ.center).angle() - (start - circ.center).angle())
                            .rem_euclid(2.0 * PI)))
                }
            }),
        }
    }
    ///get the orientation of an arc (according to the sign of its angle)
    pub fn orientation(&self) -> Orientation {
        if self.angle < 0. {
            Orientation::CW
        } else {
            Orientation::CCW
        }
    }
    ///get the inverse of an arc. this arc starts at (arc.end()) and ends at (arc.start), going the opposite direction from arc.
    ///arc and arc.inverse() contain exactly the same points
    pub fn inverse(&self) -> Arc {
        Arc {
            circle: self.circle,
            start: self.end(),
            angle: -self.angle,
        }
    }
    ///check if an arc contains a point.
    ///Border means that the point is equal to one of the endpoints.
    ///only points on arc.circle should be passed.
    pub fn contains_point(&self, point: Point) -> Contains {
        if point.approx_eq(&self.start, PRECISION) || point.approx_eq(&self.end(), PRECISION) {
            Contains::Border
        } else if self.start.approx_eq(&self.end(), PRECISION) {
            Contains::Inside
        }
        //use comp_points_on_circle to see if the endpoint or point comes first along the arc, wrt arc.start
        else if self.circle.comp_points_on_circle(
            self.start,
            point,
            self.end(),
            self.orientation(),
        ) == Ordering::Less
        {
            Contains::Inside
        } else {
            Contains::Outside
        }
    }
    ///intersect the arc with the circle.
    ///returns None if the circle and arc.circle are approximately equal.
    ///proper = true excludes intersections at the endpoints of the arc. proper = false includes them
    pub fn intersect_circle(&self, circle: Circle, proper: bool) -> Option<Vec<Point>> {
        if self.circle.approx_eq(&circle, PRECISION) {
            return None;
        }
        let intersects = self.circle.intersect_circle(circle);
        let mut inter = Vec::new();
        //loop through the intersection points, adding the ones in the arc
        for int in &intersects {
            match self.contains_point(*int) {
                Contains::Inside => {
                    inter.push(*int);
                }
                Contains::Border => {
                    if !proper {
                        inter.push(*int);
                    }
                }
                Contains::Outside => {}
            }
        }
        Some(inter)
    }
    ///cut the arc into multiple arcs. the points need not be sorted along the arc, and passing duplicate points is allowed.
    ///precondition: all of the points lie on the arc
    pub fn cut_at(&self, points: Vec<Point>) -> Vec<Self> {
        if points.is_empty() {
            return vec![*self];
        }
        let mut points = points;
        //sort the points along the arc
        points.sort_by(|a, b| {
            self.circle
                .comp_points_on_circle(self.start, *a, *b, self.orientation())
        });
        let mut endpoints = vec![self.start];
        endpoints.extend(points);
        endpoints.push(self.end());
        //get all the endpoints, including adding the start and endpoint of the arcs
        let mut arcs = Vec::new();
        for i in 0..(endpoints.len() - 1) {
            //make the new arcs from the points. exclude arcs that would have the same start and endpoint
            if !endpoints[i].approx_eq(&endpoints[i + 1], PRECISION) {
                arcs.push(Self::from_endpoints(
                    self.circle,
                    endpoints[i],
                    endpoints[i + 1],
                    self.orientation(),
                ));
            }
        }
        arcs
    }

    ///cut an arc by a circle. returns None if arc.circle and circle are approx_eq
    pub fn cut_by_circle(&self, circle: Circle) -> Option<Vec<Self>> {
        Some(self.cut_at(self.intersect_circle(circle, true)?))
    }
    ///check if an arc is in a circle.
    ///None means that the arc properly crosses the boundary of the circle.
    ///Border means that arc.circle and circle are approx_eq
    pub fn in_circle(&self, circle: Circle) -> Option<Contains> {
        if self.circle.approx_eq(&circle, PRECISION) {
            return Some(Contains::Border);
        }
        let (s, m, e) = (
            circle.contains(self.start),
            circle.contains(self.midpoint()),
            circle.contains(self.end()),
        );
        let int = self.intersect_circle(circle, true).unwrap();
        if int.is_empty() {
            Some(m)
        } else if int.len() == 1 {
            if s == e { Some(s) } else { None }
        } else {
            None
        }
    }
}
