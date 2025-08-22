use crate::PRECISION;
use crate::circle_utils::*;
use cga2d::*;
//if boundary is None, then the arc is the whole circle
#[derive(Debug, Clone, Copy)]
pub struct Arc {
    pub circle: Blade3,
    pub boundary: Option<Blade2>,
}
impl Arc {
    pub fn contains(&self, point: Blade1) -> Option<Contains> {
        if circle_contains(self.circle, point) != Contains::Border {
            return None;
        }
        if self.boundary == None {
            return Some(Contains::Inside);
        }
        if let Dipole::Real(real) = self.boundary.unwrap().unpack() {
            for p in real {
                if point.unpack().unwrap().approx_eq(&p, PRECISION) {
                    return Some(Contains::Border);
                }
            }
            return Some(contains_from_metric(
                -((self.boundary.unwrap() ^ point) << self.circle),
            ));
        }
        if let Dipole::Tangent(p, d) = self.boundary.unwrap().unpack() {
            dbg!(p);
        }
        panic!("NO!")
    }
    //IF THEY ARE TANGENT, THEN return[1] is always NONE
    pub fn intersect_circle(&self, circle: Blade3) -> [Option<Blade1>; 2] {
        if (circle & self.circle).approx_eq_zero(PRECISION) {
            return [None; 2];
        }
        match (self.circle.rescale_oriented() & circle.rescale_oriented())
            .rescale_oriented()
            .unpack()
        {
            Dipole::Real(int_points) => int_points.map(|a| match self.contains(a.into()) {
                None => None,
                Some(Contains::Outside) => None,
                Some(Contains::Border) | Some(Contains::Inside) => Some(a.into()),
            }),
            Dipole::Tangent(p, _) => match self.contains(p.into()) {
                Some(Contains::Inside) | Some(Contains::Border) => [Some(p.into()), None],
                Some(Contains::Outside) => [None; 2],
                None => {
                    dbg!(p);
                    panic!("This shouldn't be possible")
                }
            },
            _ => [None; 2],
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
    pub fn cut_by_circle(&self, circle: Blade3) -> [Vec<Arc>; 2] {
        //REWORK ALL
        let mut sorted_arcs = [Vec::new(), Vec::new()];
        let mut segments = Vec::new();
        let mut new_points: Vec<Blade1> = Vec::new();
        match (circle & self.circle).rescale_oriented().unpack() {
            Dipole::Real(intersects) => {
                for intersect in intersects {
                    if self.contains(intersect.into()).unwrap() == Contains::Inside {
                        new_points.push(intersect.into());
                    }
                }
                for point in &mut new_points {
                    *point = point.rescale_oriented();
                }
                if new_points.is_empty() {
                    segments = vec![*self];
                } else {
                    let base = new_points[0];
                    new_points.sort_by(|a, b| {
                        comp_points_on_circle(
                            match self.boundary {
                                None => base,
                                Some(x) => match x.unpack() {
                                    Dipole::Real(r) => r[0].into(),
                                    _ => panic!("television"),
                                },
                            },
                            *a,
                            *b,
                            self.circle,
                        )
                    });
                    if let Some(x) = self.boundary {
                        new_points.insert(
                            0,
                            match x.unpack() {
                                Dipole::Real(r) => r[0].into(),
                                _ => panic!("horseplay"),
                            },
                        );
                        new_points.push(match x.unpack() {
                            Dipole::Real(r) => r[1].into(),
                            _ => panic!("chemically"),
                        });
                    } else {
                        new_points.push(base);
                    }
                    //(&new_points);
                    for i in 0..(new_points.len() - 1) {
                        segments.push(Arc {
                            circle: self.circle,
                            boundary: Some((new_points[i] ^ new_points[i + 1]).rescale_oriented()),
                        })
                    }
                }
            }
            _ => segments = vec![*self],
        }
        for arc in segments {
            //dbg!(arc.circle);
            //dbg!(circle);
            match arc.in_circle(circle) {
                None => {
                    dbg!(arc);
                    dbg!(match arc.boundary.unwrap().unpack() {
                        Dipole::Real(r) => r,
                        _ => panic!("hi"),
                    });
                    dbg!(match arc.circle.unpack() {
                        Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                        _ => panic!("hi"),
                    });
                    dbg!(match circle.unpack() {
                        Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                        _ => panic!("hi"),
                    });
                    if arc.intersect_circle(circle)[0].is_some() {
                        dbg!(arc.intersect_circle(circle)[0].unwrap().unpack());
                    }
                    if arc.intersect_circle(circle)[1].is_some() {
                        dbg!(arc.intersect_circle(circle)[0].unwrap().unpack());
                    }
                    dbg!(circle);
                    panic!("whats going on? who are you?")
                }
                Some(Contains::Inside) => sorted_arcs[0].push(arc),
                Some(Contains::Border) => {
                    sorted_arcs[0].push(arc);
                    sorted_arcs[1].push(arc)
                } //in this case the arc is tangent to the circle and on the circle
                Some(Contains::Outside) => sorted_arcs[1].push(arc),
            }
        }
        //dbg!(&sorted_arcs);
        sorted_arcs
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
    pub fn contains_either_properly(&self, pair: Blade2) -> bool {
        //REWORK ALL
        let points = match pair.unpack() {
            Dipole::Real(real) => real,
            _ => panic!("492830948234"),
        };
        for p in points {
            if self.contains(p.into()) == Some(Contains::Inside) {
                //dbg!(p);
                return true;
            }
        }
        false
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
    pub fn in_circle(&self, circle: Blade3) -> Option<Contains> {
        // let arc_circle = self.circle;
        // let circ = circle;
        if (circle
            .rescale_oriented()
            .approx_eq(&self.circle.rescale_oriented(), PRECISION))
            || (circle
                .rescale_oriented()
                .approx_eq(&-self.circle.rescale_oriented(), PRECISION))
        {
            return Some(Contains::Border);
        }
        let intersect = (circle & self.circle).rescale_oriented();
        match intersect.unpack() {
            Dipole::Real(real_intersect) => {
                if self.contains_either_properly(intersect) {
                    return None;
                }
                let boundary_points = match self.boundary.unwrap().unpack() {
                    Dipole::Real(points) => points,
                    _ => {
                        dbg!(self.boundary.unwrap().unpack());
                        dbg!(self.boundary.unwrap().mag2());
                        panic!("Boundary was tangent!")
                    }
                };
                let contains = [
                    circle_contains(circle, boundary_points[0].into()),
                    circle_contains(circle, boundary_points[1].into()),
                ];
                return match contains {
                    [Contains::Inside, Contains::Inside]
                    | [Contains::Inside, Contains::Border]
                    | [Contains::Border, Contains::Inside] => Some(Contains::Inside),
                    [Contains::Outside, Contains::Outside]
                    | [Contains::Border, Contains::Outside]
                    | [Contains::Outside, Contains::Border] => Some(Contains::Outside),
                    [Contains::Border, Contains::Border] => {
                        match real_intersect[0].approx_eq(&boundary_points[0], PRECISION) {
                            true => Some(Contains::Inside),
                            false => Some(Contains::Outside),
                        }
                    }
                    _ => {
                        dbg!(contains);
                        dbg!(match self.boundary.unwrap().unpack() {
                            Dipole::Real(real) => real,
                            _ => panic!(""),
                        });
                        dbg!(self.contains_either_properly(circle & self.circle));
                        dbg!(self.contains(real_intersect[0].into()));
                        dbg!(self.contains(real_intersect[1].into()));
                        dbg!(match self.circle.unpack() {
                            Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                            _ => panic!("Lmao"),
                        });
                        dbg!(match circle.unpack() {
                            Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                            _ => panic!("Lmao"),
                        });
                        panic!("CIRCLE DID NOT INTERSECT BUT CROSSED")
                    }
                };
            }
            _ => Some(circ_border_inside_circ(circle, self.circle)),
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
