use std::f64::consts::PI;

use approx_collections::ApproxEq;

use crate::{
    PRECISION,
    complex::{
        arc::Arc,
        c64::Point,
        complex_circle::{Circle, Contains, OrientedCircle, inside_bounds},
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct PieceShape {
    pub bounds: Vec<OrientedCircle>,
    pub border: Vec<Arc>,
}

impl PieceShape {
    pub fn contains(&self, point: Point) -> Contains {
        inside_bounds(&self.bounds, point)
    }
    fn cut_circle(&self, circle: Circle) -> Option<(Vec<Arc>, Vec<Arc>)> {
        let mut points = Vec::new();
        for arc in &self.border {
            for point in arc.intersect_circle(circle, false)? {
                points.push(point);
            }
        }
        if points.is_empty() {
            return None;
        }
        let start = points.remove(0);
        let arc = Arc {
            circle,
            start,
            angle: 2. * PI,
        };
        let arcs = arc
            .cut_at(points)
            .into_iter()
            .filter(|x| self.contains(x.midpoint()) == Contains::Inside);
        Some((arcs.clone().collect(), arcs.map(|x| x.inverse()).collect()))
    }
    fn cut_border_by_circle(&self, circle: Circle) -> Option<(Vec<Arc>, Vec<Arc>)> {
        let mut arc_pieces = Vec::new();
        for arc in &self.border {
            if arc.circle.approx_eq(&circle, PRECISION) {
                return dbg!(None);
            }
            dbg!((arc.cut_by_circle(circle).unwrap().len()));
            arc_pieces.extend(arc.cut_by_circle(circle).unwrap());
        }
        let (mut inside, mut outside) = (Vec::new(), Vec::new());
        for arc in &arc_pieces {
            if ((circle).contains(((arc).midpoint()))) == Contains::Inside {
                inside.push(*arc);
            } else {
                outside.push(*arc);
            }
        }
        if dbg!((&outside).is_empty()) || dbg!((&inside).is_empty()) {
            return None;
        }
        let circle_arcs = (self.cut_circle(circle)?);
        inside.extend(circle_arcs.0);
        outside.extend(circle_arcs.1);
        Some((inside, outside))
    }
    pub fn cut_by_circle(&self, circle: Circle) -> Option<(PieceShape, PieceShape)> {
        let (inside_border, outside_border) = self.cut_border_by_circle(circle)?;
        let (mut inside_bounds, mut outside_bounds) = (self.bounds.clone(), self.bounds.clone());
        inside_bounds.push(OrientedCircle {
            circ: circle,
            ori: Contains::Inside,
        });
        outside_bounds.push(OrientedCircle {
            circ: circle,
            ori: Contains::Outside,
        });
        Some((
            PieceShape {
                border: inside_border,
                bounds: inside_bounds,
            },
            PieceShape {
                border: outside_border,
                bounds: outside_bounds,
            },
        ))
    }
    ///detect if a shape is in a circle. Some(x) means that the shape is entirely x, None means that the shape crosses the border of the circle
    pub fn in_circle(&self, circle: Circle) -> Option<Contains> {
        let mut inside = None; //tracks whether the piece is inside the circle
        for arc in &self.border {
            //iterate over the border arcs. essentially, if all of them are in or out, return Some(in or out), otherwise we return none.
            let contained = match arc.in_circle(circle) {
                //match if the arc is in the circle. if it crosses the border, we can immediately return None
                None => return None,
                Some(x) => x,
            };
            if let Some(real_inside) = inside {
                //if the arc has been decided as in or out, check the value of contained against (in or out).
                if contained != Contains::Border && real_inside != contained {
                    //if the values differ, and we arent examining the border case, return that it crosses.
                    return None;
                }
            } else if contained != Contains::Border {
                //if the piece is undecided on in/out and the arc lies properly in/out, set inside.
                inside = Some(contained);
            }
        }
        if inside.is_none_or(|x| x == Contains::Border) {
            //once all arcs have been iterated, if the value of inside was never set (i.e., all arcs lied on the border) return inside.
            //This decides a convention
            return Some(Contains::Inside);
        }
        return inside; //return in/out
    }
}
