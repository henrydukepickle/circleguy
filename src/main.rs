//#![windows_subsystem = "windows"]
use std::{
    cmp::Ordering,
    collections::HashMap,
    f32::consts::PI,
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    vec,
};

use kdl::KdlDocument;
use rand::prelude::*;

use egui::{
    Color32, Event, MouseWheelUnit, Pos2, Rect, ScrollArea, Stroke, Ui,
    epaint::{self, PathShape},
    pos2,
};

#[cfg(not(target_arch = "wasm32"))]
const DEV: bool = true;

const DETAIL: f64 = 50.0;

const OUTLINE_COLOR: Color32 = Color32::BLACK;

// const SPECTRUM: colorous::Gradient = colorous::TURBO;

const SCALE_FACTOR: f32 = 200.0;

const ANIMATION_SPEED: f64 = 5.0;

const LENIENCY: f64 = 0.00002;

const NONE_COLOR: Color32 = Color32::GRAY;

const NONE_CIRCLE: Circle = Circle {
    center: pos2_f64(0.0, 0.0),
    radius: 0.0,
};

const NONE_TURN: Turn = Turn {
    circle: NONE_CIRCLE,
    angle: 0.0,
};

const DETAIL_FACTOR: f64 = 3.0; // the amount more detailed the outlines are than the interiors

fn get_default_color_hash() -> HashMap<String, Color32> {
    let mut hash = HashMap::new();
    hash.insert("RED".to_string(), Color32::RED);
    hash.insert("BLUE".to_string(), Color32::BLUE);
    hash.insert("GREEN".to_string(), Color32::GREEN);
    hash.insert("YELLOW".to_string(), Color32::YELLOW);
    hash.insert("PURPLE".to_string(), Color32::PURPLE);
    hash.insert("GRAY".to_string(), Color32::GRAY);
    hash.insert("BLACK".to_string(), Color32::BLACK);
    hash.insert("BROWN".to_string(), Color32::BROWN);
    hash.insert("CYAN".to_string(), Color32::CYAN);
    hash.insert("WHITE".to_string(), Color32::WHITE);
    hash.insert("DARK_BLUE".to_string(), Color32::DARK_BLUE);
    hash.insert("DARK_GREEN".to_string(), Color32::DARK_GREEN);
    hash.insert("DARK_GRAY".to_string(), Color32::DARK_GRAY);
    hash.insert("DARK_RED".to_string(), Color32::DARK_RED);
    hash.insert("LIGHT_BLUE".to_string(), Color32::LIGHT_BLUE);
    hash.insert("LIGHT_GRAY".to_string(), Color32::LIGHT_GRAY);
    hash.insert("LIGHT_GREEN".to_string(), Color32::LIGHT_GREEN);
    hash.insert("LIGHT_YELLOW".to_string(), Color32::LIGHT_YELLOW);
    hash.insert("LIGHT_RED".to_string(), Color32::LIGHT_RED);
    hash.insert("KHAKI".to_string(), Color32::KHAKI);
    hash.insert("GOLD".to_string(), Color32::GOLD);
    hash.insert("MAGENTA".to_string(), Color32::MAGENTA);
    hash.insert("ORANGE".to_string(), Color32::ORANGE);
    hash
}

#[derive(Clone)]
struct FloatIntern {
    //unpaid
    floats: Vec<f64>,
    leniency: f64,
}

