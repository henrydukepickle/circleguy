use approx_collections::ApproxEq;

use crate::{
    PRECISION,
    complex::{
        c64::{Point, Scalar},
        complex_circle::{Circle, Contains, Orientation},
    },
};
use std::{cmp::Ordering, f64::consts::PI};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    pub circle: Circle,
    pub start: Point,
    pub angle: Scalar,
}

impl Arc {
    pub fn end(&self) -> Point {
        self.start.rotate_about(self.circle.center, self.angle)
    }
    pub fn midpoint(&self) -> Point {
        self.start.rotate_about(self.circle.center, self.angle / 2.)
    }
    pub fn from_endpoints(circ: Circle, start: Point, end: Point, ori: Orientation) -> Self {
        Self {
            circle: circ,
            start,
            angle: (match ori {
                Orientation::CCW => ((end - circ.center).angle() - (start - circ.center).angle())
                    .rem_euclid(2.0 * PI),
                Orientation::CW => {
                    (2. * PI
                        - (((end - circ.center).angle() - (start - circ.center).angle())
                            .rem_euclid(2.0 * PI)))
                        * -1.
                }
            }),
        }
    }
    pub fn orientation(&self) -> Orientation {
        if self.angle < 0. {
            Orientation::CW
        } else {
            Orientation::CCW
        }
    }
    pub fn inverse(&self) -> Arc {
        Arc {
            circle: self.circle,
            start: self.end(),
            angle: -self.angle,
        }
    }
    pub fn contains_point(&self, point: Point) -> Contains {
        if point.approx_eq(&self.start, PRECISION) || point.approx_eq(&self.end(), PRECISION) {
            Contains::Border
        } else if self.start.approx_eq(&self.end(), PRECISION) {
            Contains::Inside
        }
        //needs to be checked; i just guessed
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
    pub fn intersect_circle(&self, circle: Circle, proper: bool) -> Option<Vec<Point>> {
        if self.circle.approx_eq(&circle, PRECISION) {
            return None;
        }
        let intersects = self.circle.intersect_circle(circle);
        let mut inter = Vec::new();
        for int in &intersects {
            match self.contains_point(*int) {
                Contains::Inside => {
                    inter.push(int.clone());
                }
                Contains::Border => {
                    if !proper {
                        inter.push(int.clone());
                    }
                }
                Contains::Outside => {}
            }
        }
        Some(inter)
    }
    //precondition: all of the points lie (properly) on the arc and none are approxeq
    pub fn cut_at(&self, points: Vec<Point>) -> Vec<Self> {
        if points.is_empty() {
            return vec![*self];
        }
        let mut points = points;
        points.sort_by(|a, b| {
            self.circle
                .comp_points_on_circle(self.start, *a, *b, self.orientation())
        });
        let mut endpoints = vec![self.start];
        endpoints.extend(points);
        endpoints.push(self.end());
        let mut arcs = Vec::new();
        for i in 0..(endpoints.len() - 1) {
            if endpoints[i].approx_eq(&endpoints[i + 1], PRECISION) {
                continue;
            }
            arcs.push(Self::from_endpoints(
                self.circle.clone(),
                endpoints[i].clone(),
                endpoints[i + 1].clone(),
                self.orientation(),
            ));
        }
        arcs
    }

    pub fn cut_by_circle(&self, circle: Circle) -> Option<Vec<Self>> {
        Some(self.cut_at(self.intersect_circle(circle, true)?))
    }
    //None - crosses
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
