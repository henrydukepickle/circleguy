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
#[derive(Debug, Clone)]
pub struct ComponentShape {
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
) -> Result<BoundingCircles, String> {
    let mut new_bounding_circles = Vec::new();
    for circ in bounding_circles {
        if !circle_excludes(*circ, new_circle) {
            new_bounding_circles.push(*circ);
        } else {
            dbg!(match circ.unpack() {
                Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                _ =>
                    return Err(
                        "collapse_shape_and_add failed: Circle was a line or imaginary! (1)"
                            .to_string()
                    ),
            });
            dbg!(match new_circle.unpack() {
                Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                _ =>
                    return Err(
                        "collapse_shape_and_add failed: Circle was a line or imaginary! (2)"
                            .to_string()
                    ),
            });
        }
    }
    new_bounding_circles.push(new_circle);
    Ok(new_bounding_circles)
}
impl PieceShape {
    pub fn cut_boundary(&self, circle: Blade3) -> Result<Option<[BoundaryShape; 2]>, String> {
        fn get_circle_arcs(circle: Blade3, cut_points: &Vec<Point>) -> Result<Vec<Arc>, String> {
            if cut_points.len() <= 1 {
                return Ok(vec![Arc {
                    circle,
                    boundary: None,
                }]);
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
                    return Err(
                        "PieceShape.cut_boundary failed: cut points were too close!".to_string()
                    );
                }
            }
            Ok(arcs)
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
                return Ok(None);
            }
            // if arc
            //     .circle
            //     .rescale_oriented()
            //     .approx_eq(&circle.rescale_oriented(), Precision::new_simple(14))
            //     || arc
            //         .circle
            //         .rescale_oriented()
            //         .approx_eq(&-circle.rescale_oriented(), Precision::new_simple(14))
            // {
            //     dbg!(match circle.unpack() {
            //         Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
            //         _ => panic!("HAHA"),
            //     });
            //     dbg!(match arc.circle.unpack() {
            //         Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
            //         _ => panic!("HAHA"),
            //     });
            // }
            for int in arc.intersect_circle(circle)? {
                if let Some(point) = int {
                    cut_points.insert(point.rescale_unoriented().unpack().unwrap(), ());
                }
            }
            for i in [0, 1] {
                result[i].extend(arc.cut_by_circle(circle)?[i].clone());
            }
        }
        if result[0].is_empty() || result[1].is_empty() {
            return Ok(None);
        }
        let circle_arcs = get_circle_arcs(circle, &cut_points.into_keys().collect())?;
        for arc in &circle_arcs {
            if let Some(x) = arc.boundary
                && let Dipole::Tangent(_, _) = x.unpack()
            {
                return Err("PieceShape.cut_boundary failed: arc boundary was tangent!".to_string());
            }
            if self.contains_arc(arc.rescale_oriented())? == Contains::Inside {
                result[0].push(arc.rescale_oriented());
                result[1].push(arc.rescale_oriented().inverse());
            }
        }

        Ok(Some(result))
    }
    pub fn cut_by_circle(&self, circle: Blade3) -> Result<[Option<PieceShape>; 2], String> {
        let shapes_raw = self.cut_boundary(circle)?;
        let shapes = match shapes_raw {
            Some(x) => x,
            None => {
                return match self.in_circle(circle)? {
                    None => Err("PieceShape.cut_by_circle failed: piece_shape was cut, but still blocked the turn!".to_string()),
                    Some(Contains::Inside) => Ok([Some(self.clone()), None]),
                    Some(Contains::Outside) => Ok([None, Some(self.clone())]),
                    Some(Contains::Border) => Err("PieceShape.cut_by_circle failed: piece_shape is on border of circle!".to_string()),
                };
            }
        };
        let bounding_circles = [
            collapse_shape_and_add(&self.bounds, circle)?,
            collapse_shape_and_add(&self.bounds, -circle)?,
        ];
        let inside = PieceShape {
            border: shapes[0].clone(),
            bounds: bounding_circles[0].clone(),
        };
        let outside = PieceShape {
            border: shapes[1].clone(),
            bounds: bounding_circles[1].clone(),
        };
        return Ok([Some(inside), Some(outside)]);
    }
    pub fn in_circle(&self, circle: Blade3) -> Result<Option<Contains>, String> {
        let mut inside = None;
        for arc in &self.border {
            // if arc.in_circle(circle).is_none() {
            //     // dbg!((arc.circle & circle).mag2());
            //     // if let Dipole::Real(real) = arc.boundary.unwrap().unpack() {
            //     //     dbg!(real);
            //     // }
            //     // if let Circle::Circle { cx, cy, r, ori } = arc.circle.unpack() {
            //     //     dbg!((cx, cy, r, ori));
            //     // }
            //     // if let Circle::Circle { cx, cy, r, ori } = circle.unpack() {
            //     //     dbg!((cx, cy, r, ori));
            //     // }
            // }
            let contained = match arc.in_circle(circle)? {
                None => return Ok(None),
                Some(x) => x,
            };
            if let Some(real_inside) = inside {
                if contained != Contains::Border && real_inside != contained {
                    return Ok(None);
                }
            } else if contained != Contains::Border {
                inside = Some(contained);
            }
        }
        if inside.is_none_or(|x| x == Contains::Border) {
            return Ok(Some(Contains::Inside));
        }
        return Ok(inside);
    }
    pub fn turn(&self, turn: Turn) -> Result<Option<PieceShape>, String> {
        //dbg!(self.in_circle(turn.circle));
        if match self.in_circle(turn.circle)? {
            None => return Ok(None),
            Some(x) => x,
        } == Contains::Outside
        {
            return Ok(Some(self.clone()));
        }
        let mut new_border = Vec::new();
        for arc in &self.border {
            new_border.push(arc.rotate(turn.rotation));
        }
        let mut new_bounds = Vec::new();
        for bound in &self.bounds {
            new_bounds.push(turn.rotation.sandwich(*bound));
        }
        Ok(Some(PieceShape {
            bounds: new_bounds,
            border: new_border,
        }))
    }
    pub fn rotate(&self, rotation: Rotoflector) -> Self {
        let mut new_border = Vec::new();
        for arc in &self.border {
            new_border.push(arc.rotate(rotation));
        }
        let mut new_bounds = Vec::new();
        for bound in &self.bounds {
            new_bounds.push(rotation.sandwich(*bound));
        }
        Self {
            bounds: new_bounds,
            border: new_border,
        }
    }
    pub fn turn_cut(&self, turn: Turn) -> Result<[Option<PieceShape>; 2], String> {
        let mut cut_bits = self.cut_by_circle(turn.circle)?;
        if let Some(x) = &cut_bits[0] {
            cut_bits[0] = Some(x.rotate(turn.rotation));
        }
        Ok(cut_bits)
    }
    fn contains_arc(&self, arc: Arc) -> Result<Contains, String> {
        for circle in &self.bounds {
            let cont = arc.in_circle(*circle)?;
            if cont == None || cont == Some(Contains::Outside) {
                return Ok(Contains::Outside);
            }
        }
        Ok(Contains::Inside)
    }
    pub fn get_components(&self) -> Result<Vec<ComponentShape>, String> {
        let mut comps = Vec::new();
        let mut remaining_arcs = self.border.clone();
        loop {
            if remaining_arcs.is_empty() {
                break;
            }
            let mut curr_arc = remaining_arcs.pop().unwrap();
            let mut curr_comp = vec![curr_arc];
            loop {
                if curr_arc.boundary.is_none() {
                    break;
                }
                if let Some(next) = next_arc(&self.border, curr_arc) {
                    curr_arc = next;
                    curr_comp.push(curr_arc);
                } else {
                    return Err("PieceShape.get_components failed: No next arc found!".to_string());
                }
                if let Some(x) = curr_arc.boundary
                    && let Dipole::Real(pair) = x.unpack()
                    && let Some(y) = curr_comp[0].boundary
                    && let Dipole::Real(base_pair) = y.unpack()
                    && pair[1].approx_eq(&base_pair[0], PRECISION)
                {
                    break;
                }
            }
            comps.push(ComponentShape { border: curr_comp });
        }
        Ok(comps)
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
