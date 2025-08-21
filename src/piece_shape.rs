use crate::PRECISION;
use crate::arc::*;
use crate::circle_utils::*;
use crate::turn::*;
use approx_collections::*;
use cga2d::*;
use std::cmp::*;

pub type BoundingCircles = Vec<Blade3>;
pub type BoundaryShape = Vec<Arc>;
#[derive(Debug, Clone)]
pub struct PieceShape {
    pub bounds: BoundingCircles,
    pub border: BoundaryShape,
}
//CGA NEEDS TESTING
//gives the 'inside of the circle' arcs, ideally
pub fn inner_circle_arcs(mut starts: Vec<Blade1>, mut ends: Vec<Blade1>, circ: Blade3) -> Vec<Arc> {
    if (starts.len()) != (ends.len()) {
        panic!("inequal number of starts and ends passed");
    }
    if starts.is_empty() {
        return Vec::new();
    }
    let mut arcs = Vec::new();
    ends.sort_by(|a, b| comp_points_on_circle(starts[0], *a, *b, circ));
    starts.sort_by(|a, b| comp_points_on_circle(*ends.last().unwrap(), *a, *b, circ));
    for i in 0..starts.len() {
        if starts[i].approx_eq(&ends[i], PRECISION) {
            continue;
        } else {
            arcs.push(Arc {
                circle: circ,
                boundary: Some((starts[i] ^ ends[i]).rescale_oriented()),
            });
        }
    }
    return arcs;
}
pub fn cut_boundary(bound: &BoundaryShape, circle: Blade3) -> Option<[BoundaryShape; 2]> {
    //REWORK ALL
    let mut starts = Vec::new();
    let mut ends = Vec::new();
    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for i in 0..bound.len() {
        let arc = bound[i];
        if arc.circle.approx_eq(&circle, PRECISION) || arc.circle.approx_eq(&-circle, PRECISION) {
            return None;
        }
        let mut cut_points = Vec::new();
        let int = arc.intersect_circle(circle);
        if int[0].is_some()
            && !(arc.boundary.is_some()
                // && arc
                //     .boundary
                //     .unwrap()
                //     .mag2()
                //     .approx_sign(PRECISION)
                //     != Sign::Zero
                && int[0].unwrap().approx_eq(
                    &match arc.boundary.unwrap().unpack() {
                        Dipole::Real(real) => real[1].into(),
                        _ => panic!("JIM???? I HAVENT SEEN YOU IN YEARS!"),
                    },
                    PRECISION,
                )
                && (next_arc(&bound, arc).unwrap().circle & circle)
                    .mag2().approx_cmp_zero(PRECISION) == Ordering::Greater)
        {
            starts.push(int[0].unwrap());
            cut_points.push(int[0].unwrap());
        }
        if int[1].is_some()
            && !(arc.boundary.is_some()
                // && arc
                //     .boundary
                //     .unwrap()
                //     .mag2()
                //     .approx_sign(PRECISION)
                //     != Sign::Zero
                && int[1].unwrap().approx_eq(
                    &match arc.boundary.unwrap().unpack() {
                        Dipole::Real(real) => real[1].into(),
                        _ => panic!("JIM???? I HAVENT SEEN YOU IN YEARS!"),
                    },
                    PRECISION,
                )
                && (next_arc(&bound, arc).unwrap().circle & circle)
                    .mag2()
                    .approx_cmp_zero(PRECISION) == Ordering::Greater)
        {
            ends.push(int[1].unwrap());
            cut_points.push(int[1].unwrap());
        }
        let [add_inside, add_outside] = arc.cut_by_circle(circle);
        inside.extend(add_inside);
        outside.extend(add_outside);
    }
    if (inside.is_empty()) || (outside.is_empty()) {
        return None;
    }
    for arc in inner_circle_arcs(starts, ends, circle) {
        inside.push(arc);
        outside.push(arc.inverse());
    }
    return Some([inside, outside]);
}

//return the collapsed CCP representation
pub fn collapse_shape_and_add(
    bounding_circles: &BoundingCircles,
    new_circle: Blade3,
) -> BoundingCircles {
    let mut new_bounding_circles = Vec::new();
    for circ in bounding_circles {
        if !circle_excludes(*circ, new_circle) {
            new_bounding_circles.push(*circ);
        }
    }
    new_bounding_circles.push(new_circle);
    new_bounding_circles
}
impl PieceShape {
    pub fn cut_by_circle(&self, circle: Blade3) -> Option<[PieceShape; 2]> {
        let shapes = cut_boundary(&self.border, circle)?;
        let bounding_circles = [
            collapse_shape_and_add(&self.bounds, circle),
            collapse_shape_and_add(&self.bounds, -circle),
        ];
        let inside = PieceShape {
            border: shapes[0].clone(),
            bounds: bounding_circles[0].clone(),
        };
        let outside = PieceShape {
            border: shapes[1].clone(),
            bounds: bounding_circles[1].clone(),
        };
        return Some([inside, outside]);
    }
    pub fn in_circle(&self, circle: Blade3) -> Option<Contains> {
        let mut inside = None;
        for arc in &self.border {
            let contained = arc.in_circle(circle)?;
            if let Some(real_inside) = inside {
                if contained != Contains::Border && real_inside != contained {
                    return None;
                }
            } else if contained != Contains::Border {
                inside = Some(contained);
            }
        }
        if inside.is_none_or(|x| x == Contains::Border) {
            return Some(Contains::Inside);
        }
        return inside;
    }
    pub fn turn(&self, turn: Turn) -> Option<PieceShape> {
        //dbg!(self.in_circle(turn.circle));
        if self.in_circle(turn.circle)? == Contains::Outside {
            return Some(self.clone());
        }
        let mut new_border = Vec::new();
        for arc in &self.border {
            new_border.push(arc.rotate(turn.rotation));
        }
        let mut new_bounds = Vec::new();
        for bound in &self.bounds {
            new_bounds.push(turn.rotation.sandwich(*bound));
        }
        Some(PieceShape {
            bounds: new_bounds,
            border: new_border,
        })
    }
}
pub fn next_arc(bound: &BoundaryShape, curr: Arc) -> Option<Arc> {
    for arc in bound {
        if let Some(boundary) = arc.boundary
            && let Dipole::Real(real) = boundary.unpack()
            && let Dipole::Real(real_curr) = curr.boundary?.unpack()
            && (real_curr[1].approx_eq(&real[0], PRECISION))
        {
            return Some(*arc);
        }
    }
    None
}
