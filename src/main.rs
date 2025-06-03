use std::{any, cmp::Ordering, f32::consts::PI, iter::once};

use eframe::glow::NONE;
use rand::prelude::*;

use colorous;

use egui::{Color32, Pos2, Rect, Stroke, Ui, epaint::PathShape, pos2};

const DETAIL: u16 = 50;

const OUTLINE_COLOR: Color32 = Color32::BLACK;

const SPECTRUM: colorous::Gradient = colorous::TURBO;

const SCALE_FACTOR: f32 = 5.0;

const ANIMATION_SPEED: f32 = 5.0;

const LENIENCY: f32 = 0.001;

const NONE_CIRCLE: Circle = Circle {
    center: pos2(0.0, 0.0),
    radius: 0.0,
};

const NONE_TURN: Turn = Turn {
    circle: NONE_CIRCLE,
    angle: 0.0,
};

#[derive(Clone)]
struct Piece {
    shape: Vec<Arc>,
    color: Color32,
}
#[derive(Clone)]
struct Puzzle {
    pieces: Vec<Piece>,
    turns: Vec<Turn>,
    stack: Vec<Turn>,
    animation_offset: Turn,
}
#[derive(Clone, Copy)]
struct Circle {
    center: Pos2,
    radius: f32,
}
#[derive(Clone, Copy)]
struct Turn {
    circle: Circle,
    angle: f32,
}
#[derive(Clone, Copy)]

//orientation: true means that the 'inside' of the arc is the 'inside' of the piece, false means the opposite
struct Arc {
    start: Pos2,
    angle: f32,
    circle: Circle,
}

//checking if certain float-based variable types are approximately equal due to precision bs

fn aeq(f1: f32, f2: f32) -> bool {
    return (f1 - f2).abs() <= LENIENCY;
}

fn aeq_pos(p1: Pos2, p2: Pos2) -> bool {
    return (aeq(p1.x, p2.x) && aeq(p1.y, p2.y));
}

fn aeq_circ(c1: Circle, c2: Circle) -> bool {
    return (aeq_pos(c1.center, c2.center) && aeq(c1.radius, c2.radius));
}

fn aeq_arc(a1: Arc, a2: Arc) -> bool {
    return (aeq_circ(a1.circle, a2.circle) && aeq(a1.angle, a2.angle));
}

