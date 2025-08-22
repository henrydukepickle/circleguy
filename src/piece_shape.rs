use crate::PRECISION;
use crate::arc::*;
use crate::circle_utils::*;
use crate::turn::*;
use approx_collections::*;
use cga2d::*;

pub type BoundingCircles = Vec<Blade3>;
pub type BoundaryShape = Vec<Arc>;
#[derive(Debug, Clone)]
pub struct PieceShape {
    pub bounds: BoundingCircles,
    pub border: BoundaryShape,
}
//CGA NEEDS TESTING
//gives the 'inside of the circle' arcs, ideally
// pub fn inner_circle_arcs(mut starts: Vec<Blade1>, mut ends: Vec<Blade1>, circ: Blade3) -> Vec<Arc> {
//     if (starts.len()) != (ends.len()) {
//         panic!("inequal number of starts and ends passed");
//     }
//     if starts.is_empty() {
//         return Vec::new();
//     }
//     let mut arcs = Vec::new();
//     ends.sort_by(|a, b| comp_points_on_circle(starts[0], *a, *b, circ));
//     starts.sort_by(|a, b| comp_points_on_circle(*ends.last().unwrap(), *a, *b, circ));
//     for i in 0..starts.len() {
//         if starts[i].approx_eq(&ends[i], PRECISION) {
//             continue;
//         } else {
//             arcs.push(Arc {
//                 circle: circ,
//                 boundary: Some((starts[i] ^ ends[i]).rescale_oriented()),
//             });
//         }
//     }
//     return arcs;
// }

//return the collapsed CCP representation
pub fn collapse_shape_and_add(
    bounding_circles: &BoundingCircles,
    new_circle: Blade3,
) -> BoundingCircles {
    let mut new_bounding_circles = Vec::new();
    for circ in bounding_circles {
        if !circle_excludes(*circ, new_circle) {
            new_bounding_circles.push(*circ);
        } else {
            dbg!(match circ.unpack() {
                Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                _ => panic!("hi"),
            });
            dbg!(match new_circle.unpack() {
                Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                _ => panic!("hi"),
            });
        }
    }
    new_bounding_circles.push(new_circle);
    new_bounding_circles
}
impl PieceShape {
    pub fn cut_boundary(&self, circle: Blade3) -> Option<[BoundaryShape; 2]> {
        fn get_circle_arcs(circle: Blade3, cut_points: &Vec<Point>) -> Vec<Arc> {
            if cut_points.len() <= 1 {
                return vec![Arc {
                    circle,
                    boundary: None,
                }];
            }
            let mut points = cut_points.clone();
            let base = points.pop().unwrap();
            points.sort_by(|a, b| {
                comp_points_on_circle(base.into(), (*a).into(), (*b).into(), circle)
            });
            let mut arcs = Vec::new();
            points.insert(0, base);
            points.push(base);
            for i in 0..(points.len() - 1) {
                arcs.push(Arc {
                    circle,
                    boundary: Some(
                        (points[i].to_blade().rescale_unoriented()
                            ^ points[i + 1].to_blade().rescale_unoriented())
                        .rescale_oriented(),
                    ),
                });
                if points[i].approx_eq(&points[i + 1], PRECISION) {
                    panic!("POINTS WERE TOO CLOSE!");
                }
            }
            arcs
        }
        let mut result = [Vec::new(), Vec::new()];
        let mut cut_points: ApproxHashMap<Point, ()> = ApproxHashMap::new(PRECISION);
        for arc in &self.border {
            // dbg!(match arc.circle.unpack() {
            //     Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
            //     _ => panic!("hi"),
            // });
            // dbg!(match circle.unpack() {
            //     Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
            //     _ => panic!("hi"),
            // });
            if arc
                .circle
                .rescale_oriented()
                .approx_eq(&circle.rescale_oriented(), PRECISION)
                || arc
                    .circle
                    .rescale_oriented()
                    .approx_eq(&-circle.rescale_oriented(), PRECISION)
            {
                return None;
            }
            if arc
                .circle
                .rescale_oriented()
                .approx_eq(&circle.rescale_oriented(), Precision::new_simple(14))
                || arc
                    .circle
                    .rescale_oriented()
                    .approx_eq(&-circle.rescale_oriented(), Precision::new_simple(14))
            {
                dbg!(match circle.unpack() {
                    Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                    _ => panic!("HAHA"),
                });
                dbg!(match arc.circle.unpack() {
                    Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                    _ => panic!("HAHA"),
                });
            }
            for int in arc.intersect_circle(circle) {
                if let Some(point) = int {
                    cut_points.insert(point.rescale_unoriented().unpack().unwrap(), ());
                }
            }
            for i in [0, 1] {
                for arc in arc.cut_by_circle(circle)[i].clone() {
                    if let Some(x) = arc.boundary
                        && let Dipole::Tangent(_, _) = x.unpack()
                    {
                        panic!("TANGENT LENGTH 0 ARC ETC");
                    }
                }
                result[i].extend(arc.cut_by_circle(circle)[i].clone());
            }
        }
        if result[0].is_empty() || result[1].is_empty() {
            return None;
        }
        let circle_arcs = get_circle_arcs(circle, &cut_points.into_keys().collect());
        for arc in &circle_arcs {
            if let Some(x) = arc.boundary
                && let Dipole::Tangent(_, _) = x.unpack()
            {
                panic!("TANGENT LENGTH 0 ARC ETC");
            }
            if self.contains_arc(arc.rescale_oriented()) == Contains::Inside {
                result[0].push(arc.rescale_oriented());
                result[1].push(arc.rescale_oriented().inverse());
            }
        }

        Some(result)
    }
    pub fn cut_by_circle(&self, circle: Blade3) -> Option<[PieceShape; 2]> {
        let shapes = self.cut_boundary(circle)?;
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
            if arc.in_circle(circle).is_none() {
                // dbg!((arc.circle & circle).mag2());
                // if let Dipole::Real(real) = arc.boundary.unwrap().unpack() {
                //     dbg!(real);
                // }
                // if let Circle::Circle { cx, cy, r, ori } = arc.circle.unpack() {
                //     dbg!((cx, cy, r, ori));
                // }
                // if let Circle::Circle { cx, cy, r, ori } = circle.unpack() {
                //     dbg!((cx, cy, r, ori));
                // }
            }
            let contained = (arc.in_circle(circle))?;
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
    fn contains_arc(&self, arc: Arc) -> Contains {
        for circle in &self.bounds {
            let cont = (arc.in_circle(*circle));
            if cont == None || cont == Some(Contains::Outside) {
                return Contains::Outside;
            }
        }
        Contains::Inside
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