#[derive(Clone, Copy)]
struct Pos2F64 {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Vec2F64 {
    x: f64,
    y: f64,
}

type Cut = Vec<Turn>;
type Coloring = (Vec<(Circle, Contains)>, Color32);

const fn pos2_f64(x: f64, y: f64) -> Pos2F64 {
    return Pos2F64 { x, y };
}

const fn vec2_f64(x: f64, y: f64) -> Vec2F64 {
    return Vec2F64 { x, y };
}

const fn subtract_pos2_f64(first: Pos2F64, last: Pos2F64) -> Vec2F64 {
    return vec2_f64(first.x - last.x, first.y - last.y);
}

const fn scale_vec2_f64(scalar: f64, vec: Vec2F64) -> Vec2F64 {
    return vec2_f64(vec.x * scalar, vec.y * scalar);
}

const fn add_vec_to_pos2_f64(pos: Pos2F64, vec: Vec2F64) -> Pos2F64 {
    return pos2_f64(pos.x + vec.x, pos.y + vec.y);
}

const fn add_vec_to_vec_f64(v1: Vec2F64, v2: Vec2F64) -> Vec2F64 {
    return vec2_f64(v1.x + v2.x, v1.y + v2.y);
}
auto_ops::impl_op!(-|a: Pos2F64, b: Pos2F64| -> Vec2F64 { subtract_pos2_f64(a, b) });
auto_ops::impl_op!(+|a: Pos2F64, b: Vec2F64| -> Pos2F64 { add_vec_to_pos2_f64(a, b) });
auto_ops::impl_op!(*|a: f64, b: Vec2F64| -> Vec2F64 { scale_vec2_f64(a, b) });
auto_ops::impl_op!(+|a: Vec2F64, b: Vec2F64| -> Vec2F64 { add_vec_to_vec_f64(a, b) });
auto_ops::impl_op!(*|a: isize, b: Turn| -> Turn {
    Turn {
        circle: b.circle,
        angle: b.angle * a as f64,
    }
});

fn multiply_turns(a: isize, compound: &Vec<Turn>) -> Vec<Turn> {
    if a < 0 {
        return invert_compound_turn(&multiply_turns(-1 * a, compound));
    } else if a > 0 {
        let mut multiply_turns = multiply_turns(a - 1, compound);
        multiply_turns.extend(compound);
        return multiply_turns;
    } else {
        return Vec::new();
    }
}

fn invert_compound_turn(compound: &Vec<Turn>) -> Vec<Turn> {
    let mut turns = Vec::new();
    for turn in compound.into_iter().rev() {
        turns.push(turn.inverse());
    }
    return turns;
}

impl fmt::Display for Pos2F64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
struct DataStorer {
    data: Vec<(String, String)>, //puzzle preview string, puzzle data string
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Contains {
    Inside,
    Outside,
    Border,
}

#[derive(Clone)]
struct Piece {
    shape: Vec<Arc>,
    color: Color32,
}
#[derive(Clone)]
struct Puzzle {
    name: String,
    authors: Vec<String>,
    pieces: Vec<Piece>,
    turns: HashMap<String, Turn>,
    stack: Vec<String>,
    animation_offset: Turn,
    intern: FloatIntern,
    depth: u16,
    solved_state: Vec<Piece>,
    solved: bool,
}

struct PuzzlePrevData {
    name: String,
    turns: Vec<String>,
}
#[derive(Clone, Copy)]
struct Circle {
    center: Pos2F64,
    radius: f64,
}
#[derive(Clone, Copy)]
struct Turn {
    circle: Circle,
    angle: f64,
}
#[derive(Clone, Copy)]

//orientation: true means that the 'inside' of the arc is the 'inside' of the piece, false means the opposite
struct Arc {
    start: Pos2F64,
    angle: f64,
    circle: Circle,
}

//checking if certain float-based variable types are approximately equal due to precision bs

fn aeq(f1: f64, f2: f64) -> bool {
    return (f1 - f2).abs() <= LENIENCY;
}
fn aleq(f1: f64, f2: f64) -> bool {
    return (f1 - f2) <= LENIENCY;
}

fn alneq(f1: f64, f2: f64) -> bool {
    return aleq(f1, f2) && !aeq(f1, f2);
}

fn aeq_pos(p1: Pos2F64, p2: Pos2F64) -> bool {
    return aeq(p1.x, p2.x) && aeq(p1.y, p2.y);
}

fn aeq_circ(c1: Circle, c2: Circle) -> bool {
    return aeq_pos(c1.center, c2.center) && aeq(c1.radius, c2.radius);
}

fn aeq_arc(a1: Arc, a2: Arc) -> bool {
    return aeq_circ(a1.circle, a2.circle) && aeq(a1.angle, a2.angle);
}

fn aeq_shape(s1: &Vec<Arc>, s2: &Vec<Arc>) -> bool {
    if s1.len() == s2.len() {
        let mut start_index = 0;
        for i in 0..s1.len() {
            if aeq_arc(s1[i], s2[0]) {
                start_index = i;
                break;
            }
        }
        for i in 0..s1.len() {
            if !aeq_arc(s1[(i + start_index) % s1.len()], s2[i]) {
                return false;
            }
        }
        return true;
    }
    return false;
}

fn aeq_piece(p1: &Piece, p2: &Piece) -> bool {
    return p1.color.r() == p2.color.r()
        && p1.color.g() == p2.color.g()
        && p1.color.b() == p2.color.b()
        && aeq_shape(&p1.shape, &p2.shape);
}

fn aeq_pieces(v1: &Vec<Piece>, v2: &Vec<Piece>) -> bool {
    for piece in v1 {
        let mut found = false;
        for piece2 in v2 {
            if aeq_piece(piece, piece2) {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    return true;
}

fn cmp_f64(a: f64, b: f64) -> Ordering {
    if aeq(a, b) {
        return Ordering::Equal;
    }
    if a < b - LENIENCY {
        return Ordering::Less;
    }
    return Ordering::Greater;
}

impl Turn {
    fn inverse(&self) -> Turn {
        return Turn {
            circle: self.circle,
            angle: -1.0 * self.angle,
        };
    }
}

impl Pos2F64 {
    fn distance(&self, other: Pos2F64) -> f64 {
        return (other - *self).magnitude();
    }
    fn distance_sq(&self, other: Pos2F64) -> f64 {
        return (other.x - self.x).powi(2) + (other.y - self.y).powi(2);
    }
}

impl Vec2F64 {
    fn magnitude(&self) -> f64 {
        return ((self.x * self.x) + (self.y * self.y)).sqrt();
    }
    fn normalized(&self) -> Option<Vec2F64> {
        let mag = self.magnitude();
        if aeq(mag, 0.0) {
            return None;
        }
        return Some((1.0 / mag) * *self);
    }
    //returns the angle measured ccw from positive x
    fn angle(&self) -> f64 {
        return self.y.atan2(self.x);
    }
    //ccw
    fn rot90(&self) -> Vec2F64 {
        return vec2_f64(self.y * -1.0, self.x);
    }
}

impl Circle {
    //draw the circle on the ui
    fn draw(&self, ui: &mut Ui, rect: &Rect, scale_factor: f32, offset: Vec2F64) {
        ui.painter().circle_stroke(
            to_egui_coords(&self.center, rect, scale_factor, offset),
            (self.radius as f32) * scale_factor * (rect.width() / 1920.0),
            (9.0, Color32::WHITE),
        );
    }
    //rotate the circle about a point
    fn rotate_about(&self, center: Pos2F64, angle: f64) -> Circle {
        return Circle {
            center: rotate_about(center, self.center, angle),
            radius: self.radius,
        };
    }
    //check if the circle contains a point (including on the boundary). LENIENCY included to account for floating point stuff
    fn contains(&self, point: Pos2F64) -> Contains {
        let dist = self.center.distance(point);
        if alneq(dist, self.radius) {
            return Contains::Inside;
        }
        if alneq(self.radius, dist) {
            return Contains::Outside;
        }
        return Contains::Border;
    }
    //check if a circle contains an arc -- BAD/REWORK
    //None - the arc crosses the border
    fn contains_arc(&self, arc: &Arc) -> Option<Contains> {
        let intersects = arc.intersect_circle(self);
        if intersects.is_none() {
            return Some(Contains::Border);
        } else {
            return match intersects.unwrap().len() {
                0 => Some(self.contains(arc.midpoint())),
                _ => {
                    if is_tangent(arc.circle, *self) {
                        Some(self.contains(arc.start))
                    } else {
                        None
                    }
                }
            };
        }
    }
    // fn touching(&self, circle: Circle) -> bool {
    //     return aleq(
    //         self.center.distance(circle.center),
    //         self.radius + circle.radius,
    //     );
    // }
}

impl Arc {
    fn rotate_about(&self, center: Pos2F64, angle: f64) -> Arc {
        return Arc {
            start: rotate_about(center, self.start, angle),
            angle: self.angle,
            circle: self.circle.rotate_about(center, angle),
        };
    }
    fn end(&self) -> Pos2F64 {
        return rotate_about(self.circle.center, self.start, self.angle);
    }
    fn midpoint(&self) -> Pos2F64 {
        return rotate_about(self.circle.center, self.start, self.angle / 2.0);
    }
    //make the arc go the other way
    fn invert(&self) -> Arc {
        return Arc {
            start: self.end(),
            angle: -1.0 * self.angle,
            circle: self.circle,
        };
    }
    //get the angle of a point on the arc
    fn get_angle(&self, point: Pos2F64) -> f64 {
        let mut angle = angle_on_circle(self.start, point, self.circle);
        if self.angle.is_sign_negative() {
            angle = angle - (2.0 * PI as f64);
        }
        return angle;
    }
    //check if an arc contains a point properly -- returns false on self.start and self.end()
    fn contains_properly(&self, point: Pos2F64) -> bool {
        if self.circle.contains(point) != Contains::Border {
            return false;
        }
        if aeq(self.angle.abs(), 2.0 * PI as f64) {
            return true;
        }
        let angle = angle_on_circle(self.start, point, self.circle);
        if aeq(angle, 2.0 * PI as f64) {
            return false;
        }
        if self.angle >= 0.0 {
            return alneq(angle, self.angle);
        } else {
            return alneq(self.angle, -2.0 * PI as f64 + angle);
        }
    }
    //same as contains_properly, but returns true on self.start
    // fn contains_properly_start(&self, point: Pos2F64) -> bool {
    //     if self.circle.contains(point) != Contains::Border {
    //         return false;
    //     }
    //     if aeq(self.angle.abs(), 2.0 * PI as f64) {
    //         return true;
    //     }
    //     let angle = angle_on_circle(self.start, point, self.circle);
    //     if aeq(angle, 2.0 * PI as f64) {
    //         return true;
    //     }
    //     if aeq(angle, 0.0) {
    //         return true;
    //     }
    //     if self.angle >= 0.0 {
    //         return (alneq(angle, self.angle));
    //     } else {
    //         return (alneq(self.angle, (-2.0 * PI as f64 + angle)));
    //     }
    // }
    //get the points where the arc intersects a circle
    fn intersect_circle(&self, circle: &Circle) -> Option<Vec<Pos2F64>> {
        let intersect = circle_intersection(*circle, self.circle)?;
        let mut points = Vec::new();
        for point in intersect {
            if self.contains_properly(point) {
                points.push(point);
            }
        }
        return Some(points);
    }
    //get the points where the circle intersects the arc, including the start point but not the end
    // fn intersect_circle_start(&self, circle: &Circle) -> Option<Vec<Pos2F64>> {
    //     let intersect = circle_intersection(*circle, self.circle)?;
    //     let mut points = Vec::new();
    //     for point in intersect {
    //         if self.contains_properly_start(point) {
    //             points.push(point);
    //         }
    //     }
    //     return Some(points);
    // }
    //get the points where the arc intersects another arc
    // fn intersect_arc(&self, arc: Arc) -> Option<Vec<Pos2F64>> {
    //     let intersect = arc.intersect_circle(&self.circle)?;
    //     let mut points = Vec::new();
    //     for point in intersect {
    //         if self.contains_properly(point) {
    //             points.push(point);
    //         }
    //     }
    //     return Some(points);
    // }
    //same as above, but intersections at either start point are valid
    // fn intersect_arc_start(&self, arc: Arc) -> Option<Vec<Pos2F64>> {
    //     let intersect = arc.intersect_circle_start(&self.circle)?;
    //     let mut points = Vec::new();
    //     for point in intersect {
    //         if self.contains_properly_start(point) {
    //             points.push(point);
    //         }
    //     }
    //     return Some(points);
    // }
    //get a polygon representation of the arc for rendering
    fn get_polygon(&self, divisions: u16) -> Vec<Pos2F64> {
        let mut points: Vec<Pos2F64> = Vec::new();
        let inc_angle = self.angle / (divisions as f64);
        points.push(self.start);
        for i in 1..=divisions {
            points.push(rotate_about(
                self.circle.center,
                self.start,
                inc_angle * (i as f64),
            ));
        }
        return points;
    }
    //cut the arc into smaller arcs by a circle
    fn cut_by_circle(&self, circle: Circle) -> Option<Vec<Arc>> {
        let intersects = self.intersect_circle(&circle)?;
        return Some(self.cut_at(&intersects));
    }
    //takes a vec of points (must be on the arc, and not the endpoints) and returns the sorted version of them, as they appear on the arc
    //order in the sort_by call may just be wrong, reverse it potentially
    fn order_on_arc(&self, points: &Vec<Pos2F64>) -> Vec<Pos2F64> {
        let mut new_points = points.clone();
        new_points.sort_by(|a, b| cmp_f64(self.get_angle(*a).abs(), self.get_angle(*b).abs()));
        return new_points;
    }
    //cut at points in any order. passing points with !self.contains_properly(point) does not cut at those points
    //returns a Vec<Arc> that should have the resulting arcs in order from self.start to self.end()
    fn cut_at(&self, points: &Vec<Pos2F64>) -> Vec<Arc> {
        let mut total_points = vec![self.start];
        let mut valid_points = Vec::new();
        for point in points {
            if self.contains_properly(*point) {
                valid_points.push(*point);
            }
        }
        let sorted_points = self.order_on_arc(&valid_points);
        total_points.extend(sorted_points);
        total_points.push(self.end());
        let mut arcs = Vec::new();
        for i in 1..total_points.len() {
            arcs.push(get_arc(
                total_points[i - 1],
                total_points[i],
                self.circle,
                self.angle.is_sign_positive(),
            ));
        }
        return arcs;
    }
    fn get_tangent_vec(&self) -> Vec2F64 {
        let rad_vec = self.circle.center - self.start;
        if self.angle.is_sign_negative() {
            return rad_vec.rot90();
        }
        return rad_vec.rot90().rot90().rot90();
    }
    fn initially_above_y(&self, y: f64) -> bool {
        if alneq(self.start.y, y) {
            return true;
        }
        if alneq(y, self.start.y) {
            return false;
        }
        let angle = self.get_tangent_vec().angle().rem_euclid(2.0 * PI as f64);
        if aeq(angle, 2.0 * PI as f64) || aeq(angle, 0.0) || aeq(angle, PI as f64) {
            return self.circle.center.y >= y;
        } else if aleq(angle, PI as f64) {
            return true;
        }
        return false;
    }
    //triangulates the wedge (or antiwedge) between the point and the arc
    //every triangle is a Vec<Pos2> of length 3 with the first point being center
    fn triangulate(&self, center: Pos2F64, detail: f64) -> Vec<Vec<Pos2F64>> {
        let size = self.circle.radius * self.angle.abs();
        let divisions = (size / detail).max(2.0) as u16;
        let mut triangles = Vec::new();
        let polygon = self.get_polygon(divisions);
        for i in 0..(polygon.len() - 1) {
            triangles.push(vec![center, polygon[i], polygon[i + 1]]);
        }
        return triangles;
    }
    fn draw(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f64,
        offset: Turn,
        width: f32,
        scale_factor: f32,
        offset_pos: Vec2F64,
    ) {
        let size = self.circle.radius * self.angle.abs() * DETAIL_FACTOR;
        let divisions = (size / detail).max(2.0) as u16;
        let mut coords = Vec::new();
        for pos in self.get_polygon(divisions) {
            coords.push(to_egui_coords(
                &rotate_about(offset.circle.center, pos, offset.angle),
                rect,
                scale_factor,
                offset_pos,
            ));
        }
        ui.painter()
            .add(PathShape::line(coords, Stroke::new(width, OUTLINE_COLOR)));
    }
    //returns the number of points lying on the arc that are directly left of point (i.e., equal y value and lower x value). includes intersections at self.start but not at self.end() to avoid double counting
    fn arc_points_directly_left(&self, point: Pos2F64) -> Vec<Pos2F64> {
        let mut return_points = Vec::new();
        let points = circle_points_at_y(self.circle, point.y);
        for circ_point in &points {
            if (self.contains_properly(*circ_point) || aeq_pos(*circ_point, self.start)) //check if the
                && alneq(circ_point.x, point.x)
            {
                return_points.push(*circ_point);
            }
        }
        return return_points;
    }
}

impl FloatIntern {
    fn intern(&mut self, float: &mut f64) {
        for ifloat in &self.floats {
            if (*float - *ifloat).abs() <= self.leniency {
                *float = *ifloat;
                return;
            }
        }
        self.floats.push(*float);
    }
}

impl Puzzle {
    fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape {
                for value in [
                    &mut arc.start.x,
                    &mut arc.start.y,
                    &mut arc.circle.center.x,
                    &mut arc.circle.center.y,
                    &mut arc.circle.radius,
                    &mut arc.angle,
                ] {
                    self.intern.intern(value);
                }
            }
        }
    }
    //updates self.solved
    fn check(&mut self) {
        self.solved = aeq_pieces(&self.pieces, &self.solved_state)
    }
    //returns if the turn could be completed
    //Err(true) means that the turn was bandaged
    //Err(false) means that the cutting failed
    fn turn(&mut self, turn: Turn, cut: bool) -> Result<(), bool> {
        if cut {
            if self.global_cut_by_circle(turn.circle).is_err() {
                return Err(false);
            }
        }
        let mut new_pieces = Vec::new();
        for piece in &self.pieces {
            let mut new_piece = piece.clone();
            if new_piece.in_circle(&turn.circle).ok_or(true)? == Contains::Inside {
                new_piece.rotate_about(turn.circle.center, turn.angle);
            }
            new_pieces.push(new_piece);
        }
        self.pieces = new_pieces;
        if aeq_circ(self.animation_offset.circle, turn.circle) {
            self.animation_offset.angle += turn.inverse().angle;
        } else {
            self.animation_offset = turn.inverse();
        }
        self.intern_all();
        Ok(())
    }
    fn turn_id(&mut self, id: String, cut: bool) -> Result<(), bool> {
        let turn = self.turns[&id];
        self.turn(turn, cut)?;
        self.stack.push(id);
        self.check();
        Ok(())
    }
    fn undo(&mut self) -> Result<(), ()> {
        if self.stack.len() == 0 {
            return Err(());
        }
        let last_turn = self.turns[&self.stack.pop().unwrap()];
        self.turn(last_turn.inverse(), false).or(Err(())).unwrap();
        self.check();
        Ok(())
    }
    fn cut(&mut self, cut: &Cut) -> Result<(), ()> {
        for turn in cut {
            self.turn(*turn, true).or(Err(())).unwrap();
        }
        for turn in cut.clone().into_iter().rev() {
            self.turn(turn.inverse(), false).or(Err(())).unwrap();
        }
        Ok(())
    }
    fn color(&mut self, coloring: &Coloring) {
        for piece in &mut self.pieces {
            let mut color = true;
            for circle in &coloring.0 {
                let contains = piece.in_circle(&circle.0);
                if !contains.is_some_and(|x| x == circle.1) {
                    color = false;
                    break;
                }
            }
            if color {
                piece.color = coloring.1;
            }
        }
    }
    fn scramble(&mut self, cut: bool) -> Result<(), ()> {
        let mut rng = rand::rng();
        for _i in 0..self.depth {
            let key = self.turns.keys().choose(&mut rng).unwrap().clone();
            if self.turn(self.turns[&key], cut).is_err_and(|x| !x) {
                return Err(());
            }
            self.stack.push(key);
        }
        self.animation_offset = NONE_TURN;
        self.check();
        Ok(())
    }
    fn reset(&mut self) {
        loop {
            if self.undo().is_err() {
                self.animation_offset = NONE_TURN;
                return;
            }
        }
    }
    fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f64,
        outline_width: f32,
        scale_factor: f32,
        offset: Vec2F64,
    ) {
        for piece in &self.pieces {
            piece.render(
                ui,
                rect,
                self.animation_offset,
                detail,
                outline_width,
                scale_factor,
                offset,
            );
        }
    }
    fn process_click(
        &mut self,
        rect: &Rect,
        pos: Pos2,
        left: bool,
        scale_factor: f32,
        offset: Vec2F64,
        cut: bool,
    ) -> Result<(), bool> {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset);
        let mut min_dist: f64 = 10000.0;
        let mut min_rad: f64 = 10000.0;
        let mut correct_id: String = String::from("");
        for turn in &self.turns {
            if (alneq(good_pos.distance(turn.1.circle.center), min_dist)
                || (aeq(good_pos.distance(turn.1.circle.center), min_dist)
                    && alneq(turn.1.circle.radius, min_rad)))
                && turn.1.circle.contains(good_pos) == Contains::Inside
                && turn.1.angle.is_sign_negative()
            {
                min_dist = good_pos.distance(turn.1.circle.center);
                min_rad = turn.1.circle.radius;
                correct_id = turn.0.clone();
            }
        }
        if correct_id.is_empty() {
            return Ok(());
        }
        if !left {
            self.turn_id(correct_id, cut)?;
        } else {
            self.turn_id(correct_id + "'", cut)?;
        }
        Ok(())
    }
    fn get_hovered(&self, rect: &Rect, pos: Pos2, scale_factor: f32, offset: Vec2F64) -> Circle {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset);
        let mut min_dist: f64 = 10000.0;
        let mut min_rad: f64 = 10000.0;
        let mut correct_turn = NONE_TURN;
        for turn in self.turns.clone().values() {
            if (alneq(good_pos.distance(turn.circle.center), min_dist)
                || (aeq(good_pos.distance(turn.circle.center), min_dist)
                    && alneq(turn.circle.radius, min_rad)))
                && turn.circle.contains(good_pos) == Contains::Inside
            {
                min_dist = good_pos.distance(turn.circle.center);
                min_rad = turn.circle.radius;
                correct_turn = *turn;
            }
        }
        //dbg!(correct_turn.circle.center.to_pos2());
        return correct_turn.circle;
    }
    // fn cut_by_circle(&mut self, circle: Circle, turn: Turn) {
    //     let mut new_pieces = Vec::new();
    //     for piece in &self.pieces {
    //         if piece
    //             .in_circle(&turn.circle)
    //             .is_none_or(|x| x == Contains::Inside)
    //         {
    //             new_pieces.extend(piece.cut_by_circle(circle));
    //         } else {
    //             new_pieces.push(piece.clone());
    //         }
    //     }
    //     self.pieces = new_pieces;
    //     self.intern_all();
    // }
    fn global_cut_by_circle(&mut self, circle: Circle) -> Result<(), ()> {
        let mut new_pieces = Vec::new();
        for piece in &self.pieces {
            new_pieces.extend(piece.cut_by_circle(circle).ok_or(())?);
        }
        self.pieces = new_pieces;
        self.intern_all();
        Ok(())
    }
    // fn cut_with_turn_symmetry(&mut self, circle: Circle, turn: Turn) {
    //     let mut index = 0;
    //     while !aeq(turn.angle * (index as f64), 2.0 * PI as f64) {
    //         self.cut_by_circle(
    //             circle.rotate_about(turn.circle.center, turn.angle * (index as f64)),
    //             turn,
    //         );
    //         index += 1;
    //     }
    // }
    // fn draw_outline(
    //     &self,
    //     ui: &mut Ui,
    //     rect: &Rect,
    //     detail: f64,
    //     outline_width: f32,
    //     scale_factor: f32,
    //     offset: Vec2F64,
    // ) {
    //     for piece in &self.pieces {
    //         if piece
    //             .in_circle(&self.animation_offset.circle)
    //             .is_some_and(|x| x == Contains::Inside)
    //         {
    //             piece.draw_outline(
    //                 ui,
    //                 rect,
    //                 detail,
    //                 self.animation_offset,
    //                 outline_width,
    //                 scale_factor,
    //                 offset,
    //             );
    //         } else {
    //             piece.draw_outline(
    //                 ui,
    //                 rect,
    //                 detail,
    //                 NONE_TURN,
    //                 outline_width,
    //                 scale_factor,
    //                 offset,
    //             );
    //         }
    //     }
    // }
}

