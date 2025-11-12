use crate::PRECISION;
use crate::SQRT_PRECISION;
use crate::circle_utils::*;
use cga2d::*;
//if boundary is None, then the arc is the whole circle
#[derive(Debug, Clone, Copy)]
pub struct Arc {
    pub circle: Blade3,
    pub boundary: Option<Blade2>,
}
impl Arc {
    ///check if an arc contains a point, None means that the point is not even on the circle of the arc
    pub fn contains(&self, point: Blade1) -> Result<Option<Contains>, String> {
        if circle_contains(self.circle, point) != Contains::Border {
            //if the point does not lie on the circle, return None
            return Ok(None);
        }
        if self.boundary == None {
            //if the boundary doesnt exist, the arc is a circle and the point is in the circle
            return Ok(Some(Contains::Inside));
        }
        if let Dipole::Real(real) = self.boundary.unwrap().unpack() {
            //if the boundary is real, unpack it
            for p in real {
                //for each point in the dipole
                if let Some(x) = point.unpack() {
                    //unpack it
                    if x.approx_eq(&p, SQRT_PRECISION) {
                        //if the point is approximately equal to the boundary, return border
                        return Ok(Some(Contains::Border));
                    }
                } else {
                    return Err("Arc.contains failed: point passed was not real!".to_string());
                }
            }
            return Ok(Some(contains_from_metric(
                -((self.boundary.unwrap() ^ point) << self.circle), //decide in/out
            )));
        }
        // if let Dipole::Tangent(p, d) = self.boundary.unwrap().unpack() {
        //     dbg!(p);
        // }
        Err("Arc.contains failed: Arc boundary was tangent or imaginary.".to_string())
    }

    //IF THEY ARE TANGENT, THEN return[1] is always NONE
    ///intersect the arc with a circle. the two points will be in a CGA-fixed order. if the circle and the arc are tangent, the first index is used.
    pub fn intersect_circle(&self, circle: Blade3) -> Result<[Option<Blade1>; 2], String> {
        if (circle & self.circle).approx_eq_zero(SQRT_PRECISION) {
            //if the circle and arc.circle do not touch, return nothing
            return Ok([None; 2]);
        }
        match (self.circle.rescale_oriented() & circle.rescale_oriented())
            .unpack_with_prec(PRECISION)
        {
            Dipole::Real(int_points) => {
                //if there are two intersection points
                let mut new_points = [None; 2]; //instantiate new points
                for i in [0, 1] {
                    //for each of the intersect points
                    let cont = self.contains(int_points[i].into())?; //if the arc contains the point
                    if cont == Some(Contains::Border) || cont == Some(Contains::Inside) {
                        //if the point is on the arc or the border of the arc, it counts as an intersection point
                        new_points[i] = Some(int_points[i].into());
                    };
                }
                Ok(new_points)
            }
            // int_points.map(|a| match self.contains(a.into()) {
            //     None => None,
            //     Some(Contains::Outside) => None,
            //     Some(Contains::Border) | Some(Contains::Inside) => Some(a.into()),
            // }),
            Dipole::Tangent(p, _) => match self.contains(p.into())? {
                Some(Contains::Inside) | Some(Contains::Border) => Ok([Some(p.into()), None]),
                Some(Contains::Outside) => Ok([None; 2]),
                None => {
                    dbg!(p);
                    match self.circle.unpack() {
                        Circle::Circle { cx, cy, r, ori } => {
                            dbg!((cx, cy, r, ori));
                        }
                        _ => panic!("L"),
                    }
                    match circle.unpack() {
                        Circle::Circle { cx, cy, r, ori } => {
                            dbg!((cx, cy, r, ori));
                        }
                        _ => panic!("L"),
                    }
                    Err(
                        "Arc.intersect_circle failed: intersection point is not on arc.circle!"
                            .to_string(),
                    )
                }
            },
            _ => Ok([None; 2]),
        }
    }
    pub fn rescale_oriented(&self) -> Self {
        Self {
            circle: self.circle.rescale_oriented(),
            boundary: match self.boundary {
                None => None,
                Some(x) => Some(x.rescale_oriented()),
            },
        }
    }

