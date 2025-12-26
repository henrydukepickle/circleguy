use crate::LOW_PRECISION;
use crate::PRECISION;
use crate::arc::*;
use crate::circle_utils::*;
use crate::turn::*;
use approx_collections::*;
use cga2d::*;

pub type BoundingCircles = Vec<Blade3>;
pub type BoundaryShape = Vec<Arc>;
#[derive(Debug, Clone)]
///the shape of a piece (a CCP)
pub struct PieceShape {
    pub bounds: BoundingCircles, //the ccp representation, a series of (oriented) circles whose intersection yields the piece
    pub border: BoundaryShape,   //the arcs representing the border of the piece
}
#[derive(Debug, Clone)]
///a connected component for rendering in fine mode
pub struct ComponentShape {
    pub border: BoundaryShape,
}

//return the collapsed CCP representation
pub fn collapse_shape_and_add(
    bounding_circles: &BoundingCircles, // the circles of the representation we want to collapse
    new_circle: Blade3,                 //the new circle to add to the representation
) -> Result<BoundingCircles, String> {
    let mut new_bounding_circles = Vec::new(); //this will store the new representation
    for circ in bounding_circles {
        //iterate over the circles in the representation
        if !circle_excludes(*circ, new_circle) {
            //if the old circle is exlcuded by the new circle, do not add it to the representation
            new_bounding_circles.push(*circ); //add it
        } else {
            //debug some info
            dbg!(match circ.unpack_with_prec(PRECISION) {
                Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                _ =>
                    return Err(
                        "collapse_shape_and_add failed: Circle was a line or imaginary! (1)"
                            .to_string()
                    ),
            });
            dbg!(match new_circle.unpack_with_prec(PRECISION) {
                Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
                _ =>
                    return Err(
                        "collapse_shape_and_add failed: Circle was a line or imaginary! (2)"
                            .to_string()
                    ),
            });
        }
    }
    new_bounding_circles.push(new_circle); //add the new circle
    Ok(new_bounding_circles) //return the new representation
}
impl PieceShape {
    pub fn cut_boundary(&self, circle: Blade3) -> Result<Option<[BoundaryShape; 2]>, String> {
        //cut the boundary of a pieceshape by a circle
        fn get_circle_arcs(circle: Blade3, cut_points: &Vec<Point>) -> Result<Vec<Arc>, String> {
            //cut a circle into a bunch of arcs, cutting by a bunch of points
            if cut_points.len() <= 1 {
                //if there is 1 or 0 cut points, we just return
                return Ok(vec![Arc {
                    circle,
                    boundary: None,
                }]); //just return the circle itself
            }
            let mut points = cut_points.clone(); //clone the cut points
            let base = points.pop().unwrap(); //pick a base point (arbitrary)
            points.sort_by(|a, b| {
                comp_points_on_circle(base.into(), (*a).into(), (*b).into(), circle)
            }); //sort the points along the circle
            let mut arcs = Vec::new(); //this will store the arcs we cut the circle into
            points.insert(0, base); //add the base point back at the start
            points.push(base); //also add the base point to the end
            for i in 0..(points.len() - 1) {
                //for each point pair in the vec points
                arcs.push(Arc {
                    //add a new arc
                    circle, //around the circle
                    boundary: Some(
                        (points[i].to_blade().rescale_unoriented_with_prec(PRECISION)
                            ^ points[i + 1]
                                .to_blade()
                                .rescale_unoriented_with_prec(PRECISION))
                        .rescale_oriented_with_prec(PRECISION), //the point pair representing points[i] to points[i + 1]
                    ),
                });
                if points[i].approx_eq(&points[i + 1], LOW_PRECISION) {
                    //if the two points are two close, then return an error
                    return Err(
                        "PieceShape.cut_boundary failed: cut points were too close!".to_string()
                    );
                }
            }
            Ok(arcs) //return the arcs
        }
        let mut result = [Vec::new(), Vec::new()]; //initialize a result. the 0 index will be inside, and the 1 index will be outside
        let mut cut_points: ApproxHashMap<Point, ()> = ApproxHashMap::new(LOW_PRECISION); //initialize the cut points
        for arc in &self.border {
            if (circle & arc.circle).approx_eq_zero(LOW_PRECISION) {
                return Ok(None);
            }
            for int in (arc.intersect_circle(circle))? {
                //for each intersection of the arc and the circle
                if let Some(point) = int {
                    //if the point is something
                    cut_points.insert(
                        point
                            .rescale_unoriented_with_prec(PRECISION)
                            .unpack_with_prec(PRECISION)
                            .unwrap(),
                        (),
                    ); //insert the point to cut_points
                }
            }
            for i in [0, 1] {
                result[i].extend(arc.cut_by_circle(circle)?[i].clone()); //cut the arc by the circle, and add the results to in and out
            }
        }
        if result[0].is_empty() || result[1].is_empty() {
            return Ok(None); //if all the arcs fell on the inside or outside, return immediately since the piece shouldn't be cut
        }
        let circle_arcs = get_circle_arcs(circle, &cut_points.into_keys().collect())?; //get the circle arcs using the helper function
        for arc in &circle_arcs {
            //for each arc in the circle arcs
            if let Some(x) = arc.boundary //if the boundary exists and is tangent, return an error
                && let Dipole::Tangent(_, _) = x.unpack_with_prec(PRECISION)
            {
                return Err("PieceShape.cut_boundary failed: arc boundary was tangent!".to_string());
            }
            if (self.contains_arc(arc.rescale_oriented()))? == Contains::Inside {
                //if the arc is inside the piece
                result[0].push(arc.rescale_oriented()); //add the arc to the 'inside' shape. this defines the convention of 'inside' vs 'outside'
                result[1].push(arc.rescale_oriented().inverse()); //add the inverse of the arc to the outside shape
            }
        }

        Ok(Some(result)) //return the result
    }
    /// cut a shape by a circle, returning two Option<PieceShape> s. the first one is the 'inside' shape and the second is the 'outside'.
    pub fn cut_by_circle(&self, circle: Blade3) -> Result<[Option<PieceShape>; 2], String> {
        let shapes_raw = self.cut_boundary(circle)?; //cut the boundary by the circle
        let shapes = match shapes_raw {
            //check if the shape was actually cut
            Some(x) => x, //if it was cut, set shapes to the resulting shapes
            None => {
                //if it wasn't cut, return a single piece either inside or outside
                return match self.in_circle(circle)? {
                    //if the piece crosses the border, debug some info and return Err
                    None => {
                        dbg!(self.border.len());
                        for arc in &self.border {
                            match arc.circle.unpack_with_prec(PRECISION) {
                                Circle::Circle { cx, cy, r, ori } => {
                                    dbg!((cx, cy, r, ori));
                                }
                                _ => {
                                    dbg!("a");
                                }
                            }
                        }
                        Err("PieceShape.cut_by_circle failed: piece_shape was cut, but still blocked the turn!".to_string())
                    }
                    Some(Contains::Inside) => Ok([Some(self.clone()), None]), //if its inside, return it inside
                    Some(Contains::Outside) => Ok([None, Some(self.clone())]), //likewise for outside
                    Some(Contains::Border) => Err(
                        "PieceShape.cut_by_circle failed: piece_shape is on border of circle!"
                            .to_string(), //if its on the border, return an error as well
                    ),
                };
            }
        };
        let bounding_circles = [
            //get the bounding circles for inside and outside
            collapse_shape_and_add(&self.bounds, circle)?, //the inside bounding circles
            collapse_shape_and_add(&self.bounds, -circle)?, //the outside bounding circles
        ];
        let inside = PieceShape {
            //make a new shape for the inside
            border: shapes[0].clone(), //border is the inside shape border
            bounds: bounding_circles[0].clone(), //bounds are the bounds containing the inside circle
        };
        let outside = PieceShape {
            //analogous for outside
            border: shapes[1].clone(),
            bounds: bounding_circles[1].clone(),
        };
        return Ok([Some(inside), Some(outside)]); //return the shapes
    }
    ///detect if a shape is in a circle. Some(x) means that the shape is entirely x, None means that the shape crosses the border of the circle
    pub fn in_circle(&self, circle: Blade3) -> Result<Option<Contains>, String> {
        let mut inside = None; //tracks whether the piece is inside the circle
        for arc in &self.border {
            //iterate over the border arcs. essentially, if all of them are in or out, return Some(in or out), otherwise we return none.
            let contained = match arc.in_circle(circle)? {
                //match if the arc is in the circle. if it crosses the border, we can immediately return None
                None => return Ok(None),
                Some(x) => x,
            };
            if let Some(real_inside) = inside {
                //if the arc has been decided as in or out, check the value of contained against (in or out).
                if contained != Contains::Border && real_inside != contained {
                    //if the values differ, and we arent examining the border case, return that it crosses.
                    return Ok(None);
                }
            } else if contained != Contains::Border {
                //if the piece is undecided on in/out and the arc lies properly in/out, set inside.
                inside = Some(contained);
            }
        }
        if inside.is_none_or(|x| x == Contains::Border) {
            //once all arcs have been iterated, if the value of inside was never set (i.e., all arcs lied on the border) return inside.
            //This decides a convention
            return Ok(Some(Contains::Inside));
        }
        return Ok(inside); //return in/out
    }
    ///turn a pieceshape according to a turn
    ///Ok(None) means that the pieceshape blocks the turn
    pub fn turn(&self, turn: Turn) -> Result<Option<PieceShape>, String> {
        //if the pieceshape blocks the turn, return none
        if match self.in_circle(turn.circle)? {
            None => return Ok(None),
            Some(x) => x,
        } == Contains::Outside
        //if the piece is outside the turn, do not turn and return the piece
        {
            return Ok(Some(self.clone()));
        }
        let mut new_border = Vec::new();
        for arc in &self.border {
            //rotate all the arcs according to the turn
            new_border.push(arc.rotate(turn.rotation));
        }
        let mut new_bounds = Vec::new();
        for bound in &self.bounds {
            //also rotate the bounds
            new_bounds.push(turn.rotation.sandwich(*bound));
        }
        Ok(Some(PieceShape {
            bounds: new_bounds,
            border: new_border,
        })) //return the new piece
    }
    ///rotate a pieceshape according to a rotoflector
    pub fn rotate(&self, rotation: Rotoflector) -> Self {
        let mut new_border = Vec::new();
        for arc in &self.border {
            //rotate all the arcs in the border
            new_border.push(arc.rotate(rotation));
        }
        let mut new_bounds = Vec::new();
        for bound in &self.bounds {
            //rotate the bounds as well
            new_bounds.push(rotation.sandwich(*bound));
        }
        Self {
            bounds: new_bounds,
            border: new_border,
        }
    }
    ///turn a piece along a turn and cut the piece along that turn
    /// like with other cutting functions, returns two Option<pieceshape>s
    /// the first index is the 'inside' piece and the second is the 'outside'
    /// None is in either index if the resulting piece doesnt exist (i.e., the piece was fully inside/outside of the circle)
    pub fn turn_cut(&self, turn: Turn) -> Result<[Option<PieceShape>; 2], String> {
        let mut cut_bits = self.cut_by_circle(turn.circle)?; //cut the pieceshape by the circle
        if let Some(x) = &cut_bits[0] {
            cut_bits[0] = Some(x.rotate(turn.rotation)); //rotate the inside piece according to the turn
        }
        Ok(cut_bits)
    }
    ///return if a pieceshape contains a given arc
    ///if the arc crosses the border, returns Ok(Outside)
    fn contains_arc(&self, arc: Arc) -> Result<Contains, String> {
        for circle in &self.bounds {
            //check the arc against all the bounding circles
            let cont = arc.in_circle(*circle)?;
            if cont == None || cont == Some(Contains::Outside) {
                //if the arc is outside one of the bounding circles or crosses it, return Outside
                return Ok(Contains::Outside);
            }
        }
        Ok(Contains::Inside) //otherwise return inside
    }
    ///get the components of the pieceshape for fine mode rendering. in the case that correct is false, just returns a single 'component' (the entire piece)
    pub fn get_components(&self, correct: bool) -> Result<Vec<ComponentShape>, String> {
        if !correct {
            //return a single component
            return Ok(vec![ComponentShape {
                border: self.border.clone(),
            }]);
        }
        let mut comps = Vec::new(); //initialize a new vec for the components
        let mut remaining_arcs = self.border.clone(); //get all the arcs
        loop {
            //essentially, whenever we complete a component, we pick a new starting point and make a new component
            if remaining_arcs.is_empty() {
                //if there are no arcs left, return
                break;
            }
            let mut curr_arc = remaining_arcs.pop().unwrap(); //start a new components
            let mut curr_comp = vec![curr_arc]; //initialize the component
            loop {
                if curr_arc.boundary.is_none() {
                    //if the boundary is None, this component is done
                    break;
                }
                if let Some(next) = next_arc(&self.border, curr_arc) {
                    //get the next arc in the component and add it
                    curr_arc = next;
                    curr_comp.push(curr_arc);
                } else {
                    return Err("PieceShape.get_components failed: No next arc found!".to_string());
                }
                if let Some(x) = curr_arc.boundary //if we've reached the starting point of this component, break and make a new component
                    && let Dipole::Real(pair) = x.unpack_with_prec(PRECISION)
                    && let Some(y) = curr_comp[0].boundary
                    && let Dipole::Real(base_pair) = y.unpack_with_prec(PRECISION)
                    && pair[1].approx_eq(&base_pair[0], LOW_PRECISION)
                {
                    break;
                }
            }
            comps.push(ComponentShape { border: curr_comp }); //add the component
        }
        Ok(comps)
    }
}
///get the next arc from a series of arcs (this simply returns the first arc it finds with curr.end = arc.start)
pub fn next_arc(bound: &BoundaryShape, curr: Arc) -> Option<Arc> {
    for arc in bound {
        //iterate over the arcs
        if let Some(boundary) = arc.boundary
            && let Dipole::Real(real) = boundary.unpack_with_prec(PRECISION)
            && let Dipole::Real(real_curr) = curr.boundary?.unpack_with_prec(PRECISION)
            && (real_curr[1].approx_eq(&real[0], LOW_PRECISION))
        {
            //if the end point of curr is the start point of arc, return
            return Some(*arc);
        }
    }
    None
}