impl Piece {
    fn rotate_about(&mut self, center: Pos2F64, angle: f64) {
        let mut new_arcs: Vec<Arc> = Vec::new();
        for arc in self.shape.clone() {
            new_arcs.push(arc.rotate_about(center, angle));
        }
        self.shape = new_arcs;
    }
    fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        offset: Turn,
        detail: f64,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2F64,
    ) {
        let true_offset = if self
            .in_circle(&offset.circle)
            .is_some_and(|x| x == Contains::Inside)
        {
            offset
        } else {
            NONE_TURN
        };
        let triangulation = self.triangulate(self.barycenter(), detail);
        let mut triangle_vertices: Vec<epaint::Vertex> = Vec::new();
        for triangle in triangulation {
            if !almost_degenerate(&triangle, 0.0) {
                for point in triangle {
                    let vertex = epaint::Vertex {
                        pos: to_egui_coords(
                            &rotate_about(true_offset.circle.center, point, true_offset.angle),
                            rect,
                            scale_factor,
                            offset_pos,
                        ),
                        uv: pos2(0.0, 0.0),
                        color: self.color,
                    };
                    triangle_vertices.push(vertex);
                }
            }
        }
        let mut mesh = epaint::Mesh::default();
        mesh.indices = (0..(triangle_vertices.len() as u32)).collect();
        mesh.vertices = triangle_vertices;
        ui.painter().add(egui::Shape::Mesh(mesh.into()));
        self.draw_outline(
            ui,
            rect,
            detail,
            true_offset,
            outline_size,
            scale_factor,
            offset_pos,
        );
    }
    //returns a list of triangles for rendering
    fn triangulate(&self, center: Pos2F64, detail: f64) -> Vec<Vec<Pos2F64>> {
        let mut triangles = Vec::new();
        for arc in &self.shape {
            triangles.extend(arc.triangulate(center, detail));
        }
        return triangles;
    }
    fn barycenter(&self) -> Pos2F64 {
        let mut points = Vec::new();
        for arc in &self.shape {
            points.push(arc.midpoint());
        }
        return avg_points(&points);
    }
    //None: the piece is inside and outside -- blocking
    fn in_circle(&self, circle: &Circle) -> Option<Contains> {
        let mut inside = None;
        for arc in &self.shape {
            let contained = circle.contains_arc(arc);
            if contained.is_some() {
                if let Some(real_inside) = inside {
                    if contained.unwrap() != Contains::Border && real_inside != contained.unwrap() {
                        return None;
                    }
                } else if contained.unwrap() != Contains::Border {
                    inside = contained;
                }
            } else {
                return None;
            }
        }
        if inside.is_none_or(|x| x == Contains::Border) {
            return Some(Contains::Inside);
        }
        return inside;
    }
    //return if the shape contains the point properly -- not on the border
    //should return false if the point is properly outside the piece and true if it is properly inside the piece -- behavior on border points is undefined, may panic or return either option.
    //essentially, check how many 'valid' points that are on the border of self and directly left (within leniency) of point, and then take that mod 2 to get the answer
    fn contains_point(&self, point: Pos2F64) -> bool {
        let y = point.y;
        let mut intersects = 0;
        for i in 0..self.shape.len() {
            let arc = self.shape[i];
            let prev_arc = self.shape[match i {
                0 => self.shape.len() - 1,
                _ => i - 1,
            }];
            let points = arc.arc_points_directly_left(point); //get the points on the current arc directly left of the point
            for int_point in points {
                if aeq_pos(arc.start, int_point) {
                    if ((arc.initially_above_y(y)) != (prev_arc.invert().initially_above_y(y))) //tangent case -- basically we only add in this case if the arcs actually cross the y line
                        || aeq(arc.angle, 2.0 * PI as f64)
                    {
                        intersects += 1;
                    }
                } else {
                    if !aeq((arc.circle.center.y - y).abs(), arc.circle.radius) {
                        //throw out the case where the arc is tangent at this point to the y line
                        intersects += 1;
                    }
                }
            }
        }
        //dbg!(intersects);
        return (intersects % 2) != 0; //the point is inside the piece if the number of valid intersects is odd, and outside otherwise
    }
    // fn contains_point_border(&self, point: Pos2F64) -> bool {
    //     for arc in &self.shape {
    //         if arc.contains_properly(point) || aeq_pos(point, arc.start) {
    //             return true;
    //         }
    //     }
    //     return false;
    // }
    //doesnt count arcs
    //fn intersect_arc_start(&self, arc: Arc) -> Option<Vec<Pos2F64>> {}
    //fn intersect_piece(&self, other: Piece) -> Vec<Pos2F64> {}
    // cut a piece by a circle and return the resulting pieces as a Vec
    //the color is maintained
    fn cut_by_circle(&self, circle: Circle) -> Option<Vec<Piece>> {
        let mut shapes: Vec<Vec<Arc>> = Vec::new(); // the shapes of the final pieces
        let mut piece_arcs = Vec::new(); // the arcs obtained by cutting up the piece by the circle
        for arc in &self.shape {
            let bits = arc.cut_by_circle(circle); //bits created from cutting up the arc -- None iff the arc lies on circle
            if bits.is_none() {
                return Some(vec![self.clone()]); //don't cut the shape in this case
            }
            let arc_bits = bits.unwrap(); //add the bits to the piece_arcs
            piece_arcs.extend(arc_bits);
        } //populate piece_arcs
        // println!("startpos: {}", piece_arcs[1].start);
        //println!("{}", piece_arcs.len());
        let mut circle_cut_points = Vec::new(); //the points at which to cut circle--effectively the intersection points between circle and arc in piece_arcs
        let mut circle_pieces = Vec::new(); //the created pieces of the circle
        for arc in &piece_arcs {
            if circle.contains(arc.start) == Contains::Border {
                //every intersection point is already tracked, because the piece_arcs were cut
                circle_cut_points.push(arc.start);
            }
        }
        if circle_cut_points.is_empty() {
            return Some(vec![self.clone()]); //if the circle lies fully outside the piece
        }
        // for point in &circle_cut_points {
        //     println!("{}", point);
        // }
        let start_point = circle_cut_points.remove(0);
        let circle_as_arc = get_arc(start_point, start_point, circle, true); //make the circle into a 2PI arc
        circle_pieces.extend(circle_as_arc.cut_at(&circle_cut_points)); //cut the circle up
        circle_pieces.extend(circle_as_arc.invert().cut_at(&circle_cut_points)); //also add the inverse arcs
        let mut inside_circle_pieces = Vec::new(); //the pieces of the circle that lie inside the original piece -- the ones we care about
        for arc in circle_pieces {
            if self.contains_point(arc.midpoint()) {
                //we check if the circle piece is in the piece by checking the midpoint -- the case where midpoint is on the border is already handled, since in this case arc.circle == circle for some arc in piece_arcs
                inside_circle_pieces.push(arc);
            }
        }
        let mut all_arcs = piece_arcs.clone();
        all_arcs.extend(inside_circle_pieces); //all arcs to construct pieces from
        while !all_arcs.is_empty() {
            //println!("{}, {}", piece_arcs.len(), circle_arcs.len());
            //iterate through the list of all the arcs
            let mut curr_shape = vec![all_arcs.remove(0)]; //a shape created from these arc
            // println!(
            //     "{} -> {} -> {}, {}",
            //     curr_shape[0].start,
            //     curr_shape[0].midpoint(),
            //     curr_shape[0].end(),
            //     curr_shape[0].angle
            // );
            // for arc in &piece_arcs {
            //     println!("{} -> {}", arc.start, arc.end());
            // }
            // for arc in &circle_arcs {
            //     println!("{} -> {}", arc.start, arc.end());
            // }
            //remove the first arc
            loop {
                curr_shape.push(get_best_arc_and_pop(
                    &mut all_arcs,
                    *curr_shape.last().unwrap(),
                )?);
                // println!(
                //     "{} -> {} ->  {}, {}",
                //     curr_shape.last().unwrap().start,
                //     curr_shape.last().unwrap().midpoint(),
                //     curr_shape.last().unwrap().end(),
                //     curr_shape.last().unwrap().angle
                // );
                if aeq_pos(curr_shape.last().unwrap().end(), curr_shape[0].start) {
                    //if the closed shape is created
                    shapes.push(collapse_shape(&curr_shape).unwrap());
                    break;
                }
            }
        }
        let mut pieces = Vec::new();
        for shape in shapes {
            pieces.push(Piece {
                shape: shape.clone(),
                color: self.color,
            });
        }
        return Some(pieces);
    }
    //determine which piece occurs earlier on the piece
    //self.shape[0].start is the minimum
    //if the two points are equivalent, returns Less
    // fn order_along_piece(&self, point1: Pos2F64, point2: Pos2F64) -> Ordering {
    //     for arc in &self.shape {
    //         if aeq_pos(point1, arc.start) {
    //             return Ordering::Less;
    //         } else if aeq_pos(point2, arc.start) {
    //             return Ordering::Greater;
    //         } else if arc.contains_properly(point1) {
    //             if arc.contains_properly(point2) {
    //                 if arc.get_angle(point1).abs() <= arc.get_angle(point2).abs() {
    //                     return Ordering::Less;
    //                 }
    //                 return Ordering::Greater;
    //             }
    //             return Ordering::Less;
    //         } else if arc.contains_properly(point2) {
    //             return Ordering::Greater;
    //         }
    //     }
    //     return Ordering::Equal;
    // }
    //sort points based on how they appear along the piece, ccw
    // fn sort_along_piece(&self, points: &Vec<Pos2F64>) -> Vec<Pos2F64> {
    //     let mut sorted = points.clone();
    //     sorted.sort_by(|a, b| self.order_along_piece(*a, *b));
    //     return sorted;
    // }
    //cut the piece into segments (cut at a bunch of points)
    // fn cut_at(&self, points: &Vec<Pos2F64>) -> Vec<Vec<Arc>> {
    //     let mut remaining_shape = Vec::new();
    //     for arc in &self.shape {
    //         remaining_shape.extend(arc.cut_at(points))
    //     }
    //     let sorted_points = self.sort_along_piece(points);
    //     let index = 0;
    //     let mut segments = Vec::new();
    //     let mut curr_segment = Vec::new();
    //     while !remaining_shape.is_empty() {
    //         let arc = remaining_shape[0];
    //         let point = sorted_points[index];
    //         if aeq_pos(point, arc.start) {
    //             segments.push(curr_segment.clone());
    //             curr_segment = Vec::new();
    //         }
    //     }
    //     return segments;
    // }
    // fn cut_by_piece(&self, other: Piece) -> Vec<Piece> {
    //     return Vec::new();
    // }
    fn draw_outline(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f64,
        offset: Turn,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2F64,
    ) {
        for arc in &self.shape {
            arc.draw(
                ui,
                rect,
                detail,
                offset,
                outline_size,
                scale_factor,
                offset_pos,
            );
        }
    }
    // fn get_polygon(&self, detail: u16) -> Vec<Pos2F64> {
    //     let mut points = Vec::new();
    //     for arc in &self.shape {
    //         points.extend(arc.get_polygon(detail));
    //         points.pop();
    //     }
    //     return points;
    // }
}

