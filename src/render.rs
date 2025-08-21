use crate::PRECISION;
use crate::arc::*;
use crate::circle_utils::*;
use crate::piece::*;
use crate::puzzle::*;
use crate::turn::*;
use approx_collections::*;
use cga2d::*;
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
    epaint::{self, PathShape},
    pos2, vec2,
};
use std::cmp::*;
use std::f32::consts::PI;
fn aeq_pos(p1: Pos2, p2: Pos2) -> bool {
    p1.x.approx_eq(&p2.x, PRECISION) && p1.y.approx_eq(&p2.y, PRECISION)
}
const DETAIL: f64 = 0.5;

const OUTLINE_COLOR: Color32 = Color32::BLACK;

pub fn draw_circle(real_circle: Blade3, ui: &mut Ui, rect: &Rect, scale_factor: f32, offset: Vec2) {
    if let Circle::Circle {
        cx: x,
        cy: y,
        r,
        ori: _,
    } = real_circle.unpack()
    {
        ui.painter().circle_stroke(
            to_egui_coords(pos2(x as f32, y as f32), &rect, scale_factor, offset),
            r as f32 * scale_factor * (rect.width() / 1920.0),
            (10.0, Color32::WHITE),
        );
    }
}

const DETAIL_FACTOR: f64 = 3.0; // the amount more detailed the outlines are than the interiors
//take in a triangle and return if its 'almost degenerate' within some leniency (i.e. its points are 'almost colinear')
fn almost_degenerate(triangle: &Vec<Pos2>, leniency: f32) -> bool {
    let angle_1 = (triangle[1] - triangle[0]).angle() - (triangle[1] - triangle[2]).angle();
    let close = angle_1.abs().min((PI - angle_1).abs());
    if close < leniency {
        return true;
    }
    return false;
}