    //result[0] inside
    //dont pass aeq circles
    //please, i beg of you, dont do it
    //dont you dare
    //if you pass aeq circles i will hunt you down
    //im not joking
    //will sort if passed an arc that doesnt intersect the circle
    pub fn cut_by_circle(&self, circle: Blade3) -> Result<[Vec<Arc>; 2], String> {
        //REWORK ALL
        let mut sorted_arcs = [Vec::new(), Vec::new()];
        let mut segments = Vec::new();
        let mut new_points: Vec<Blade1> = Vec::new();
        // if let Circle::Circle { cx, cy, r, ori } = circle.unpack() {
        //     dbg!((cx, cy, r, ori));
        // }
        // if let Circle::Circle { cx, cy, r, ori } = self.circle.unpack() {
        //     dbg!((cx, cy, r, ori));
        // }
        match (circle & self.circle).unpack_with_prec(SQRT_PRECISION) {
            Dipole::Real(intersects) => {
                for intersect in intersects {
                    if self.contains(intersect.into())?.ok_or(
                        "Arc.cut_by_circle failed: intersection point was not on arc.circle!"
                            .to_string(),
                    )? == Contains::Inside
                    {
                        new_points.push(intersect.into());
                    }
                }
                for point in &mut new_points {
                    *point = point.rescale_oriented();
                }
                if new_points.is_empty() {
                    segments = vec![*self];
                } else {
                    let mut base = new_points[0];
                    if let Some(x) = self.boundary {
                        if let Dipole::Real(r) = x.unpack() {
                            base = r[0].into();
                        } else {
                            return Err(
                            "Arc.cut_by_circle failed: arc boundary was tangent or imaginary! (1)"
                                .to_string(),
                        );
                        }
                    }
                    new_points.sort_by(|a, b| comp_points_on_circle(base, *a, *b, self.circle));
                    if let Some(x) = self.boundary {
                        new_points.insert(
                            0,
                            match x.unpack() {
                                Dipole::Real(r) => r[0].into(),
                                _ => return Err(
                            "Arc.cut_by_circle failed: arc boundary was tangent or imaginary! (2)".to_string()
                        ),
                            },
                        );
                        new_points.push(match x.unpack() {
                            Dipole::Real(r) => r[1].into(),
                            _ => return Err(
                            "Arc.cut_by_circle failed: arc boundary was tangent or imaginary! (3)".to_string()
                        ),
                        });
                    } else {
                        new_points.push(base);
                    }
                    //(&new_points);
                    for i in 0..(new_points.len() - 1) {
                        let arc = Arc {
                            circle: self.circle,
                            boundary: Some(
                                (new_points[i].rescale_unoriented()
                                    ^ new_points[i + 1].rescale_unoriented())
                                .rescale_oriented(),
                            ),
                        };
                        if let Some(x) = arc.boundary
                            && let Dipole::Tangent(_, _) = x.unpack()
                        {
                            // dbg!(new_points[i].unpack().unwrap());
                            // dbg!(new_points[i + 1].unpack().unwrap());
                            return Err(
                            "Arc.cut_by_circle failed: arc boundary was tangent or imaginary! (4)".to_string()
                        );
                        }
                        segments.push(arc);
                    }
                }
            }
            _ => segments = vec![*self],
        }
        for arc in segments {
            if let Some(x) = arc.boundary
                && let Dipole::Tangent(_, _) = x.unpack()
            {
                return Err("Arc.cut_by_circle failed: arc.boundary was tangent!".to_string());
            }
            //dbg!(arc.circle);
            //dbg!(circle);
            match arc.in_circle(circle)? {
                None => {
                    dbg!(arc);
                    // dbg!(match arc.boundary.unwrap().unpack() {
                    //     Dipole::Real(r) => r,
                    //     _ => panic!("hi"),
                    // });
                    // dbg!(match arc.circle.unpack() {
                    //     Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                    //     _ => panic!("hi"),
                    // });
                    // dbg!(match circle.unpack() {
                    //     Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                    //     _ => panic!("hi"),
                    // });
                    // if arc.intersect_circle(circle)[0].is_some() {
                    //     dbg!(arc.intersect_circle(circle)[0].unwrap().unpack());
                    // }
                    // if arc.intersect_circle(circle)[1].is_some() {
                    //     dbg!(arc.intersect_circle(circle)[0].unwrap().unpack());
                    // }
                    dbg!(circle);
                    return Err("Arc.cut_by_circle failed: cut arc piece still overlaps properly with circle!".to_string());
                }
                Some(Contains::Inside) => sorted_arcs[0].push(arc),
                Some(Contains::Border) => {
                    sorted_arcs[0].push(arc);
                    sorted_arcs[1].push(arc);
                    panic!("AAAA");
                } //in this case the arc is tangent to the circle and on the circle
                Some(Contains::Outside) => sorted_arcs[1].push(arc),
            }
        }
        //dbg!(&sorted_arcs);
        Ok(sorted_arcs)
    }
    pub fn inverse(&self) -> Arc {
        return Arc {
            circle: -self.circle,
            boundary: match self.boundary {
                None => None,
                Some(x) => Some(-x),
            },
        };
    }
    //helper for in_circle
    pub fn contains_either_properly(&self, pair: [Point; 2]) -> Result<bool, String> {
        //REWORK ALL
        for p in pair {
            if self.contains(p.into())? == Some(Contains::Inside) {
                //dbg!(p);
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn rotate(&self, rot: Rotoflector) -> Arc {
        Arc {
            boundary: match self.boundary {
                None => None,
                Some(x) => Some(rot.sandwich(x).rescale_oriented()),
            },
            circle: rot.sandwich(self.circle).rescale_oriented(),
        }
    }
    //None -- the arc crosses the circles boundary
    //Border -- the arc is on the circle
    //Inside/Outside -- arc endpoints can be on the boundary
    //potential useful precondition -- the arc does not cross the boundary, only touches it. should be sufficient for cutting, however not sufficient for bandaging reasons
    pub fn in_circle(&self, circle: Blade3) -> Result<Option<Contains>, String> {
        // let arc_circle = self.circle;
        // let circ = circle;
        if
        //(circle
        // .rescale_oriented()
        // .approx_eq(&self.circle.rescale_oriented(), PRECISION))
        // || (circle
        //     .rescale_oriented()
        //     .approx_eq(&-self.circle.rescale_oriented(), PRECISION))
        //||
        (circle & self.circle).approx_eq_zero(SQRT_PRECISION) {
            return Ok(Some(Contains::Border));
        }
        let intersect = circle & self.circle;
        match intersect.unpack_with_prec(PRECISION) {
            Dipole::Real(real_intersect) => {
                if self.boundary == None || self.contains_either_properly(real_intersect)? {
                    return Ok(None);
                }
                let boundary_points = match self.boundary.unwrap().unpack() {
                    Dipole::Real(points) => points,
                    _ => {
                        // dbg!(self.boundary.unwrap().unpack());
                        // dbg!(self.boundary.unwrap().mag2());
                        return Err(
                            "Arc.in_circle failed: arc boundary was tangent or imaginary!"
                                .to_string(),
                        );
                    }
                };
                let contains = [
                    circle_contains(circle, boundary_points[0].into()),
                    circle_contains(circle, boundary_points[1].into()),
                ];
                return match contains {
                    [Contains::Inside, Contains::Inside]
                    | [Contains::Inside, Contains::Border]
                    | [Contains::Border, Contains::Inside] => Ok(Some(Contains::Inside)),
                    [Contains::Outside, Contains::Outside]
                    | [Contains::Border, Contains::Outside]
                    | [Contains::Outside, Contains::Border] => Ok(Some(Contains::Outside)),
                    [Contains::Border, Contains::Border] => {
                        match real_intersect[0].approx_eq(&boundary_points[0], SQRT_PRECISION) {
                            true => Ok(Some(Contains::Inside)),
                            false => Ok(Some(Contains::Outside)),
                        }
                    }
                    _ => {
                        // dbg!(contains);
                        // dbg!(match self.boundary.unwrap().unpack() {
                        //     Dipole::Real(real) => real,
                        //     _ => return Err(""),
                        // });
                        // //dbg!(self.contains_either_properly(circle & self.circle));
                        // dbg!(self.contains(real_intersect[0].into()));
                        // dbg!(self.contains(real_intersect[1].into()));
                        // dbg!(match self.circle.unpack() {
                        //     Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                        //     _ => panic!("Lmao"),
                        // });
                        // dbg!(match circle.unpack() {
                        //     Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                        //     _ => panic!("Lmao"),
                        // });
                        return Err(
                            "Arc.in_circle failed: arc did not contain either intersection point properly but its boundary crossed the border!".to_string()
                        );
                    }
                };
            }
            _ => Ok(Some(circ_border_inside_circ(circle, self.circle))),
        }
        // let intersect = circ & arc_circle;
        // match intersect.unpack() {
        //     Dipole::Real(real) => {
        //         if self.contains_either_properly(intersect) {
        //             return None;
        //         }
        //         //FLIP SIGN MAYBE
        //         let bound_points = match self.boundary?.unpack() {
        //             Dipole::Real(r) => r,
        //             _ => {
        //                 dbg!(self.boundary.unwrap().mag2());
        //                 dbg!(self.boundary);
        //                 dbg!(self.boundary.unwrap().unpack());
        //                 panic!("schlimble")
        //             }
        //         };
        //         let contains = [
        //             circle_contains(circ, bound_points[0].into()),
        //             circle_contains(circ, bound_points[1].into()),
        //         ];
        //         return match contains {
        //             [Contains::Inside, Contains::Inside]
        //             | [Contains::Inside, Contains::Border]
        //             | [Contains::Border, Contains::Inside] => Some(Contains::Inside),
        //             [Contains::Outside, Contains::Outside]
        //             | [Contains::Outside, Contains::Border]
        //             | [Contains::Border, Contains::Outside] => Some(Contains::Outside),
        //             [Contains::Border, Contains::Border] => Some(
        //                 //SIGN NEEDS CHECKING
        //                 match real[0].approx_eq(
        //                     &match self.boundary?.unpack() {
        //                         Dipole::Real(real_boundary) => real_boundary[0],
        //                         _ => panic!("terrorism"),
        //                     },
        //                     PRECISION,
        //                 ) {
        //                     false => Contains::Outside,
        //                     true => Contains::Inside,
        //                 },
        //             ),
        //             _ => {
        //                 dbg!(self);
        //                 dbg!(circ);
        //                 dbg!(
        //                     dbg!(
        //                         -(self.boundary.unwrap() ^ Into::<Blade1>::into(real[0]))
        //                             << self.circle
        //                     )
        //                     .approx_eq(&0.0, PRECISION)
        //                 );
        //                 dbg!(3.2195042811735317e-5.approx_eq(&0.0, PRECISION));
        //                 panic!("what have you done.")
        //             }
        //         };
        //     }
        //     _ => Some(circ_border_inside_circ(circ, arc_circle)),
        // }
    }
}