impl DataStorer {
    fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        self.data = Vec::new();
        let paths = fs::read_dir(def_path).or(Err(())).unwrap().into_iter();
        for path in paths {
            let data = read_file_to_string(
                &(String::from(def_path)
                    + (&path
                        .or(Err(()))
                        .unwrap()
                        .file_name()
                        .into_string()
                        .or(Err(()))
                        .unwrap())),
            )
            .or(Err(()))
            .unwrap();
            self.data.push((get_preview_string(&data), data));
        }
        self.data.sort_by_key(|a| a.0.clone());
        Ok(())
    }
    fn render_panel(&self, ctx: &egui::Context) -> Result<Option<(Puzzle, String)>, ()> {
        let panel = egui::SidePanel::new(egui::panel::Side::Right, "data_panel").resizable(false);
        let mut puzzle = None;
        panel.show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for puz in &self.data {
                    if ui.add(egui::Button::new(&puz.0)).clicked() {
                        puzzle = match parse_kdl(&puz.1) {
                            Some(inside) => Some((inside, puz.1.clone())),
                            None => None,
                        }
                    }
                }
            })
        });
        Ok(puzzle)
    }
}

//pass an orig_arc with the same start as a1, a2. finds the first arc ccw

fn order_arcs(a1: Arc, a2: Arc, orig_arc: Arc) -> Ordering {
    let arcs = [a1, a2];
    fn in_tangency_case(a1: Arc, a2: Arc) -> Ordering {
        if aeq_circ(a1.circle, a2.circle) {
            return Ordering::Equal;
        }
        if a1.angle.is_sign_positive() {
            if a2.angle.is_sign_negative() {
                return Ordering::Less;
            }
            if a1.circle.radius <= a2.circle.radius {
                return Ordering::Less;
            }
            return Ordering::Greater;
        } else {
            if a2.angle.is_sign_positive() {
                return Ordering::Greater;
            }
            if a1.circle.radius <= a2.circle.radius {
                return Ordering::Greater;
            }
            return Ordering::Less;
        }
    }
    let orig_tang = orig_arc.get_tangent_vec();
    let tangents = [a1.get_tangent_vec(), a2.get_tangent_vec()];
    let mut angles = [
        (orig_tang.angle() - tangents[0].angle()).rem_euclid(2.0 * PI as f64),
        (orig_tang.angle() - tangents[1].angle()).rem_euclid(2.0 * PI as f64),
    ];
    for i in 0..=1 {
        if aeq(angles[i], 0.0) || aeq(angles[i], 2.0 * PI as f64) {
            if in_tangency_case(orig_arc, arcs[i]) == Ordering::Less {
                angles[i] = 0.0;
            } else if in_tangency_case(orig_arc, arcs[i]) == Ordering::Greater {
                angles[i] = 2.0 * PI as f64;
            }
        }
    }
    //dbg!(angles);
    if alneq(angles[0], angles[1]) {
        return Ordering::Less;
    } else if alneq(angles[1], angles[0]) {
        return Ordering::Greater;
    } else if aeq(angles[0], angles[1]) {
        return in_tangency_case(a1, a2);
    }
    return Ordering::Equal;
}