fn cmp_f32(a: f32, b: f32) -> Ordering {
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

impl Circle {
    //draw the circle on the ui
    fn draw(&self, ui: &mut Ui, rect: &Rect) {
        ui.painter().circle_stroke(
            to_egui_coords(&self.center, rect),
            self.radius * SCALE_FACTOR * (rect.width() / 1920.0),
            (9.0, Color32::WHITE),
        );
    }
    //rotate the circle about a point
    fn rotate_about(&self, center: Pos2, angle: f32) -> Circle {
        return Circle {
            center: rotate_about(center, self.center, angle),
            radius: self.radius,
        };
    }
    //check if the circle contains a point (including on the boundary). LENIENCY included to account for floating point stuff
    fn contains_inside(&self, point: Pos2) -> bool {
        return (self.center.distance(point) <= self.radius + LENIENCY);
    }
    //check if the circle contains a point on its border/circumference
    fn contains_border(&self, point: Pos2) -> bool {
        return aeq(self.center.distance(point), self.radius);
    }
    //check if a circle contains an arc -- BAD/REWORK
    fn contains_arc(&self, arc: &Arc) -> bool {
        let bigger_circle = Circle {
            radius: self.radius + LENIENCY,
            center: self.center,
        };
        return (arc.intersect_circle(&bigger_circle).len() <= 1
            && self.contains_inside(arc.start)
            && self.contains_inside(arc.end()));
    }
}

impl Arc {
    fn rotate_about(&self, center: Pos2, angle: f32) -> Arc {
        return Arc {
            start: rotate_about(center, self.start, angle),
            angle: self.angle,
            circle: self.circle.rotate_about(center, angle),
        };
    }
    fn end(&self) -> Pos2 {
        return rotate_about(self.circle.center, self.start, self.angle);
    }
    fn midpoint(&self) -> Pos2 {
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
    fn get_angle(&self, point: &Pos2) -> f32 {
        let mut angle = angle_on_circle(self.start, point.clone(), self.circle);
        if self.angle.is_sign_negative() {
            angle = angle - (2.0 * PI);
        }
        return angle;
    }
    //check if an arc contains a point
    fn contains(&self, point: Pos2) -> bool {
        if !self.circle.contains_border(point) {
            return false;
        }
        let angle = angle_on_circle(self.start, point, self.circle);
        if self.angle >= 0.0 {
            //use approximate comparison - AAAAAAAAAAAAAAAAA
            return (0.0 <= angle && angle <= self.angle)
                || (angle < 0.0 && (2.0 * PI + angle) <= self.angle);
        } else {
            return (0.0 >= angle && angle >= self.angle)
                || (angle > 0.0 && (-2.0 * PI + angle) >= self.angle);
        }
    }
    //get the points where the arc intersects a circle
    fn intersect_circle(&self, circle: &Circle) -> Vec<Pos2> {
        let intersect = circle_intersection(&circle, &self.circle);
        let mut points = Vec::new();
        for point in intersect {
            if self.contains(point) {
                points.push(point);
            }
        }
        return points;
    }
    //get the points where the arc intersects another arc
    fn intersect_arc(&self, arc: Arc) -> Vec<Pos2> {
        let intersect = arc.intersect_circle(&self.circle);
        let mut points = Vec::new();
        for point in intersect {
            if self.contains(point) {
                points.push(point);
            }
        }
        return points;
    }
    //get a polygon representation of the arc for rendering
    fn get_polygon(&self, detail: u16) -> Vec<Pos2> {
        let mut points: Vec<Pos2> = Vec::new();
        let inc_angle = self.angle / (detail as f32);
        points.push(self.start);
        for i in 1..=detail {
            points.push(rotate_about(
                self.circle.center,
                self.start,
                inc_angle * (i as f32),
            ));
        }
        return points;
    }
    //cut the arc into smaller arcs by a circle
    fn cut_by_circle(&self, circle: Circle) -> Vec<Arc> {
        let intersects = self.intersect_circle(&circle);
        return self.cut_at(&intersects);
    }
    //takes a vec of points (must be on the arc, and not the endpoints) and returns the sorted version of them, as they appear on the arc
    //order in the sort_by call may just be wrong, reverse it potentially
    fn order_on_arc(&self, points: &Vec<Pos2>) -> Vec<Pos2> {
        let mut new_points = points.clone();
        new_points.sort_by(|a, b| cmp_f32(self.get_angle(a).abs(), self.get_angle(b).abs()));
        return new_points;
    }
    //cut at points in any order. all points must be on arc
    //returns a Vec<Arc> that should have the resulting arcs in order from self.start to self.end()
    fn cut_at(&self, points: &Vec<Pos2>) -> Vec<Arc> {
        let mut total_points = vec![self.start];
        let sorted_points = self.order_on_arc(points);
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

    fn draw(&self, ui: &mut Ui, rect: &Rect, detail: u16) {
        let mut coords = Vec::new();
        for pos in self.get_polygon(detail) {
            coords.push(to_egui_coords(&pos, rect));
        }
        ui.painter()
            .add(PathShape::line(coords, Stroke::new(3.0, Color32::RED)));
    }
}

impl Puzzle {
    fn turn(&mut self, turn: Turn) {
        let mut new_pieces = Vec::new();
        for piece in &self.pieces {
            let mut new_piece = piece.clone();
            if new_piece.in_circle(&turn.circle) {
                new_piece.rotate_about(turn.circle.center, turn.angle);
            }
            new_pieces.push(new_piece);
        }
        self.pieces = new_pieces;
        self.animation_offset = turn.inverse();
        self.stack.push(turn);
    }
    fn turn_id(&mut self, id: usize) {
        let turn = self.turns[id];
        self.turn(turn);
    }
    fn undo(&mut self) {
        if self.stack.len() == 0 {
            return;
        }
        let last_turn = self.stack.pop().unwrap();
        self.turn(last_turn.inverse());
        self.stack.pop();
    }
    fn render(&self, ui: &mut Ui, rect: &Rect, detail: u16) {
        for piece in &self.pieces {
            piece.render(ui, rect, self.animation_offset, detail);
        }
    }
    fn process_click(&mut self, rect: &Rect, pos: Pos2, left: bool) {
        let good_pos = from_egui_coords(&pos, rect);
        for turn in self.turns.clone() {
            if good_pos.distance(turn.circle.center) <= turn.circle.radius
                && (turn.angle > 0.0) == left
            {
                self.turn(turn);
                return;
            }
        }
    }
    fn get_hovered(&self, rect: &Rect, pos: Pos2) -> Circle {
        let good_pos = from_egui_coords(&pos, rect);
        for turn in self.turns.clone() {
            if good_pos.distance(turn.circle.center) <= turn.circle.radius {
                return turn.circle;
            }
        }
        return NONE_CIRCLE;
    }
    fn cut_by_circle(&mut self, circle: Circle, turn: Turn) {
        let mut new_pieces = Vec::new();
        for piece in &self.pieces {
            if piece.in_circle(&turn.circle) {
                new_pieces.extend(piece.cut_by_circle(circle));
            } else {
                new_pieces.push(piece.clone());
            }
        }
        self.pieces = new_pieces;
    }
    fn cut_with_turn(&mut self, circle: Circle, turn: Turn) {
        let mut index = 0;
        while !aeq(turn.angle * (index as f32), 2.0 * PI) {
            self.cut_by_circle(
                circle.rotate_about(turn.circle.center, turn.angle * (index as f32)),
                turn,
            );
            index += 1;
        }
    }
    fn draw_outline(&self, ui: &mut Ui, rect: &Rect, detail: u16) {
        for piece in &self.pieces {
            piece.draw_outline(ui, rect, detail);
        }
    }
}

impl Piece {
    fn rotate_about(&mut self, center: Pos2, angle: f32) {
        let mut new_arcs: Vec<Arc> = Vec::new();
        for mut arc in self.shape.clone() {
            new_arcs.push(arc.rotate_about(center, angle));
        }
        self.shape = new_arcs;
    }
    fn render(&self, ui: &mut Ui, rect: &Rect, offset: Turn, detail: u16) {
        let mut true_offset = NONE_TURN;
        if self.in_circle(&offset.circle) {
            true_offset = offset;
        }
        let mut render_points: Vec<Pos2> = Vec::new();
        for arc in self.shape.clone() {
            render_points.extend(arc.get_polygon(detail))
        }
        let mut good_render_points = Vec::new();
        for point in render_points {
            good_render_points.push(to_egui_coords(&point, rect));
        }
        ui.painter().add(egui::Shape::convex_polygon(
            good_render_points,
            self.color,
            (2.0, OUTLINE_COLOR),
        ));
    }
    fn in_circle(&self, circle: &Circle) -> bool {
        for arc in &self.shape {
            if !circle.contains_arc(arc) {
                return false;
            }
        }
        return true;
    }
    // ASSUMPTIONS FOR NOW:
    //the circle is not tangent to any arc in the piece
    //the circle does not intersect any arc at its endpoint
    fn cut_by_circle(&self, circle: Circle) -> Vec<Piece> {
        let mut shapes: Vec<Vec<Arc>> = Vec::new(); // the shapes of the final pieces
        let mut start_points: Vec<Pos2> = Vec::new(); // the start points (ccw, wrt circle) of the arcs of the circle contained in the piece
        let mut end_points: Vec<Pos2> = Vec::new(); // the end points "
        let mut piece_arcs = Vec::new(); // the arcs obtained by cutting up the piece by the circle
        for arc in &self.shape {
            let arc_bits = arc.cut_by_circle(circle);
            piece_arcs.extend(arc_bits);
        } //populate piece_arcs
        //println!("{}", piece_arcs.len());
        //println!("{}", piece_arcs.len());
        for arc in &piece_arcs {
            if circle.contains_border(arc.start) {
                if circle.contains_inside(arc.midpoint()) {
                    end_points.push(arc.start);
                } else {
                    start_points.push(arc.start);
                }
            }
        } // populate start and end
        if start_points.is_empty() {
            return vec![self.clone()];
        }
        if !circle.contains_inside(piece_arcs[0].start) {
            end_points.rotate_left(1);
        } //sync the indexing in start and end in this case
        let mut circle_arcs = Vec::new(); // arcs created by cutting the circle that land inside the piece
        for i in 0..start_points.len() {
            let circle_arc = get_arc(start_points[i], end_points[i], circle, true);
            //println!("{}", circle_arc.angle);
            circle_arcs.push(circle_arc);
            circle_arcs.push(circle_arc.invert());
        } //populate circle_bits -- we add both directions since each circle arc will be used by 2 pieces
        let mut on_circle: bool = false;
        while piece_arcs.len() != 0 && circle_arcs.len() != 0 {
            //println!("{}, {}", piece_arcs.len(), circle_arcs.len());
            //iterate through the list of all the arcs
            let mut curr_arc = piece_arcs[0]; //start at the first arc that remains from the piece arcs(arbitrary)
            let mut curr_shape = vec![curr_arc]; //a shape created from these arcs
            piece_arcs.remove(0); //remove the first arc
            loop {
                let mut get_arc_piece = true; //whether or not we want to grab the arc from piece_arcs
                if !on_circle {
                    let circle_arc = pop_arc_from_vec_from_start(curr_arc.end(), &mut circle_arcs);
                    if let Some(c) = circle_arc {
                        // if there is a circle arc in the right spot, use that
                        curr_arc = c;
                        on_circle = true;
                        get_arc_piece = false;
                    }
                }
                if get_arc_piece {
                    //otherwise, use the proper piece arc
                    curr_arc =
                        pop_arc_from_vec_from_start(curr_arc.end(), &mut piece_arcs).unwrap();
                    on_circle = false;
                }
                curr_shape.push(curr_arc);
                if aeq_pos(curr_arc.end(), curr_shape[0].start) {
                    //if the closed shape is created
                    shapes.push(curr_shape);
                    break;
                }
            }
        }
        let mut pieces = Vec::new();
        for shape in shapes {
            pieces.push(Piece {
                shape: shape.clone(),
                color: Color32::RED,
            });
        }
        return pieces;
    }
    fn draw_outline(&self, ui: &mut Ui, rect: &Rect, detail: u16) {
        for arc in &self.shape {
            arc.draw(ui, rect, detail);
        }
    }
}

fn pop_arc_from_vec_from_start(start: Pos2, arcs: &mut Vec<Arc>) -> Option<Arc> {
    for i in 0..arcs.len() {
        if aeq_pos(arcs[i].start, start) {
            let ret_arc = arcs[i].clone();
            arcs.remove(i);
            return Some(ret_arc);
        }
    }
    return None;
}

fn get_arc_starting_at(point: Pos2, arcs: Vec<Arc>) -> Option<Arc> {
    for arc in arcs {
        if aeq_pos(arc.start, point) {
            return Some(arc);
        }
    }
    return None;
}

//gives the angle between two points, counterclockwise point1 -> point2, on the same circle. angle is in (0, 2PI]
//BAD - REWORK
fn angle_on_circle(point1: Pos2, point2: Pos2, circle: Circle) -> f32 {
    let mut angle: f32 = ((circle.center.distance_sq(point1) - (0.5 * point1.distance_sq(point2)))
        / (circle.center.distance_sq(point1)))
    .acos();
    if !aeq_pos(rotate_about(circle.center, point1, angle), point2) {
        return (2.0 * PI) - angle;
    }
    if aeq(angle, 0.0) {
        angle = 2.0 * PI;
    }
    return angle;
}

fn arc_from_circle(circle: Circle, start: Pos2, ccw: bool) -> Arc {
    return get_arc(start, start, circle, ccw);
}

//when start == end is passed, a full circle is returned
//ccw is true -> creates counterclockwise start -> end, ccw = false does clockwise

fn get_arc(start: Pos2, end: Pos2, circle: Circle, ccw: bool) -> Arc {
    let mut angle = angle_on_circle(start, end, circle);
    if aeq(angle, 0.0) {
        angle = 2.0 * PI;
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
            angle: (-2.0 * PI) + angle,
            circle: circle,
        };
    }
}

//translates from nice coords to egui coords
fn to_egui_coords(pos: &Pos2, rect: &Rect) -> Pos2 {
    return pos2(
        pos.x * (SCALE_FACTOR * rect.width() / 1920.0) + (rect.width() / 2.0),
        -1.0 * pos.y * (SCALE_FACTOR * rect.width() / 1920.0) + (rect.height() / 2.0),
    );
}

//translates from egui coords to nice coords
fn from_egui_coords(pos: &Pos2, rect: &Rect) -> Pos2 {
    return pos2(
        (pos.x - (rect.width() / 2.0)) * (1920.0 / (SCALE_FACTOR * rect.width())),
        (pos.y - (rect.height() / 2.0)) * (-1920.0 / (SCALE_FACTOR * rect.width())),
    );
}

//rotate a point about a point a certain angle
fn rotate_about(center: Pos2, point: Pos2, angle: f32) -> Pos2 {
    if center == point {
        return point;
    }
    let dist: f32 = center.distance(point);
    let mut curr_angle: f32 = ((point.y - center.y) / dist).asin();
    if point.x < center.x {
        curr_angle = PI - curr_angle;
    }
    let end_angle: f32 = angle + curr_angle;
    return pos2(
        center.x + (dist * end_angle.cos()),
        center.y + (dist * end_angle.sin()),
    );
}

//needs to draw ccw with respect to the circle
fn circle_region(detail: u16, center: Pos2, point1: Pos2, point2: Pos2) -> Vec<Pos2> {
    let angle: f32 = ((center.distance_sq(point1) - (0.5 * point1.distance_sq(point2)))
        / (center.distance_sq(point1)))
    .acos();
    let mut point: Pos2 = point1.clone();
    let mut points: Vec<Pos2> = Vec::new();
    for i in 0..detail {
        points.push(point.clone());
        point = rotate_about(center, point, angle / f32::from(detail));
    }
    return points;
}

//returns the 0-2 circle intersection points. the one that's clockwise above the horizon from circle1 is returned first
fn circle_intersection(circle1: &Circle, circle2: &Circle) -> Vec<Pos2> {
    if circle1.center.distance(circle2.center) > circle1.radius + circle2.radius
        || circle1.center.distance(circle2.center) + circle2.radius < circle1.radius
        || circle1.center.distance(circle2.center) + circle1.radius < circle2.radius
    {
        return Vec::new();
    }
    if circle1.center.distance(circle2.center) == circle1.radius + circle2.radius {
        // write circle inside other circle and tangent
        return vec![
            (circle1.center + (circle1.radius * (circle2.center - circle1.center).normalized())),
        ];
    }
    let dist_sq = circle1.center.distance_sq(circle2.center);
    let angle: f32 = ((dist_sq + (circle1.radius * circle1.radius)
        - (circle2.radius * circle2.radius))
        / (2.0 * circle1.radius * circle1.center.distance(circle2.center)))
    .acos();
    let difference = circle2.center - circle1.center;
    let unit_difference = difference.normalized();
    let arc_point = circle1.center + (circle1.radius * unit_difference);
    let point1 = rotate_about(circle1.center, arc_point, -1.0 * angle);
    let point2 = rotate_about(circle1.center, arc_point, angle);
    return (vec![point1, point2]);
}

//need to pass in a Vec<Arc> shape with shape[n].end() == shape[n + 1].start and also shape[-1].end() == shape[0].start

fn get_color(i: u16, n: u16) -> Color32 {
    let color = SPECTRUM.eval_rational((i + 1) as _, (n + 1) as _);
    return Color32::from_rgb(color.r, color.g, color.b);
}

fn get_color_range(start: u16, number: u16, total: u16) -> Vec<Color32> {
    let mut colors: Vec<Color32> = Vec::new();
    for i in 0..number {
        colors.push(get_color(start + i, total));
    }
    return colors;
}

// fn puzzle_from_circles(circles: Vec<Circle>, numbers: Vec<u16>) -> Puzzle {
//     let mut puzzle = Puzzle {
//         animation_offset: NONE_TURN,
//         stack: Vec::new(),
//         turns: Vec::new(),
//         pieces: Vec::new(),
//     };
//     let point = pos2(circles[0].center.x + circles[0].radius, circles[0].center.y);
//     let piece = Piece {
//         shape: vec![get_arc(point, point, circles[0], true)],
//         color: Color32::RED,
//     };
//     puzzle.pieces.push(piece);
//     puzzle.turns.push(Turn {});
//     for i in 1..circles.len() {
//         let circle = circles[i];

//     }
// }

fn puzzle_from_two_circles(c1: Circle, c2: Circle, n1: u16, n2: u16) -> Puzzle {
    let point = pos2(c1.center.x - c1.radius, c1.center.y);
    let piece = Piece {
        shape: vec![get_arc(point, point, c1, true)],
        color: Color32::RED,
    };
    let mut puzzle = Puzzle {
        animation_offset: NONE_TURN,
        stack: Vec::new(),
        turns: vec![
            Turn {
                circle: c1,
                angle: (2.0 * PI) / (n1 as f32),
            },
            Turn {
                circle: c2,
                angle: (2.0 * PI) / (n2 as f32),
            },
        ],
        pieces: vec![piece],
    };
    let point2 = pos2(c2.center.x + c2.radius, c2.center.y);
    let arc = get_arc(point2, point2, c2, true);
    let piece2 = Piece {
        shape: vec![arc],
        color: Color32::RED,
    };
    puzzle.cut_with_turn(
        c2,
        Turn {
            circle: c1,
            angle: (2.0 * PI) / (n1 as f32),
        },
    );
    puzzle.pieces.push(piece2.cut_by_circle(c1)[0].clone());
    return puzzle;
}
fn main() -> eframe::Result {
    let mut rng = rand::rng();
    let mut last_frame_time = std::time::Instant::now();
    let left_center = pos2(-50.0, 0.0);
    let right_center = pos2(50.0, 0.0);
    let mut left_slider_value: u16 = 5;
    let mut right_slider_value: u16 = 5;
    let mut number_l = left_slider_value;
    let mut number_r = right_slider_value;
    let point = pos2(-50.0, 0.0);
    let circle1 = Circle {
        radius: 50.0,
        center: pos2(0.0, 0.0),
    };
    let full_circle_arc = get_arc(point, point, circle1, true);
    let shape = vec![full_circle_arc];
    let mut piece = Piece {
        shape: shape,
        color: Color32::RED,
    };
    let circle2 = Circle {
        radius: 50.0,
        center: pos2(60.0, 0.0),
    };
    let circle3 = Circle {
        radius: 10.0,
        center: pos2(50.0, 0.0),
    }
    .rotate_about(circle1.center, 0.125 * PI);
    let mut puzzle = Puzzle {
        pieces: vec![piece],
        animation_offset: NONE_TURN,
        stack: Vec::new(),
        turns: vec![Turn {
            circle: circle1,
            angle: 0.25 * PI,
        }],
    };
    let circle4 = Circle {
        radius: 15.0,
        center: pos2(20.0, 0.0),
    };
    let circle5 = Circle {
        radius: 6.0,
        center: pos2(50.0, 0.0),
    };
    puzzle.cut_with_turn(circle2, puzzle.turns[0]);
    puzzle.cut_with_turn(circle3, puzzle.turns[0]);
    puzzle.cut_with_turn(circle4, puzzle.turns[0]);
    puzzle.cut_with_turn(circle5, puzzle.turns[0]);
    //let mut puzzle = puzzle_from_two_circles(circle1, circle2, 4, 4);
    //println!("{}", puzzle.pieces[0].shape.len());
    let mut prev_states: Vec<Puzzle> = Vec::new();
    eframe::run_simple_native("circleguy", Default::default(), move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            //circle3.draw(ui, &rect);
            puzzle.draw_outline(ui, &rect, 50);
            //puzzle.pieces[0].shape[1].draw(ui, &rect, 50);
            // let delta_time = last_frame_time.elapsed();
            // last_frame_time = std::time::Instant::now();
            // if puzzle.animation_offset.angle >= 0.0 {
            //     puzzle.animation_offset.angle = f32::max(
            //         puzzle.animation_offset.angle - (delta_time.as_secs_f32() * ANIMATION_SPEED),
            //         0.0,
            //     );
            // } else {
            //     puzzle.animation_offset.angle = f32::min(
            //         puzzle.animation_offset.angle + (delta_time.as_secs_f32() * ANIMATION_SPEED),
            //         0.0,
            //     );
            // }

            // let angle_l = (2.0 * PI) / f32::from(number_l);
            // let angle_r = (2.0 * PI) / f32::from(number_r);

            // let width = rect.width();
            // let height = rect.height();
            // let minimum = rect.size().min_elem();
            // let center = pos2(rect.width() * 0.5, rect.height() * 0.5);

            // for i in 0..puzzle2.pieces.len() {
            //     let piece = &puzzle2.pieces[i];
            //     ui.painter().circle_filled(
            //         pos2(
            //             center.x + (piece.position.x * 5.0 * (width / 1920.0)),
            //             center.y + (piece.position.y * 5.0 * (height / 1080.0)),
            //         ),
            //         60.0 * width / 1920.0,
            //         COLORS[i],
            //     );
            // }
            // for i in 0..puzzle.pieces.len() {
            //     let piece = &mut puzzle.pieces[i];
            //     ui.painter().circle_filled(
            //         pos2(
            //             center.x + (piece.position.x * 5.0 * (width / 1920.0)),
            //             center.y + (piece.position.y * 5.0 * (height / 1080.0)),
            //         ),
            //         50.0 * width / 1920.0,
            //         COLORS[i],
            //     );
            // }
            // if ui.add(egui::Button::new("LEFT CCW")).clicked() {
            //     puzzle.turn_id(0);
            // }
            // if ui.add(egui::Button::new("LEFT CW")).clicked() {
            //     puzzle.turn_id(1);
            // }
            // if ui.add(egui::Button::new("RIGHT CCW")).clicked() {
            //     puzzle.turn_id(2);
            // }
            // if ui.add(egui::Button::new("RIGHT CW")).clicked() {
            //     puzzle.turn_id(3);
            // }
            // if ui.add(egui::Button::new("UNDO")).clicked() {
            //     puzzle.undo();
            // }
            // if ui.add(egui::Button::new("GENERATE")).clicked() {}
            // if ui.add(egui::Button::new("SCRAMBLE")).clicked() {
            //     for i in 0..500 {
            //         puzzle.turn(*puzzle.turns.choose(&mut rng).unwrap());
            //     }
            //     puzzle.animation_offset = NONE_TURN;
            // }
            // ui.add(egui::Slider::new(&mut left_slider_value, 3..=50).text("Left Number"));
            // ui.add(egui::Slider::new(&mut right_slider_value, 3..=50).text("Right Number"));
            // puzzle.render(ui, &rect);
            // let max_rad = f32::max(puzzle.turns[0].circle.radius, puzzle.turns[2].circle.radius);
            // let cor_rect = Rect {
            //     min: to_egui_coords(
            //         &(puzzle.turns[0].circle.center + (max_rad * Vec2::from([-1.0, 1.0]))),
            //         &rect,
            //     ),
            //     max: to_egui_coords(
            //         &(puzzle.turns[2].circle.center + (max_rad * Vec2::from([1.0, -1.0]))),
            //         &rect,
            //     ),
            // };
            // if puzzle.animation_offset.angle != 0.0 {
            //     ui.ctx().request_repaint();
            // }
            // let r = ui.interact(cor_rect, egui::Id::new(19), egui::Sense::all());
            // if r.clicked() {
            //     puzzle.process_click(&rect, r.interact_pointer_pos().unwrap(), true);
            // }
            // if r.clicked_by(egui::PointerButton::Secondary) {
            //     puzzle.process_click(&rect, r.interact_pointer_pos().unwrap(), false);
            // }
            // if r.hover_pos().is_some() {
            //     let hovered_circle = puzzle.get_hovered(&rect, r.hover_pos().unwrap());
            //     if hovered_circle.radius > 0.0 {
            //         hovered_circle.draw(ui, &rect);
            //     }
            // }
        });
    })
}