fn avg_points(points: &Vec<Pos2>) -> Pos2 {
    let n = points.len() as f32;
    let mut pos = pos2(0.0, 0.0);
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

//translates from nice coords to egui coords
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
fn from_egui_coords(pos: &Pos2, rect: &Rect, scale_factor: f32, offset: Vec2) -> Pos2 {
    return pos2(
        ((pos.x - (rect.width() / 2.0)) * (1920.0 / (scale_factor * rect.width()))) - offset.x,
        ((pos.y - (rect.height() / 2.0)) * (-1920.0 / (scale_factor * rect.width()))) - offset.y,
    );
}

//rotate a point about a point a certain angle
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
fn euc_center_rad(circ: Blade3) -> (Pos2, f32) {
    return match circ.unpack() {
        Circle::Circle { cx, cy, r, ori: _ } => (pos2(cx as f32, cy as f32), r as f32),
        _ => {
            dbg!(circ);
            panic!("you passed a line!")
        }
    };
}
impl Arc {
    fn draw(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        width: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) {
        let size =
            self.angle_euc().abs() as f32 * euc_center_rad(self.circle).1 * DETAIL_FACTOR as f32;
        let divisions = (size * detail * DETAIL as f32).max(2.0) as u16;
        let mut coords = Vec::new();
        for pos in self.get_polygon(divisions) {
            coords.push(to_egui_coords(
                //&rotate_about(offset.circle.center, pos, offset.angle),
                pos,
                rect,
                scale_factor,
                offset_pos,
            ));
        }
        ui.painter()
            .add(PathShape::line(coords, Stroke::new(width, OUTLINE_COLOR)));
    }
    //precondition: the boundary is not a tangent
    fn get_polygon(&self, divisions: u16) -> Vec<Pos2> {
        let mut points: Vec<Pos2> = Vec::new();
        let start_point = match self.boundary {
            None => euc_center_rad(self.circle).0 + vec2(0.0, euc_center_rad(self.circle).1),
            Some(b2) => {
                if let Dipole::Real(real) = b2.unpack()
                    && let Point::Finite([x, y]) = real[0]
                {
                    pos2(x as f32, y as f32)
                } else {
                    panic!("doorbell")
                }
            }
        };
        let angle = self.angle_euc() as f32;
        let inc_angle = angle / (divisions as f32);
        points.push(start_point);
        for i in 1..=divisions {
            points.push(rotate_about(
                euc_center_rad(self.circle).0,
                start_point,
                inc_angle * (i as f32),
            ));
        }
        return points;
    }
    fn triangulate(&self, center: Pos2, detail: f32) -> Vec<Vec<Pos2>> {
        let size = self.angle_euc().abs() as f32 * euc_center_rad(self.circle).1;
        // if aeq(self.angle_euc() as f64, 0.0) {
        //     dbg!(self.angle_euc());
        // }
        let div = (detail * size * DETAIL as f32).max(2.0) as u16;
        let polygon = self.get_polygon(div);
        let mut triangles = Vec::new();
        for i in 0..(polygon.len() - 1) {
            triangles.push(vec![center, polygon[i], polygon[i + 1]]);
        }
        triangles
    }
    //what it does: 'angle' returns the angle 'between' the lines going through the euclidian center of self.circle and the points of the boundary. this angle is between
    //-PI and PI and the sign is inverted based on the orientation of both self.circle and self.boundary. this answer is then taken mod 2PI, yielding a positive answer between
    //0 and 2PI. if the circle is clockwise (negative orientation), then the sign of the final answer is inverted and then returned
    fn angle_euc(&self) -> f32 {
        let orientation = circle_orientation_euclid(self.circle) == Contains::Inside;
        if self.boundary == None {
            return if orientation { 2.0 * PI } else { -2.0 * PI };
        } else {
            let Dipole::Real([p1, p2]) = self.boundary.unwrap().unpack() else {
                return 0.0;
            };
            let (pos1, pos2) = match (p1, p2) {
                (Point::Finite([x1, y1]), Point::Finite([x2, y2])) => {
                    (pos2(x1 as f32, y1 as f32), pos2(x2 as f32, y2 as f32))
                }
                _ => panic!("the boundary isnt real"),
            };
            let center = euc_center_rad(self.circle).0;
            let angle = ((pos2 - center).angle() - (pos1 - center).angle()).rem_euclid(2.0 * PI);
            if orientation {
                angle
            } else {
                angle - (2.0 * PI)
            }
        }
        // } else {
        //     let [a, b] = match self.boundary.unwrap().unpack() {
        //         Dipole::Real(k) => k,
        //         _ => panic!("boundary is not real!"),
        //     };
        //     let cp = !self.circle ^ NI;
        //     let (la, lb) = ((cp ^ a).normalize(), (cp ^ b).normalize());
        //     let lp = -(!la ^ cp).normalize();
        //     //lp ~= !la ^ !self.circle ^ NI, imagine the zodiac sign its that
        //     let (a1, a2) = (la << lb, lp << lb);
        //     let angle = f64::atan2(a2, a1);
        //     let pos_angle = angle.rem_euclid(2.0 * PI as f64);
        //     return if !orientation { -pos_angle } else { pos_angle };
        // }
    }
    fn midpoint_euc(&self) -> Option<Pos2> {
        let p = match self.boundary?.unpack() {
            Dipole::Real(real) => match real[0] {
                Point::Finite([x, y]) => pos2(x as f32, y as f32),
                _ => panic!("point is infinite!"),
            },
            _ => panic!("ABSOLUTELY NOT!"),
        };
        Some(rotate_about(
            euc_center_rad(self.circle).0,
            p,
            self.angle_euc() as f32 / 2.0,
        ))
    }
}
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
    ) {
        let true_offset = if offset.is_none()
            || self
                .in_circle(offset.unwrap().circle)
                .is_some_and(|x| x == Contains::Inside)
        {
            offset
        } else {
            None
        };
        let true_piece = if let Some(twist) = true_offset {
            self.turn(twist).unwrap_or(self.clone())
        } else {
            self.clone()
        };
        let triangulation = true_piece.triangulate(true_piece.barycenter(), detail);
        let mut triangle_vertices: Vec<epaint::Vertex> = Vec::new();
        for triangle in triangulation {
            if !almost_degenerate(&triangle, 0.0) {
                for point in triangle {
                    let vertex = epaint::Vertex {
                        pos: to_egui_coords(point, rect, scale_factor, offset_pos),
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
        true_piece.draw_outline(ui, rect, detail, outline_size, scale_factor, offset_pos);
    }
    // returns a list of triangles for rendering
    fn triangulate(&self, center: Pos2, detail: f32) -> Vec<Vec<Pos2>> {
        let mut triangles = Vec::new();
        for arc in &self.shape.border {
            triangles.extend(arc.triangulate(center, detail));
        }
        return triangles;
    }
    fn barycenter(&self) -> Pos2 {
        let mut points = Vec::new();
        for arc in &self.shape.border {
            if let Some(x) = arc.midpoint_euc() {
                points.push(x);
            };
        }
        if points.is_empty() {
            return euc_center_rad(self.shape.border[0].circle).0;
        }
        return avg_points(&points);
    }
    fn draw_outline(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) {
        for arc in &self.shape.border {
            arc.draw(ui, rect, detail, outline_size, scale_factor, offset_pos);
        }
    }
}
impl Puzzle {
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        outline_width: f32,
        scale_factor: f32,
        offset: Vec2,
    ) {
        let proper_offset = if let Some(off) = self.animation_offset {
            Some(Turn {
                circle: off.circle,
                rotation: self.anim_left as f64 * off.rotation
                    + (1.0 - self.anim_left) as f64 * Rotoflector::ident(),
            })
        } else {
            None
        };
        for piece in &self.pieces {
            piece.render(
                ui,
                rect,
                proper_offset,
                detail,
                outline_width,
                scale_factor,
                offset,
            );
        }
    }
    pub fn process_click(
        &mut self,
        rect: &Rect,
        pos: Pos2,
        left: bool,
        scale_factor: f32,
        offset: Vec2,
        cut: bool,
    ) -> Result<(), bool> {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset);
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_id: String = String::from("");
        for turn in &self.turns {
            let (center, radius) = match turn.1.circle.unpack() {
                Circle::Circle { cx, cy, r, ori: _ } => (pos2(cx as f32, cy as f32), r as f32),
                _ => panic!("not a circle lol!"),
            };
            if ((good_pos.distance(center).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.distance(center).approx_eq(&min_dist, PRECISION))
                    && (radius.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && circle_contains(turn.1.circle, point(good_pos.x as f64, good_pos.y as f64))
                    == Contains::Inside
                && !turn.0.ends_with("'")
            {
                min_dist = good_pos.distance(center);
                min_rad = radius;
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
    pub fn get_hovered(
        &self,
        rect: &Rect,
        pos: Pos2,
        scale_factor: f32,
        offset: Vec2,
    ) -> Option<Blade3> {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset);
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_turn = None;
        for turn in self.turns.clone().values() {
            let (cent, rad) = euc_center_rad(turn.circle);
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
            return None;
        }
        //dbg!(correct_turn.circle.center.to_pos2());
        return Some(correct_turn?.circle);
    }
}