//sorts arcs, give the same conditions as in order_arcs
fn find_min_arc_index(arcs: &Vec<Arc>, orig_arc: Arc) -> usize {
    let mut min = 0;
    for i in 0..arcs.len() {
        if order_arcs(arcs[i], arcs[min], orig_arc) == Ordering::Less {
            min = i;
        }
    }
    return min;
}

fn get_best_arc_and_pop(all_arcs: &mut Vec<Arc>, curr_arc: Arc) -> Option<Arc> {
    let mut good_arcs = Vec::new();
    let mut indices = Vec::new();
    for i in 0..all_arcs.len() {
        if aeq_pos(curr_arc.end(), all_arcs[i].start)
            && !(aeq_circ(curr_arc.circle, all_arcs[i].circle)
                && (curr_arc.angle.is_sign_positive() != all_arcs[i].angle.is_sign_positive()))
        {
            good_arcs.push(all_arcs[i].clone());
            indices.push(i);
        }
    }
    let index = find_min_arc_index(&good_arcs, curr_arc.invert());
    if good_arcs.len() == 0 {
        return None;
    }
    return Some(all_arcs.remove(indices[index]));
}
// fn get_best_segment_and_pop(segments: &mut Vec<Vec<Arc>>, curr_segment: &Vec<Arc>) -> Vec<Arc> {
//     let mut good_arcs = Vec::new();
//     let mut indices = Vec::new();
//     for i in 0..segments.len() {
//         if aeq_pos(curr_segment.last().unwrap().end(), segments[i][0].start)
//             && !(aeq_circ(curr_segment.last().unwrap().circle, segments[i][0].circle)
//                 && curr_segment.last().unwrap().angle.is_sign_positive()
//                     != segments[i][0].angle.is_sign_positive())
//         {
//             good_arcs.push(segments[i][0].clone());
//             indices.push(i);
//         }
//     }
//     let index = find_min_arc_index(&good_arcs, *curr_segment.last().unwrap());
//     return segments.remove(indices[index]);
// }

//take in a triangle and return if its 'almost degenerate' within some leniency (i.e. its points are 'almost colinear')
fn almost_degenerate(triangle: &Vec<Pos2F64>, leniency: f64) -> bool {
    let angle_1 = (triangle[1] - triangle[0]).angle() - (triangle[1] - triangle[2]).angle();
    let close = angle_1.abs().min((PI as f64 - angle_1).abs());
    if close < leniency {
        return true;
    }
    return false;
}

//intersections between a line segment (including endpoints) and a circle

fn circle_points_at_y(circle: Circle, y: f64) -> Vec<Pos2F64> {
    if alneq(circle.radius, (circle.center.y - y).abs()) {
        return Vec::new();
    } else if aeq(circle.radius, (circle.center.y - y).abs()) {
        return vec![pos2_f64(circle.center.x, y)];
    }
    let proper_x =
        ((circle.radius * circle.radius) - ((circle.center.y - y) * (circle.center.y - y))).sqrt();
    return vec![
        pos2_f64(circle.center.x - proper_x, y),
        pos2_f64(circle.center.x + proper_x, y),
    ];
}

fn avg_points(points: &Vec<Pos2F64>) -> Pos2F64 {
    let n: f64 = points.len() as f64;
    let mut pos = pos2_f64(0.0, 0.0);
    for point in points {
        pos.x += point.x / n;
        pos.y += point.y / n;
    }
    return pos;
}

// fn pop_arc_from_vec_from_start(start: Pos2F64, arcs: &mut Vec<Arc>) -> Option<Arc> {
//     for i in 0..arcs.len() {
//         if aeq_pos(arcs[i].start, start) {
//             let ret_arc = arcs[i].clone();
//             arcs.remove(i);
//             return Some(ret_arc);
//         }
//     }
//     return None;
// }

// fn get_arc_starting_at(point: Pos2F64, arcs: Vec<Arc>) -> Option<Arc> {
//     for arc in arcs {
//         if aeq_pos(arc.start, point) {
//             return Some(arc);
//         }
//     }
//     return None;
// }

fn is_tangent(c1: Circle, c2: Circle) -> bool {
    return (aeq(c1.center.distance(c2.center) + c1.radius, c2.radius)
        || aeq(c1.center.distance(c2.center) + c2.radius, c1.radius))
        || aeq(c1.radius + c2.radius, c1.center.distance(c2.center));
}

//gives the angle between two points, counterclockwise point1 -> point2, on the same circle. angle is in (0, 2PI]
//BAD - REWORK
fn angle_on_circle(point1: Pos2F64, point2: Pos2F64, circle: Circle) -> f64 {
    // let mut angle: f32 = ((circle.center.distance_sq(point1) - (0.5 * point1.distance_sq(point2)))
    //     / (circle.center.distance_sq(point1)))
    // .acos();
    // if !aeq_pos(rotate_about(circle.center, point1, angle), point2) {
    //     return (2.0 * PI) - angle;
    // }
    // if aeq(angle, 0.0) {
    //     angle = 2.0 * PI;
    // }

    let angle = ((point2 - circle.center).angle() - (point1 - circle.center).angle())
        .rem_euclid(PI as f64 * 2.0);
    if aeq(angle, 0.0) {
        return 2.0 * PI as f64;
    }
    return angle;
}

// fn arc_from_circle(circle: Circle, start: Pos2F64, ccw: bool) -> Arc {
//     return get_arc(start, start, circle, ccw);
// }

//when start == end is passed, a full circle is returned
//ccw is true -> creates counterclockwise start -> end, ccw = false does clockwise

fn get_arc(start: Pos2F64, end: Pos2F64, circle: Circle, ccw: bool) -> Arc {
    let angle = angle_on_circle(start, end, circle);
    if aeq_pos(start, end) {
        if ccw {
            return Arc {
                start: start,
                angle: 2.0 * (PI as f64),
                circle: circle,
            };
        } else {
            return Arc {
                start: start,
                angle: -2.0 * (PI as f64),
                circle: circle,
            };
        }
    }

    if ccw {
        return Arc {
            start: start,
            angle: angle,
            circle: circle,
        };
    } else {
        return Arc {
            start: start,
            angle: (-2.0 * PI as f64) + angle,
            circle: circle,
        };
    }
}

//translates from nice coords to egui coords
fn to_egui_coords(pos: &Pos2F64, rect: &Rect, scale_factor: f32, offset: Vec2F64) -> Pos2 {
    return pos2(
        ((pos.x + offset.x) as f32) * (scale_factor * rect.width() / 1920.0)
            + (rect.width() / 2.0)
            + rect.min.x,
        -1.0 * ((pos.y + offset.y) as f32) * (scale_factor * rect.width() / 1920.0)
            + (rect.height() / 2.0)
            + rect.min.y,
    );
}

