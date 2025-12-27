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
    ///check if a shape contains a point, according to its bounding circles (CCP representation)
    pub fn contains(&self, point: Point) -> Contains {
        inside_bounds(&self.bounds, point)
    }
    ///cut a circle by a piece, used in cutting the border of the piece by a circle.
    ///throws out the resulting arcs lying outside the piece.
    ///returns two sets of arcs, with opposite orientations, again for cutting.
    ///returns None if the circle and self.border do not intersect.
    fn cut_circle(&self, circle: Circle) -> Option<(Vec<Arc>, Vec<Arc>)> {
        let mut points = Vec::new();
        //get the points where the piece border intersects the circle
        for arc in &self.border {
            for point in arc.intersect_circle(circle, false)? {
                points.push(point);
            }
        }
        if points.is_empty() {
            return None;
        }
        let start = points.remove(0);
        //make a new arc representing the whole circle
        let arc = Arc {
            circle,
            start,
            angle: 2. * PI,
        };
        let arcs = arc //cut the arc at the intersection points
            .cut_at(points)
            .into_iter()
            .filter(|x| self.contains(x.midpoint()) == Contains::Inside); //filter out the arcs not in the piece
        Some((arcs.clone().collect(), arcs.map(|x| x.inverse()).collect())) //return the arcs (with normal and opposite orientation)
    }
    ///cut the border of a shape by a circle.
    ///returns None if no cut was made (the piece didnt cross the circle)
    ///returns (inside, outside) pieces otherwise
    fn cut_border_by_circle(&self, circle: Circle) -> Option<(Vec<Arc>, Vec<Arc>)> {
        let mut arc_pieces = Vec::new();
        for arc in &self.border {
            //if any arc is around circle, no cut is made
            if arc.circle.approx_eq(&circle, PRECISION) {
                return None;
            }
            //cut the arc and store the pieces
            arc_pieces.extend(arc.cut_by_circle(circle).unwrap());
        }
        //the arcs in the inside and outside pieces
        let (mut inside, mut outside) = (Vec::new(), Vec::new());
        for arc in &arc_pieces {
            //add the arc to the inside if its midpoint is in the circle. otherwise, add it to the outside
            //this works because the arcs were cut by the circle, and so lie either inside or outside of the circle (they do not cross)
            if ((circle).contains((arc).midpoint())) == Contains::Inside {
                inside.push(*arc);
            } else {
                outside.push(*arc);
            }
        }
        //if all the arcs are inside or outside, no cut is made since one of the resulting pieces would be empty
        if outside.is_empty() || inside.is_empty() {
            return None;
        }
        //cut the circle by the border and add the pieces to inside or outside
        //if the circle isnt cut, the piece might still be cut (if it was disconnected) so we just don't add any arcs
        if let Some(circle_arcs) = self.cut_circle(circle) {
            inside.extend(circle_arcs.0);
            outside.extend(circle_arcs.1);
        }
        Some((inside, outside))
    }
    ///cut the shape by a circle. returns None if no cut was made, otherwise returns (inside, outside)
    pub fn cut_by_circle(&self, circle: Circle) -> Option<(PieceShape, PieceShape)> {
        let (inside_border, outside_border) = self.cut_border_by_circle(circle)?; //the borders of the cut pieces
        let (mut inside_bounds, mut outside_bounds) = (self.bounds.clone(), self.bounds.clone());
        //add the new bounding circles to the new pieces
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
            return Some(Contains::Inside);
        }
        return inside;
    }
}
