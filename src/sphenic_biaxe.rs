
use std::f32::consts::PI;

use rand::prelude::*;

use colorous;

use egui::{Color32, Pos2, Rect, Ui, Vec2, epaint::PathShape, pos2};

const DETAIL: u16 = 50;

const OUTLINE_COLOR: Color32 = Color32::BLACK;

const SPECTRUM: colorous::Gradient = colorous::TURBO;

const SCALE_FACTOR: f32 = 5.0;

const ANIMATION_SPEED: f32 = 5.0;

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
    shape: Vec<Pos2>,
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
struct Arc {
    start: Pos2,
    end: Pos2,
    circle: Circle,
    ccw: bool,
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
    fn draw(&self, ui: &mut Ui, rect: &Rect) {
        ui.painter().circle_stroke(
            good(&self.center, rect),
            self.radius * SCALE_FACTOR * (rect.width() / 1920.0),
            (9.0, Color32::WHITE),
        );
    }
    fn rotate_about(&self, center: Pos2, angle: f32) -> Circle {
        return Circle {
            center: rotate_about(center, self.center, angle),
            radius: self.radius,
        };
    }
}

impl Puzzle {
    fn turn(&mut self, turn: Turn) {
        let mut new_pieces = Vec::new();
        for mut piece in &self.pieces {
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
    fn render(&self, ui: &mut Ui, rect: &Rect) {
        for piece in &self.pieces {
            piece.render(ui, rect, self.animation_offset);
        }
    }
    fn process_click(&mut self, rect: &Rect, pos: Pos2, left: bool) {
        let good_pos = ungood(&pos, rect);
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
        let good_pos = ungood(&pos, rect);
        for turn in self.turns.clone() {
            if good_pos.distance(turn.circle.center) <= turn.circle.radius {
                return turn.circle;
            }
        }
        return NONE_CIRCLE;
    }
}

impl Piece {
    fn rotate_about(&mut self, center: Pos2, angle: f32) {
        let mut new_points: Vec<Pos2> = Vec::new();
        for mut point in self.shape.clone() {
            new_points.push(rotate_about(center, point, angle));
        }
        self.shape = new_points;
    }
    fn render(&self, ui: &mut Ui, rect: &Rect, offset: Turn) {
        let mut true_offset = NONE_TURN;
        if self.in_circle(&offset.circle) {
            true_offset = offset;
        }
        let mut render_points: Vec<Pos2> = Vec::new();
        for point in &self.shape {
            let new_point =
                rotate_about(true_offset.circle.center, point.clone(), true_offset.angle);
            render_points.push(good(&new_point, rect))
        }
        ui.painter().add(egui::Shape::convex_polygon(
            render_points,
            self.color,
            (2.0, OUTLINE_COLOR),
        ));
    }
    fn in_circle(&self, circle: &Circle) -> bool {
        for point in &self.shape {
            if point.distance(circle.center) > circle.radius + 0.1 {
                return false;
            }
        }
        return true;
    }
}

//translates from nice coords to egui coords
fn good(pos: &Pos2, rect: &Rect) -> Pos2 {
    return pos2(
        pos.x * (SCALE_FACTOR * rect.width() / 1920.0) + (rect.width() / 2.0),
        -1.0 * pos.y * (SCALE_FACTOR * rect.width() / 1920.0) + (rect.height() / 2.0),
    );
}

fn ungood(pos: &Pos2, rect: &Rect) -> Pos2 {
    return pos2(
        (pos.x - (rect.width() / 2.0)) * (1920.0 / (SCALE_FACTOR * rect.width())),
        (pos.y - (rect.height() / 2.0)) * (-1920.0 / (SCALE_FACTOR * rect.width())),
    );
}

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

fn get_sphenic_biaxe(
    detail: u16,
    left_center: Pos2,
    right_center: Pos2,
    number_l: u16,
    number_r: u16,
) -> Puzzle {
    let mut pieces: Vec<Piece> = Vec::new();
    let [circle1, circle2] =
        get_sphenic_biaxe_circles(left_center, right_center, number_l, number_r);
    let start_point_1 =
        circle1.center + (circle1.radius * (circle2.center - circle1.center).normalized());
    let start_point_2 =
        circle2.center + (circle2.radius * (circle1.center - circle2.center).normalized());
    let total_colors = number_l + number_r - 1;
    let colors_l = get_color_range(0, number_l, total_colors);
    let mut colors_r = vec![get_color(0, total_colors)];
    colors_r.extend(get_color_range(number_l, number_r - 1, total_colors));
    pieces.extend(get_sb_wedge_regions(
        detail,
        start_point_1,
        circle1,
        circle2,
        number_l,
        &colors_l,
    ));
    pieces.extend(get_sb_wedge_regions(
        detail,
        start_point_2,
        circle2,
        circle1,
        number_r,
        &colors_r,
    ));
    pieces.extend(get_sb_sphene_regions(
        detail, &circle1, &circle2, number_l, &colors_l,
    ));
    pieces.extend(get_sb_sphene_regions(
        detail, &circle2, &circle1, number_r, &colors_r,
    ));
    return Puzzle {
        pieces: pieces,
        turns: vec![
            Turn {
                circle: circle1.clone(),
                angle: 2.0 * PI / (number_l as f32),
            },
            Turn {
                circle: circle1.clone(),
                angle: -2.0 * PI / (number_l as f32),
            },
            Turn {
                circle: circle2.clone(),
                angle: 2.0 * PI / (number_r as f32),
            },
            Turn {
                circle: circle2.clone(),
                angle: -2.0 * PI / (number_r as f32),
            },
        ],
        stack: vec![],
        animation_offset: Turn {
            circle: circle1,
            angle: 0.0,
        },
    };
}

fn get_sb_wedge_regions(
    detail: u16,
    start_point: Pos2,
    start_circle: Circle,
    rotating_circle: Circle,
    divisions: u16,
    colors: &Vec<Color32>,
) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    for i in 0..divisions {
        let mut shape: Vec<Pos2> = Vec::new();
        let rotated_circle = rotating_circle.rotate_about(
            start_circle.center,
            ((i as f32) * 2.0 * PI) / (divisions as f32),
        );
        let point1 = rotate_about(
            start_circle.center,
            start_point,
            ((i as f32 - 0.5) * 2.0 * PI) / (divisions as f32),
        );
        let point4 = rotate_about(
            start_circle.center,
            start_point,
            ((i as f32 + 0.5) * 2.0 * PI) / (divisions as f32),
        );
        let intersection = circle_intersection(&rotated_circle, &start_circle);
        let point2 = intersection[1];
        let point3 = intersection[0];
        shape.extend(circle_region(detail, start_circle.center, point1, point2));
        shape.extend(
            circle_region(detail, rotated_circle.center, point3, point2)
                .into_iter()
                .rev(),
        );
        shape.extend(circle_region(detail, start_circle.center, point3, point4));
        shape.push(start_circle.center);
        pieces.push(Piece {
            shape: shape,
            color: colors[i as usize],
        });
    }
    return pieces;
}

fn get_sb_sphene_regions(
    detail: u16,
    start_circle: &Circle,
    rotated_circle: &Circle,
    divisions: u16,
    colors: &Vec<Color32>,
) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    for i in 0..divisions {
        pieces.push(Piece {
            shape: sphene_region(
                detail,
                &start_circle,
                &(rotated_circle.rotate_about(
                    start_circle.center,
                    (i as f32) * 2.0 * PI / (divisions as f32),
                )),
            ),
            color: colors[i as usize],
        })
    }
    return pieces;
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
    if circle1.center.distance(circle2.center) > circle1.radius + circle2.radius {
        return Vec::new();
    }
    if circle1.center.distance(circle2.center) == circle1.radius + circle2.radius {
        return vec![
            circle1.center + (circle1.radius * (circle2.center - circle1.center).normalized()),
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
    return vec![point1, point2];
}

fn sphene_region(detail: u16, circle1: &Circle, circle2: &Circle) -> Vec<Pos2> {
    let mut points: Vec<Pos2> = Vec::new();
    let intersection_points = circle_intersection(circle1, circle2);
    let point1 = intersection_points[0];
    let point2 = intersection_points[1];
    points.extend(circle_region(detail, circle1.center, point1, point2));
    points.extend(circle_region(detail, circle2.center, point2, point1));
    points.pop();
    return points;
}

fn wedge_region(detail: u16, point: Pos2, center: Pos2, angle: f32) -> Vec<Pos2> {
    let mut point = rotate_about(center, point, angle / -2.0);
    let mut points = vec![];
    points.push(point.clone());
    for i in 0..detail {
        point = rotate_about(center, point, angle / (detail as f32));
        points.push(point.clone());
    }
    points.push(center.clone());
    return points;
}

fn get_sphenic_biaxe_circles(
    center1: Pos2,
    center2: Pos2,
    number_l: u16,
    number_r: u16,
) -> [Circle; 2] {
    let number_l = number_l + 1;
    let number_r = number_r + 1;
    let angle_l = PI / f32::from(number_l);
    let angle_r = PI / f32::from(number_r);
    let angle = PI - (angle_l + angle_r);
    let ratio = center1.distance(center2) / angle.sin();
    return [
        Circle {
            center: center1,
            radius: ratio * angle_r.sin(),
        },
        Circle {
            center: center2,
            radius: ratio * angle_l.sin(),
        },
    ];
}

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
fn main() -> eframe::Result {
    let mut rng = rand::rng();
    let mut last_frame_time = std::time::Instant::now();
    let left_center = pos2(-50.0, 0.0);
    let right_center = pos2(50.0, 0.0);
    let mut left_slider_value: u16 = 5;
    let mut right_slider_value: u16 = 5;
    let mut number_l = left_slider_value;
    let mut number_r = right_slider_value;
    let mut puzzle = get_sphenic_biaxe(
        DETAIL,
        left_center,
        right_center,
        left_slider_value,
        right_slider_value,
    );
    let mut prev_states: Vec<Puzzle> = Vec::new();
    eframe::run_simple_native("circleguy", Default::default(), move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let delta_time = last_frame_time.elapsed();
            last_frame_time = std::time::Instant::now();
            if puzzle.animation_offset.angle >= 0.0 {
                puzzle.animation_offset.angle = f32::max(
                    puzzle.animation_offset.angle - (delta_time.as_secs_f32() * ANIMATION_SPEED),
                    0.0,
                );
            } else {
                puzzle.animation_offset.angle = f32::min(
                    puzzle.animation_offset.angle + (delta_time.as_secs_f32() * ANIMATION_SPEED),
                    0.0,
                );
            }

            let angle_l = (2.0 * PI) / f32::from(number_l);
            let angle_r = (2.0 * PI) / f32::from(number_r);

            let rect = ui.available_rect_before_wrap();
            let width = rect.width();
            let height = rect.height();
            let minimum = rect.size().min_elem();
            let center = pos2(rect.width() * 0.5, rect.height() * 0.5);

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
            if ui.add(egui::Button::new("UNDO")).clicked() {
                puzzle.undo();
            }
            if ui.add(egui::Button::new("GENERATE")).clicked() {
                puzzle = get_sphenic_biaxe(
                    DETAIL,
                    left_center,
                    right_center,
                    left_slider_value,
                    right_slider_value,
                );
                number_l = left_slider_value;
                number_r = right_slider_value;
            }
            if ui.add(egui::Button::new("SCRAMBLE")).clicked() {
                for i in 0..500 {
                    puzzle.turn(*puzzle.turns.choose(&mut rng).unwrap());
                }
                puzzle.animation_offset = NONE_TURN;
            }
            ui.add(egui::Slider::new(&mut left_slider_value, 3..=50).text("Left Number"));
            ui.add(egui::Slider::new(&mut right_slider_value, 3..=50).text("Right Number"));
            puzzle.render(ui, &rect);
            let max_rad = f32::max(puzzle.turns[0].circle.radius, puzzle.turns[2].circle.radius);
            let cor_rect = Rect {
                min: good(
                    &(puzzle.turns[0].circle.center + (max_rad * Vec2::from([-1.0, 1.0]))),
                    &rect,
                ),
                max: good(
                    &(puzzle.turns[2].circle.center + (max_rad * Vec2::from([1.0, -1.0]))),
                    &rect,
                ),
            };
            if puzzle.animation_offset.angle != 0.0 {
                ui.ctx().request_repaint();
            }
            let r = ui.interact(cor_rect, egui::Id::new(19), egui::Sense::all());
            if r.clicked() {
                puzzle.process_click(&rect, r.interact_pointer_pos().unwrap(), true);
            }
            if r.clicked_by(egui::PointerButton::Secondary) {
                puzzle.process_click(&rect, r.interact_pointer_pos().unwrap(), false);
            }
            if r.hover_pos().is_some() {
                let hovered_circle = puzzle.get_hovered(&rect, r.hover_pos().unwrap());
                if hovered_circle.radius > 0.0 {
                    hovered_circle.draw(ui, &rect);
                }
            }
        });
    })
}