// fn to_egui_coords_vec(
//     points: &Vec<Pos2F64>,
//     rect: &Rect,
//     scale_factor: f32,
//     offset: Vec2F64,
// ) -> Vec<Pos2> {
//     let mut good_points = Vec::new();
//     for point in points {
//         good_points.push(to_egui_coords(
//             &(*point + offset),
//             rect,
//             scale_factor,
//             offset,
//         ));
//     }
//     return good_points;
// }

//translates from egui coords to nice coords
fn from_egui_coords(pos: &Pos2, rect: &Rect, scale_factor: f32, offset: Vec2F64) -> Pos2F64 {
    return pos2_f64(
        ((pos.x - (rect.width() / 2.0)) * (1920.0 / (scale_factor * rect.width()))) as f64
            - offset.x,
        ((pos.y - (rect.height() / 2.0)) * (-1920.0 / (scale_factor * rect.width()))) as f64
            - offset.y,
    );
}

//rotate a point about a point a certain angle
fn rotate_about(center: Pos2F64, point: Pos2F64, angle: f64) -> Pos2F64 {
    if aeq_pos(center, point) {
        return point;
    }
    let dist = center.distance(point);
    let curr_angle = (point - center).angle();
    let end_angle = angle + curr_angle;
    return pos2_f64(
        center.x + (dist * end_angle.cos()),
        center.y + (dist * end_angle.sin()),
    );
}

// fn rotate_about_vec(center: Pos2F64, points: &Vec<Pos2F64>, angle: f64) -> Vec<Pos2F64> {
//     let mut rot_points = Vec::new();
//     for point in points {
//         rot_points.push(rotate_about(center, point.clone(), angle));
//     }
//     return rot_points;
// }

//needs to draw ccw with respect to the circle
// fn circle_region(detail: u16, center: Pos2, point1: Pos2, point2: Pos2) -> Vec<Pos2> {
//     let angle: f32 = ((center.distance_sq(point1) - (0.5 * point1.distance_sq(point2)))
//         / (center.distance_sq(point1)))
//     .acos();
//     let mut point: Pos2 = point1.clone();
//     let mut points: Vec<Pos2> = Vec::new();
//     for i in 0..detail {
//         points.push(point.clone());
//         point = rotate_about(center, point, angle / f32::from(detail));
//     }
//     return points;
// }

//returns the 0-2 circle intersection points. the one that's clockwise above the horizon from circle1 is returned first
fn circle_intersection(circle1: Circle, circle2: Circle) -> Option<Vec<Pos2F64>> {
    if aeq_circ(circle1, circle2) {
        return None;
    }
    if alneq(
        circle1.radius + circle2.radius,
        circle1.center.distance(circle2.center),
    ) || alneq(
        circle1.center.distance(circle2.center) + circle2.radius,
        circle1.radius,
    ) || alneq(
        circle1.center.distance(circle2.center) + circle1.radius,
        circle2.radius,
    ) {
        return Some(Vec::new());
    }
    if aeq(
        circle1.center.distance(circle2.center),
        circle1.radius + circle2.radius,
    ) || aeq(
        circle1.center.distance(circle2.center) + circle2.radius,
        circle1.radius,
    ) {
        return Some(vec![
            (circle1.center + (circle1.radius * (circle2.center - circle1.center).normalized()?)),
        ]);
    }
    if aeq(
        circle1.center.distance(circle2.center) + circle1.radius,
        circle2.radius,
    ) {
        return Some(vec![
            (circle2.center + (circle2.radius * (circle1.center - circle2.center).normalized()?)),
        ]);
    }
    let dist_sq = circle1.center.distance_sq(circle2.center);
    let angle = ((dist_sq + (circle1.radius * circle1.radius) - (circle2.radius * circle2.radius))
        / (2.0 * circle1.radius * circle1.center.distance(circle2.center)))
    .acos();
    let difference = circle2.center - circle1.center;
    let unit_difference = difference.normalized()?;
    let arc_point = circle1.center + (circle1.radius * unit_difference);
    let point1 = rotate_about(circle1.center, arc_point, -1.0 * angle);
    let point2 = rotate_about(circle1.center, arc_point, angle);
    return Some(vec![point1, point2]);
}

fn collapse_shape(shape: &Vec<Arc>) -> Option<Vec<Arc>> {
    let mut new_shape: Vec<Arc> = vec![shape[0]];
    for i in 1..shape.len() {
        let arc = shape[i];
        if aeq_circ(new_shape.last()?.circle, arc.circle) {
            new_shape.last_mut()?.angle += arc.angle;
        } else {
            new_shape.push(arc);
        }
    }
    if aeq_circ(new_shape[0].circle, new_shape.last()?.circle) {
        new_shape.last_mut()?.angle += new_shape[0].angle;
        new_shape.remove(0);
    }
    return Some(new_shape);
}

fn make_basic_puzzle(disks: Vec<Circle>) -> Result<Vec<Piece>, ()> {
    let mut pieces = Vec::new();
    let mut old_disks = Vec::new();
    for disk in &disks {
        let point = pos2_f64(disk.center.x + disk.radius, disk.center.y);
        let disk_piece = Piece {
            shape: vec![get_arc(point, point, *disk, true)],
            color: NONE_COLOR,
        };
        let mut disk_pieces = vec![disk_piece];
        for old_disk in &old_disks {
            let mut new_pieces: Vec<Piece> = Vec::new();
            for piece in &disk_pieces {
                new_pieces.extend(piece.cut_by_circle(*old_disk).ok_or(())?);
            }
            disk_pieces = new_pieces.clone();
        }
        let mut valid_pieces = Vec::new();
        for piece in &disk_pieces {
            let mut add = true;
            for disk in &old_disks {
                if piece.in_circle(disk).unwrap() == Contains::Inside {
                    add = false;
                    break;
                }
            }
            if add {
                valid_pieces.push(piece.clone());
            }
        }
        old_disks.push(*disk);
        pieces.extend(valid_pieces);
    }
    return Ok(pieces);
}

fn puzzle_from_string(string: String) -> Option<Puzzle> {
    let components = string
        .split("--LOG FILE \n")
        .into_iter()
        .collect::<Vec<&str>>();
    let mut puzzle = parse_kdl(components.get(0)?)?;
    if components.len() > 1 {
        let turns = components.get(1)?.split(",");
        for turn in turns {
            puzzle.turn_id(String::from(turn), false).ok()?;
        }
    }
    return Some(puzzle);
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file_to_string(path: &String) -> std::io::Result<String> {
    let curr_path = match DEV {
        false => String::from(
            std::env::current_exe()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .split("circleguy.exe")
                .into_iter()
                .collect::<Vec<&str>>()[0],
        ),
        true => String::new(),
    };
    std::fs::read_to_string(curr_path + &path)
}

#[cfg(target_arch = "wasm32")]
fn read_file_to_string(path: &str) -> Result<String, &'static str> {
    static PUZZLE_DEFINITIONS: include_dir::Dir<'_> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/Puzzles");
    let path = path.strip_prefix("Puzzles/").unwrap_or(path);
    Ok(PUZZLE_DEFINITIONS
        .get_file(path)
        .ok_or("no such file")?
        .contents_utf8()
        .ok_or("invalid UTF-8")?
        .to_string())
}

fn load_puzzle_and_def_from_file(path: &String) -> Option<(Puzzle, String)> {
    let contents = read_file_to_string(path).ok()?;
    return Some((
        puzzle_from_string(contents.clone())?,
        String::from(
            contents
                .split("--LOG FILE")
                .into_iter()
                .collect::<Vec<&str>>()[0],
        ),
    ));
}

// fn load(to_load: PuzzleDef, to_set: &mut PuzzleDef) -> Puzzle {
//     *to_set = to_load;
//     return puzzle_from_two_circles(to_load);
// }

fn strip_number_end(str: &str) -> Option<(String, String)> {
    let chars = str.chars();
    let end = chars
        .rev()
        .take_while(|x| x.is_numeric())
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect::<String>();
    return match end.is_empty() {
        true => None,
        false => Some((String::from(str.strip_suffix(&end)?), end)),
    };
}

fn get_preview_string(data: &String) -> String {
    let data = match prev_parse_kdl(data.as_str()) {
        None => return String::from("Could not parse preview!"),
        Some(real) => real,
    };
    return data.name + ": " + &data.turns.join(",");
}

