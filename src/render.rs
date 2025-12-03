use crate::PRECISION;
use crate::arc::*;
use crate::circle_utils::*;
use crate::data_storer::DataStorer;
use crate::data_storer::PuzzleData;
use crate::piece::*;
use crate::puzzle::*;
use crate::turn::*;
use approx_collections::*;
use cga2d::*;
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2, RichText,
    epaint::{self, PathShape, FontId},
    pos2, vec2,
};
use std::cmp::*;
use std::f32::consts::PI;
fn aeq_pos(p1: Pos2, p2: Pos2) -> bool {
    p1.x.approx_eq(&p2.x, PRECISION) && p1.y.approx_eq(&p2.y, PRECISION)
}
///the default rendering color
const DETAIL: f64 = 0.5;
///the color of the outlines
const OUTLINE_COLOR: Color32 = Color32::BLACK;

///draws a the circumference of a circle given the coordinates
pub fn draw_circle(real_circle: Blade3, ui: &mut Ui, rect: &Rect, scale_factor: f32, offset: Vec2) {
    if let Circle::Circle {
        cx: x,
        cy: y,
        r,
        ori: _,
    } = real_circle.unpack()
    //get the circle
    {
        ui.painter().circle_stroke(
            to_egui_coords(pos2(x as f32, y as f32), &rect, scale_factor, offset),
            r as f32 * scale_factor * (rect.width() / 1920.0),
            (10.0, Color32::WHITE),
        );
    }
}
///the amount more detailed the outlines are than the interiors
const DETAIL_FACTOR: f64 = 3.0;
///take in a triangle and return if its 'almost degenerate' within some leniency (i.e. its points are 'almost colinear')
fn almost_degenerate(triangle: &Vec<Pos2>, leniency: f32) -> bool {
    let angle_1 = (triangle[1] - triangle[0]).angle() - (triangle[1] - triangle[2]).angle(); //get the relevant (smallest/largest) angle of the triangle, by construction
    let close = angle_1.abs().min((PI - angle_1).abs()); //find how close it is to either extreme (0 or PI)
    if close < leniency {
        return true;
    }
    return false;
}

///averages (takes the barycenter) of a vec of points
fn avg_points(points: &Vec<Pos2>) -> Pos2 {
    let n = points.len() as f32;
    let mut pos = pos2(0.0, 0.0);
    for point in points {
        pos.x += point.x / n;
        pos.y += point.y / n;
    }
    return pos;
}

///translates from cga2d coords to egui coords
fn to_egui_coords(pos: Pos2, rect: &Rect, scale_factor: f32, offset: Vec2) -> Pos2 {
    return pos2(
        ((pos.x + offset.x) as f32) * (scale_factor * rect.width() / 1920.0)
            + (rect.width() / 2.0)
            + rect.min.x,
        -1.0 * ((pos.y + offset.y) as f32) * (scale_factor * rect.width() / 1920.0)
            + (rect.height() / 2.0)
            + rect.min.y,
    );
}

///translates from egui coords to cga2d coords
fn from_egui_coords(pos: &Pos2, rect: &Rect, scale_factor: f32, offset: Vec2) -> Pos2 {
    return pos2(
        ((pos.x - (rect.width() / 2.0)) * (1920.0 / (scale_factor * rect.width()))) - offset.x,
        ((pos.y - (rect.height() / 2.0)) * (-1920.0 / (scale_factor * rect.width()))) - offset.y,
    );
}