fn prev_parse_kdl(string: &str) -> Option<PuzzlePrevData> {
    let mut data = PuzzlePrevData {
        name: String::new(),
        turns: Vec::new(),
    };
    let mut numbers = Vec::new();
    let doc: KdlDocument = string.parse().ok()?;
    for node in doc.nodes() {
        match node.name().value() {
            "name" => {
                data.name = String::from(node.entries().get(0)?.value().as_string()?);
            }
            "twists" => {
                for twist in node.children()?.nodes() {
                    if twist.entries().len() == 2 {
                        numbers.push(twist.entries().get(1)?.value().as_integer()?)
                    }
                }
            }
            _ => {}
        }
    }
    numbers.sort();
    numbers.reverse();
    for turn in &numbers {
        data.turns.push(turn.to_string());
    }
    return Some(data);
}
fn parse_kdl(string: &str) -> Option<Puzzle> {
    let mut puzzle = Puzzle {
        name: String::new(),
        authors: Vec::new(),
        pieces: Vec::new(),
        turns: HashMap::new(),
        stack: Vec::new(),
        animation_offset: NONE_TURN,
        intern: FloatIntern {
            floats: Vec::new(),
            leniency: LENIENCY,
        },
        depth: 500,
        solved_state: Vec::new(),
        solved: true,
    };
    let mut def_stack = Vec::new();
    let doc: KdlDocument = match string.parse() {
        Ok(real) => real,
        Err(_err) => return None,
    };
    let mut circles: HashMap<&str, Circle> = HashMap::new();
    let mut twists: HashMap<&str, Turn> = HashMap::new();
    let mut real_twists: HashMap<&str, Turn> = HashMap::new();
    let mut colors: HashMap<String, Color32> = get_default_color_hash();
    let mut compounds: HashMap<&str, Vec<Turn>> = HashMap::new();
    let mut ctx = meval::Context::new();
    for node in doc.nodes() {
        match node.name().value() {
            "name" => puzzle.name = String::from(node.entries().get(0)?.value().as_string()?),
            "author" => puzzle
                .authors
                .push(String::from(node.entries().get(0)?.value().as_string()?)),
            "vars" => {
                for var in node.children()?.nodes() {
                    let val = var.entries().get(0)?.value();
                    let float_val = match val.is_string() {
                        true => meval::eval_str_with_context(val.as_string()?, &ctx).ok()?,
                        false => match val.is_integer() {
                            true => val.as_integer()? as f64,
                            false => val.as_float()?,
                        },
                    };
                    ctx.var(var.name().value(), float_val);
                }
            }
            "circles" => {
                for circle in node.children()?.nodes() {
                    circles.insert(
                        circle.name().value(),
                        Circle {
                            center: pos2_f64(
                                match circle.get("x")?.is_string() {
                                    true => meval::eval_str_with_context(
                                        circle.get("x")?.as_string()?,
                                        &ctx,
                                    )
                                    .ok()?,
                                    false => match circle.get("x")?.is_float() {
                                        true => circle.get("x")?.as_float()?,
                                        false => circle.get("x")?.as_integer()? as f64,
                                    },
                                },
                                match circle.get("y")?.is_string() {
                                    true => meval::eval_str_with_context(
                                        circle.get("y")?.as_string()?,
                                        &ctx,
                                    )
                                    .ok()?,
                                    false => match circle.get("y")?.is_float() {
                                        true => circle.get("y")?.as_float()?,
                                        false => circle.get("y")?.as_integer()? as f64,
                                    },
                                },
                            ),
                            radius: match circle.get("r")?.is_string() {
                                true => meval::eval_str_with_context(
                                    circle.get("r")?.as_string()?,
                                    &ctx,
                                )
                                .ok()?,
                                false => match circle.get("r")?.is_float() {
                                    true => circle.get("r")?.as_float()?,
                                    false => circle.get("r")?.as_integer()? as f64,
                                },
                            },
                        },
                    );
                }
            }
            "base" => {
                let mut disks = Vec::new();
                for disk in node.entries().into_iter() {
                    disks.push(*circles.get(disk.value().as_string()?)?);
                }
                puzzle.pieces = make_basic_puzzle(disks).ok()?;
            }
            "twists" => {
                for turn in node.children()?.nodes() {
                    twists.insert(
                        turn.name().value(),
                        Turn {
                            circle: *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                            angle: -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        },
                    );
                    if turn.entries().len() == 2 {
                        real_twists.insert(
                            turn.name().value(),
                            Turn {
                                circle: *circles
                                    .get(turn.entries().get(0)?.value().as_string()?)?,
                                angle: -2.0 * PI as f64
                                    / (turn.entries().get(1)?.value().as_integer()? as f64),
                            },
                        );
                    }
                    compounds.insert(
                        turn.name().value(),
                        vec![Turn {
                            circle: *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                            angle: -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        }],
                    );
                }
            }
            "compounds" => {
                let mut compound_adds: Vec<Vec<Turn>> = vec![Vec::new()];
                let mut extend: Vec<Turn>;
                for compound in node.children()?.nodes() {
                    for val in compound.entries() {
                        match val.value().as_string()?.strip_suffix("'") {
                            None => match strip_number_end(val.value().as_string()?) {
                                None => extend = compounds.get(val.value().as_string()?)?.clone(),
                                Some(real) => {
                                    extend = multiply_turns(
                                        real.1.parse::<isize>().unwrap(),
                                        compounds.get(real.0.as_str())?,
                                    );
                                }
                            },
                            Some(real) => match strip_number_end(real) {
                                None => {
                                    extend = invert_compound_turn(compounds.get(real)?);
                                }
                                Some(inside) => {
                                    extend = invert_compound_turn(&multiply_turns(
                                        inside.1.parse::<isize>().unwrap(),
                                        compounds.get(inside.0.as_str())?,
                                    ));
                                }
                            },
                        }
                        for compound_add in &mut compound_adds {
                            compound_add.extend(extend.clone());
                        }
                    }
                    for compound_add in &compound_adds {
                        compounds.insert(compound.name().value(), compound_add.clone());
                    }
                }
            }

            "cut" => {
                let mut turn_seqs = vec![Vec::new()];
                let mut extend = Vec::new();
                for val in node.entries() {
                    match val.value().as_string()?.strip_suffix("'") {
                        None => match strip_number_end(val.value().as_string()?) {
                            None => match val.value().as_string()?.strip_suffix("*") {
                                None => extend = compounds.get(val.value().as_string()?)?.clone(),
                                Some(real) => {
                                    let turn = *twists.get(real)?;
                                    let number: isize =
                                        ((2.0 * PI as f64) / turn.angle.abs()) as isize;
                                    let mut new_adds = Vec::new();
                                    for add in &turn_seqs {
                                        for i in 1..number {
                                            let mut new_add = add.clone();
                                            new_add.push(i * turn);
                                            new_adds.push(new_add);
                                        }
                                    }
                                    turn_seqs.extend(new_adds);
                                }
                            },
                            Some(real) => {
                                extend = multiply_turns(
                                    real.1.parse::<isize>().unwrap(),
                                    compounds.get(real.0.as_str())?,
                                );
                            }
                        },
                        Some(real) => match strip_number_end(real) {
                            None => {
                                extend = invert_compound_turn(compounds.get(real)?);
                            }
                            Some(inside) => {
                                extend = invert_compound_turn(&multiply_turns(
                                    inside.1.parse::<isize>().unwrap(),
                                    compounds.get(inside.0.as_str())?,
                                ))
                            }
                        },
                    }
                    for turns in &mut turn_seqs {
                        turns.extend(extend.clone());
                    }
                }
                for turns in &turn_seqs {
                    puzzle.cut(turns).ok()?;
                    puzzle.pieces.len();
                }
            }
            "colors" => {
                for color in node.children()?.nodes() {
                    colors.insert(
                        color.name().value().to_string(),
                        Color32::from_rgb(
                            color.entries().get(0)?.value().as_integer()? as u8,
                            color.entries().get(1)?.value().as_integer()? as u8,
                            color.entries().get(2)?.value().as_integer()? as u8,
                        ),
                    );
                }
            }
            "color" => {
                let color = *colors.get(node.entries()[0].value().as_string()?)?;
                let mut coloring_circles = Vec::new();
                for i in 1..node.entries().len() {
                    let circle = node.entries().get(i)?.value().as_string()?;
                    coloring_circles.push(match circle.strip_prefix("!") {
                        None => (*circles.get(circle)?, Contains::Inside),
                        Some(real) => (*circles.get(real)?, Contains::Outside),
                    });
                }
                puzzle.color(&(coloring_circles, color));
            }
            "twist" => {
                let mut sequence = Vec::new();
                for val in node.entries() {
                    let extend;
                    match val.value().as_string()?.strip_suffix("'") {
                        None => match strip_number_end(val.value().as_string()?) {
                            None => {
                                extend = compounds.get(val.value().as_string()?)?.clone();
                            }
                            Some(real) => {
                                extend = multiply_turns(
                                    real.1.parse::<isize>().unwrap(),
                                    compounds.get(real.0.as_str())?,
                                );
                            }
                        },
                        Some(real) => match strip_number_end(real) {
                            None => {
                                extend = invert_compound_turn(compounds.get(real)?);
                            }
                            Some(inside) => {
                                extend = invert_compound_turn(&multiply_turns(
                                    inside.1.parse::<isize>().unwrap(),
                                    compounds.get(inside.0.as_str())?,
                                ))
                            }
                        },
                    }
                    sequence.extend(extend);
                }
                let mut add_seq = Vec::new();
                for turn in &sequence {
                    puzzle.turn(*turn, false).ok()?;
                    add_seq.push(turn.clone());
                }
                def_stack.push(add_seq);
            }
            "undo" => {
                let mut number;
                if node.entries().is_empty() {
                    number = 1;
                } else {
                    let entry = &node.entries().get(0)?;
                    match entry.value().as_integer() {
                        None => {
                            number = -1;
                        }
                        Some(num) => {
                            number = num;
                        }
                    }
                }
                while number != 0 {
                    number -= 1;
                    if let Some(turns) = def_stack.pop() {
                        for turn in invert_compound_turn(&turns) {
                            puzzle.turn(turn, false).ok()?;
                        }
                    } else {
                        break;
                    }
                }
            }
            _ => (),
        }
    }
    for turn in real_twists {
        puzzle.turns.insert(String::from(turn.0), turn.1);
        puzzle
            .turns
            .insert(String::from(turn.0) + "'", turn.1.inverse());
    }
    puzzle.solved_state = puzzle.pieces.clone();
    puzzle.animation_offset = NONE_TURN;
    puzzle.stack = Vec::new();
    return Some(puzzle);
}

fn write_to_file(def: &String, stack: &Vec<String>, path: &str) -> Result<(), std::io::Error> {
    let curr_path = match DEV {
        false => String::from(
            std::env::current_exe()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .split("circleguy.exe")
                .into_iter()
                .collect::<Vec<&str>>()[0],
        ),
        true => String::new(),
    };
    let real_path = curr_path + path;
    let mut buffer = OpenOptions::new()
        .write(true)
        .create(true)
        .open(real_path)?;
    buffer.write(get_puzzle_string(def.clone(), stack).as_str().as_bytes())?;
    Ok(())
}