///rotate a point about a point a certain angle
fn rotate_about(center: Pos2, point: Pos2, angle: f32) -> Pos2 {
    if aeq_pos(center, point) {
        return point;
    }
    let dist = center.distance(point);
    let curr_angle = (point - center).angle();
    let end_angle = angle + curr_angle;
    return pos2(
        center.x + (dist * end_angle.cos()),
        center.y + (dist * end_angle.sin()),
    );
}
///get the euclidian center and radius of some cga2d circle
///panics if passed a line/imaginary circle
fn euc_center_rad(circ: Blade3) -> Result<(Pos2, f32), String> {
    return match circ.unpack() {
        Circle::Circle { cx, cy, r, ori: _ } => Ok((pos2(cx as f32, cy as f32), r as f32)),
        _ => {
            dbg!(circ);
            Err("euc_center_rad failed: A line or imaginary circle was passed!".to_string())
        }
    };
}
impl Arc {
    ///draws an arc (the outline) in OUTLINE_COLOR, according to the parameters passed
    fn draw(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        width: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        let size =
            self.angle_euc()?.abs() as f32 * euc_center_rad(self.circle)?.1 * DETAIL_FACTOR as f32; //get the absolute size of the arc, to measure how finely we need to render it
        let divisions = (size * detail * DETAIL as f32).max(2.0) as u16; //find the number of divisions we do for the arc
        let mut coords = Vec::new();
        for pos in self.get_polygon(divisions)? {
            //divide up the arc into a polygon and convert to egui
            coords.push(to_egui_coords(pos, rect, scale_factor, offset_pos));
        }
        ui.painter() //paint the path along the polygon
            .add(PathShape::line(coords, Stroke::new(width, OUTLINE_COLOR)));
        Ok(())
    }
    ///gets the polygon representation of an arc for rendering its outline and for triangulation
    fn get_polygon(&self, divisions: u16) -> Result<Vec<Pos2>, String> {
        let mut points: Vec<Pos2> = Vec::new();
        let start_point = match self.boundary {
            //pick an arbitrary start point
            None => euc_center_rad(self.circle)?.0 + vec2(0.0, euc_center_rad(self.circle)?.1),
            Some(b2) => {
                if let Dipole::Real(real) = b2.unpack()
                    && let Point::Finite([x, y]) = real[0]
                {
                    pos2(x as f32, y as f32)
                } else {
                    return Err(
                        "Arc.get_polygon failed: Arc boundary was infinite or not real!"
                            .to_string(),
                    );
                }
            }
        };
        let angle = self.angle_euc()? as f32; //take the angle of the arc
        let inc_angle = angle / (divisions as f32);
        points.push(start_point);
        for i in 1..=divisions {
            //increment the angle and take points
            points.push(rotate_about(
                euc_center_rad(self.circle)?.0,
                start_point,
                inc_angle * (i as f32),
            ));
        }
        return Ok(points);
    }
    ///triangulate the arc with respect to a given center
    fn triangulate(&self, center: Pos2, detail: f32) -> Result<Vec<Vec<Pos2>>, String> {
        let size = self.angle_euc()?.abs() as f32 * euc_center_rad(self.circle)?.1;
        let div = (detail * size * DETAIL as f32).max(2.0) as u16; //get the absolute size and use it to determine the level of detail
        let polygon = self.get_polygon(div)?;
        let mut triangles = Vec::new();
        for i in 0..(polygon.len() - 1) {
            //use the polygon to divide into triangles
            triangles.push(vec![center, polygon[i], polygon[i + 1]]);
        }
        Ok(triangles)
    }
    ///get the euclidian angle of the arc. clockwise arcs are negative by convention
    fn angle_euc(&self) -> Result<f32, String> {
        let orientation = circle_orientation_euclid(self.circle) == Contains::Inside; //get the orientation of the arc's circle as a bool
        if self.boundary == None {
            return if orientation {
                Ok(2.0 * PI)
            } else {
                Ok(-2.0 * PI)
            }; //if there is no boundary, we can immediately say the angle is ori * 2PI
        } else {
            let Dipole::Real([p1, p2]) = self.boundary.unwrap().unpack() else {
                //unpack the boundary
                return Err(
                    "Arc.angle_euc failed: arc.boundary was tangent or imaginary!".to_string(),
                );
            };
            let (pos1, pos2) = match (p1, p2) {
                //get the points as egui poins
                (Point::Finite([x1, y1]), Point::Finite([x2, y2])) => {
                    (pos2(x1 as f32, y1 as f32), pos2(x2 as f32, y2 as f32))
                }
                _ => {
                    return Err(
                        "Arc.angle_euc failed: arc.boundary had infinite endpoint(s)!".to_string(),
                    );
                }
            };
            let center = euc_center_rad(self.circle)?.0; //do angle math
            let angle = ((pos2 - center).angle() - (pos1 - center).angle()).rem_euclid(2.0 * PI);
            if orientation {
                Ok(angle)
            } else {
                Ok(angle - (2.0 * PI))
            }
        }
    }
    ///get the euclidian midpoint of the arc
    fn midpoint_euc(&self) -> Result<Option<Pos2>, String> {
        let p = match (match self.boundary {
            //unpack the boundary
            None => return Ok(None),
            Some(x) => x,
        })
        .unpack()
        {
            Dipole::Real(real) => match real[0] {
                //get the start point of the arc
                Point::Finite([x, y]) => pos2(x as f32, y as f32),
                _ => {
                    return Err(
                        "Arc.midpoint_euc failed: Arc has an infinite endpoint!".to_string()
                    );
                }
            },
            _ => return Err("Arc.midpoint_euc failed: arc.boundary was not real!".to_string()),
        };
        Ok(Some(rotate_about(
            //rotate the start point by half the euclidian angle
            euc_center_rad(self.circle)?.0,
            p,
            self.angle_euc()? as f32 / 2.0,
        )))
    }
}
///render a piece, with an outline
impl Piece {
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        offset: Option<Turn>,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
        correct: bool,
    ) -> Result<(), String> {
        //get the offset of the piece, base on if its in the animation_offset circle
        let true_offset = if offset.is_none()
            || self
                .shape
                .in_circle(offset.unwrap().circle)?
                .is_some_and(|x| x == Contains::Inside)
        {
            offset
        } else {
            None
        };
        let true_piece = if let Some(twist) = true_offset {
            //turn the piece around the offset
            self.turn(twist)?.unwrap_or(self.clone())
        } else {
            self.clone()
        };
        for comp in true_piece.get_components(correct)? {
            //get the components and render those
            comp.render(ui, rect, detail, outline_size, scale_factor, offset_pos)?;
        }
        Ok(())
    }
}
impl Component {
    ///render a component of a piece
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        let triangulation = self.triangulate(self.barycenter()?, detail)?; //triangulate the component around its barycenter
        let mut triangle_vertices: Vec<epaint::Vertex> = Vec::new(); //make a new vector of epaint vertices
        for triangle in triangulation {
            //iterate over the triangles
            if !almost_degenerate(&triangle, 0.0) {
                //remove the degenerate ones
                for point in triangle {
                    let vertex = epaint::Vertex {
                        pos: to_egui_coords(point, rect, scale_factor, offset_pos),
                        uv: pos2(0.0, 0.0),
                        color: self.color,
                    };
                    triangle_vertices.push(vertex); //add the nondegenerate triangle vertices
                }
            }
        }
        let mut mesh = epaint::Mesh::default(); //make a new mesh
        mesh.indices = (0..(triangle_vertices.len() as u32)).collect();
        mesh.vertices = triangle_vertices; //add all the vertices
        ui.painter().add(egui::Shape::Mesh(mesh.into())); //paint the triangles
        self.draw_outline(ui, rect, detail, outline_size, scale_factor, offset_pos)?; //draw the outline
        Ok(())
    }
    ///returns a list of triangles for rendering
    fn triangulate(&self, center: Pos2, detail: f32) -> Result<Vec<Vec<Pos2>>, String> {
        let mut triangles = Vec::new();
        for arc in &self.shape.border {
            //triangulate each arc by the center
            triangles.extend(arc.triangulate(center, detail)?);
        }
        return Ok(triangles);
    }
    ///get the barycenter of the piece based on the arcs for triangulation
    fn barycenter(&self) -> Result<Pos2, String> {
        let mut points = Vec::new();
        for arc in &self.shape.border {
            if let Some(x) = arc.midpoint_euc()? {
                points.push(x);
            };
        }
        if points.is_empty() {
            return Ok(euc_center_rad(self.shape.border[0].circle)?.0);
        }
        return Ok(avg_points(&points)); //average the midpoints of the arcs
    }
    ///draw the outline of the component
    fn draw_outline(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        for arc in &self.shape.border {
            //draw the outline of each arc
            arc.draw(ui, rect, detail, outline_size, scale_factor, offset_pos)?;
        }
        Ok(())
    }
}
impl Puzzle {
    ///render the puzzle, including outlines
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        outline_width: f32,
        scale_factor: f32,
        offset: Vec2,
        correct: bool,
    ) -> Result<(), String> {
        let proper_offset = if let Some(off) = self.animation_offset {
            //get the offset from the animation_offset and anim_left
            Some(Turn {
                circle: off.circle,
                rotation: self.anim_left as f64 * off.rotation
                    + (1.0 - self.anim_left) as f64 * Rotoflector::ident(),
            })
        } else {
            None
        };
        for piece in &self.pieces {
            //render each piece
            piece.render(
                ui,
                rect,
                proper_offset,
                detail,
                outline_width,
                scale_factor,
                offset,
                correct,
            )?;
        }
        Ok(())
    }
    ///processes a click input and does the corresponding turns
    ///Ok(true) means the turn was completed
    ///Ok(false) means that the turn was bandanged, or no turn was found
    ///Err(e) means that an error was encountered
    ///'cut' is whether the turn should cut
    pub fn process_click(
        &mut self,
        rect: &Rect,
        pos: Pos2,
        left: bool,
        scale_factor: f32,
        offset: Vec2,
        cut: bool,
    ) -> Result<bool, String> {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset); //the cga2d position of the click
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_id: String = String::from("");
        for turn in &self.base_turns {
            //iterate over the turns to find the closest one
            let (center, radius) = match turn.1.circle.unpack() {
                Circle::Circle { cx, cy, r, ori: _ } => (pos2(cx as f32, cy as f32), r as f32),
                _ => {
                    return Err(
                        "Puzzle.process_click failed: Circle was a line or imaginary!".to_string(),
                    );
                }
            };
            //compare how close they are
            //ties are broken by the radius, smaller radius gets priority (so that concentric circles work)
            if ((good_pos.distance(center).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.distance(center).approx_eq(&min_dist, PRECISION))
                    && (radius.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && circle_contains(turn.1.circle, point(good_pos.x as f64, good_pos.y as f64))
                    == Contains::Inside
            {
                min_dist = good_pos.distance(center);
                min_rad = radius;
                correct_id = turn.0.clone();
            }
        }
        if correct_id.is_empty() {
            //if no circle was found
            return Ok(false);
        }
        if !left {
            //invert based on the type of click
            Ok(self.turn_id(correct_id, cut, 1)?)
        } else {
            Ok(self.turn_id(correct_id, cut, -1)?)
        }
    }
    ///get the circle hovered by the mouse
    pub fn get_hovered(
        &self,
        rect: &Rect,
        pos: Pos2,
        scale_factor: f32,
        offset: Vec2,
    ) -> Result<Option<Blade3>, String> {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset); //get the position
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_turn = None;
        for turn in self.base_turns.clone().values() {
            //this algorithm proceeds very similarly to the process_click algorithm above
            let (cent, rad) = euc_center_rad(turn.circle)?;
            if ((good_pos.distance(cent).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.distance(cent).approx_eq(&min_dist, PRECISION))
                    && (rad.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && good_pos.distance(cent) < rad
            {
                min_dist = good_pos.distance(cent);
                min_rad = rad;
                correct_turn = Some(*turn);
            }
        }
        if min_rad == 10000.0 {
            return Ok(None);
        }
        return Ok(Some(
            match correct_turn {
                None => return Ok(None),
                Some(x) => x,
            }
            .circle,
        ));
    }
}

impl DataStorer {
    ///render the data panel on the screen and read input for which button is clicked
    pub fn render_panel(&mut self, ctx: &egui::Context) -> Result<Option<PuzzleData>, ()> {
        let panel = egui::SidePanel::new(egui::panel::Side::Right, "data_panel").resizable(false); //make the new panel
        let mut puzzle_data = None;
        panel.show(ctx, |ui| {
            ui.label(RichText::new("Puzzles").font(FontId::proportional(20.0)));
            //button to reload the puzzles into the data_storer if they were modifed (doing this every frame is too costly)
            if ui.add(egui::Button::new("Reload Puzzle List")).clicked() {
                let _ = self.load_puzzles(
                    "Puzzles/Definitions/",
                    "Configs/Keybinds/Puzzles/",
                    "Configs/Keybinds/groups.kdl",
                );
            }
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for puz in &self.sorted_puzzles {
                    if ui.add(egui::Button::new(puz.preview.clone())).clicked() {
                        //make the buttons for each puzzle
                        puzzle_data = Some(puz.clone());
                    }
                }
            })
        });
        Ok(puzzle_data)
    }
}