fn get_puzzle_string(def: String, stack: &Vec<String>) -> String {
    if stack.is_empty() {
        return def;
    };
    return def + "\n --LOG FILE \n" + &stack.join(",");
}

struct App {
    data_storer: DataStorer,
    puzzle: Puzzle,
    def_string: String,
    log_path: String,
    curr_msg: String,
    animation_speed: f64,
    last_frame_time: web_time::Instant,
    outline_width: f32,
    detail: f64,
    scale_factor: f32,
    offset: Vec2F64,
    cut_on_turn: bool,
    preview: bool,
}
impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut data_storer = DataStorer { data: Vec::new() };
        let _ = data_storer.load_puzzles(&String::from("Puzzles/Definitions/"));
        return Self {
            data_storer,
            puzzle: load_puzzle_and_def_from_file(&String::from(
                "Puzzles/Definitions/1010101010geranium.kdl",
            ))
            .unwrap()
            .0,
            def_string: load_puzzle_and_def_from_file(&String::from(
                "Puzzles/Definitions/1010101010geranium.kdl",
            ))
            .unwrap()
            .1,
            log_path: String::from("logfile"),
            curr_msg: String::new(),
            animation_speed: ANIMATION_SPEED,
            last_frame_time: web_time::Instant::now(),
            outline_width: 5.0,
            detail: DETAIL,
            scale_factor: SCALE_FACTOR,
            offset: vec2_f64(0.0, 0.0),
            cut_on_turn: false,
            preview: false,
        };
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let good_detail = 1.0 / (self.detail);
            if !self.preview {
                self.puzzle.render(
                    ui,
                    &rect,
                    good_detail,
                    self.outline_width,
                    self.scale_factor,
                    self.offset,
                );
            } else {
                for piece in &self.puzzle.solved_state {
                    piece.render(
                        ui,
                        &rect,
                        NONE_TURN,
                        good_detail,
                        self.outline_width,
                        self.scale_factor,
                        self.offset,
                    );
                }
            }
            match self.data_storer.render_panel(ctx) {
                Err(()) => {
                    self.curr_msg =
                        String::from("Failed to render side panel or failed to create puzzle!")
                }
                Ok(Some(puz)) => {
                    (self.puzzle, self.def_string) = puz;
                }
                _ => {}
            }
            let delta_time = self.last_frame_time.elapsed();
            self.last_frame_time = web_time::Instant::now();
            if self.puzzle.animation_offset.angle >= 0.0 {
                self.puzzle.animation_offset.angle = f64::max(
                    self.puzzle.animation_offset.angle
                        - (delta_time.as_secs_f64() * self.animation_speed),
                    0.0,
                );
            } else {
                self.puzzle.animation_offset.angle = f64::min(
                    self.puzzle.animation_offset.angle
                        + (delta_time.as_secs_f64() * self.animation_speed),
                    0.0,
                );
            }
            if aleq(25.0, self.animation_speed) {
                self.puzzle.animation_offset = NONE_TURN;
            }
            if (ui.add(egui::Button::new("UNDO")).clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Z)))
                && !self.preview
            {
                let _ = self.puzzle.undo();
            }
            if ui.add(egui::Button::new("SCRAMBLE")).clicked() && !self.preview {
                let _ = self.puzzle.scramble(self.cut_on_turn);
            }
            if ui.add(egui::Button::new("RESET")).clicked() && !self.preview {
                self.puzzle.reset();
            }
            ui.add(
                egui::Slider::new(&mut self.outline_width, (0.0)..=(10.0)).text("Outline Width"),
            );
            ui.add(egui::Slider::new(&mut self.detail, (1.0)..=(100.0)).text("Detail"));
            ui.add(
                egui::Slider::new(&mut self.animation_speed, (1.0)..=(25.0))
                    .text("Animation Speed"),
            );
            ui.add(
                egui::Slider::new(&mut self.scale_factor, (10.0)..=(5000.0)).text("Rendering Size"),
            );
            // ui.add(egui::Slider::new(&mut def.r_left, (0.01)..=(2.0)).text("Left Radius"));
            // ui.add(egui::Slider::new(&mut def.n_left, 2..=50).text("Left Number"));
            // ui.add(egui::Slider::new(&mut def.r_right, (0.01)..=(2.0)).text("Right Radius"));
            // ui.add(egui::Slider::new(&mut def.n_right, 2..=50).text("Right Number"));
            ui.add(egui::Slider::new(&mut self.offset.x, (-2.0)..=(2.0)).text("Move X"));
            ui.add(egui::Slider::new(&mut self.offset.y, (-2.0)..=(2.0)).text("Move Y"));
            // ui.add(egui::Slider::new(&mut def.depth, 0..=5000).text("Scramble Depth"));
            if ui.add(egui::Button::new("RESET VIEW")).clicked() {
                (self.scale_factor, self.offset) = (SCALE_FACTOR, vec2_f64(0.0, 0.0))
            }
            ui.label("Log File Path");
            ui.add(egui::TextEdit::singleline(&mut self.log_path));
            if ui.add(egui::Button::new("SAVE")).clicked() {
                self.curr_msg = match write_to_file(
                    &self.def_string,
                    &self.puzzle.stack,
                    &(String::from("Puzzles/Logs/") + &self.log_path + ".kdl"),
                ) {
                    Ok(()) => String::new(),
                    Err(err) => err.to_string(),
                }
            }
            if ui.add(egui::Button::new("LOAD LOG")).clicked() {
                (self.puzzle, self.def_string) = load_puzzle_and_def_from_file(
                    &(String::from("Puzzles/Logs/") + &self.log_path + ".kdl"),
                )
                .unwrap_or((self.puzzle.clone(), self.def_string.clone()));
            }
            if ui.add(egui::Button::new("RELOAD PUZZLES")).clicked() {
                let _ = self.data_storer.load_puzzles("Puzzles/Definitions/");
            }
            // if ui.add(egui::Button::new("GENERATE")).clicked()
            //     && alneq(1.0, def.r_left + def.r_right)
            // {
            //     puzzle = load(def.clone(), &mut def);
            // }
            // let new_p = data.show_puzzles(ui, &rect);
            // if new_p.is_some() {
            //     puzzle = load(new_p.unwrap(), &mut def);
            // }
            ui.checkbox(&mut self.cut_on_turn, "Cut on turn?");
            ui.checkbox(&mut self.preview, "Preview solved state?");
            ui.label(String::from("Name: ") + &self.puzzle.name.clone());
            ui.label(String::from("Authors: ") + &self.puzzle.authors.join(","));
            ui.label(self.puzzle.pieces.len().to_string() + " pieces");
            if !self.curr_msg.is_empty() {
                ui.label(&self.curr_msg);
            }
            if self.puzzle.solved {
                ui.label("Solved!");
            }
            let cor_rect = Rect {
                min: pos2(180.0, 0.0),
                max: pos2(rect.width() - 180.0, rect.height()),
            };
            // dbg!((puzzle.turns[1].circle.center).to_pos2());
            if self.puzzle.animation_offset.angle != 0.0 {
                ui.ctx().request_repaint();
            }
            let r = ui.interact(cor_rect, egui::Id::new(19), egui::Sense::all());
            let scroll = ui.input(|input| {
                input
                    .raw
                    .events
                    .iter()
                    .filter_map(|ev| match ev {
                        Event::MouseWheel {
                            unit: MouseWheelUnit::Line | MouseWheelUnit::Page,
                            delta,
                            modifiers: _,
                        } => Some((delta.x + delta.y).signum() as i32),
                        _ => None,
                    })
                    .sum::<i32>()
            });
            if r.clicked() && !self.preview {
                let _ = self.puzzle.process_click(
                    &rect,
                    r.interact_pointer_pos().unwrap(),
                    true,
                    self.scale_factor,
                    self.offset,
                    self.cut_on_turn,
                );
            }
            if r.clicked_by(egui::PointerButton::Secondary) && !self.preview {
                let _ = self.puzzle.process_click(
                    &rect,
                    r.interact_pointer_pos().unwrap(),
                    false,
                    self.scale_factor,
                    self.offset,
                    self.cut_on_turn,
                );
            }
            if r.hover_pos().is_some() && !self.preview {
                let hovered_circle = self.puzzle.get_hovered(
                    &rect,
                    r.hover_pos().unwrap(),
                    self.scale_factor,
                    self.offset,
                );
                if hovered_circle.radius > 0.0 {
                    hovered_circle.draw(ui, &rect, self.scale_factor, self.offset);
                }
                if scroll != 0
                    && !r.dragged_by(egui::PointerButton::Middle)
                    && !ui.input(|i| i.modifiers.command_only())
                    && !self.preview
                {
                    let _ = self.puzzle.process_click(
                        &rect,
                        r.hover_pos().unwrap(),
                        scroll > 0,
                        self.scale_factor,
                        self.offset,
                        self.cut_on_turn,
                    );
                }
            }
            if r.dragged_by(egui::PointerButton::Middle) {
                let delta = r.drag_delta();
                let good_delta = vec2_f64(
                    (delta.x / self.scale_factor) as f64,
                    -1.0 * (delta.y / self.scale_factor) as f64,
                );
                self.offset = self.offset + good_delta;
            }
            if ui.input(|i| i.modifiers.command_only()) && scroll != 0 {
                self.scale_factor += 10.0 * scroll as f32;
                // if r.hover_pos().is_some() {
                //     let pos = from_egui_coords(
                //         &r.hover_pos().unwrap(),
                //         &rect,
                //         self.scale_factor,
                //         self.offset,
                //     );
                //     let curr_center =
                //         from_egui_coords(&rect.center(), &rect, self.scale_factor, self.offset)
                //             + self.offset;
                //     self.offset = self.offset + (curr_center - pos);
                //}
            }
        });
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "circleguy",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
