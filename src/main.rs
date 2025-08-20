//#![windows_subsystem = "windows"]
use std::{
    cmp::Ordering,
    collections::HashMap,
    f32::consts::PI,
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    ops::Bound,
    vec,
};

use kdl::KdlDocument;
use rand::prelude::*;

use egui::{
    Color32, Event, MouseWheelUnit, Pos2, Rect, ScrollArea, Stroke, Ui, Vec2,
    epaint::{self, PathShape},
    pos2, vec2,
};

use cga2d::*;

type Cut = Vec<Turn>;
type Coloring = (BoundingCircles, Color32);

#[cfg(not(target_arch = "wasm32"))]
const DEV: bool = true;

const DETAIL: f64 = 0.5;

const OUTLINE_COLOR: Color32 = Color32::BLACK;

// const SPECTRUM: colorous::Gradient = colorous::TURBO;

const SCALE_FACTOR: f32 = 200.0;

const ANIMATION_SPEED: f64 = 5.0;

const LENIENCY: f64 = 0.0002;

const NONE_COLOR: Color32 = Color32::GRAY;

const NONE_TURN: Turn = Turn {
    circle: Blade3 {
        mpx: 0.0,
        mpy: 0.0,
        mxy: 0.0,
        pxy: 0.0,
    },
    rotation: Rotoflector::Rotor(Rotor {
        s: 1.0,
        mp: 0.0,
        mx: 0.0,
        px: 0.0,
        my: 0.0,
        py: 0.0,
        xy: 0.0,
        mpxy: 0.0,
    }),
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
        rotation: rotor_pow(a, b.rotation),
    }
});

fn rotor_pow(i: isize, rot: Rotoflector) -> Rotoflector {
    if i == 0 {
        return Rotoflector::ident();
    }
    if i > 0 {
        return rotor_pow(i - 1, rot) * rot;
    } else {
        return rotor_pow(-i, rot).rev();
    }
}
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

//arbitrarily decided - ~0.0 is border, > 0.0 is inside, < 0.0 is outside

fn contains_from_metric(metric: f64) -> Contains {
    if alneq(metric, 0.0) {
        return Contains::Outside;
    }
    if alneq(0.0, metric) {
        return Contains::Inside;
    }
    return Contains::Border;
}

type BoundingCircles = Vec<Blade3>;

type BoundaryShape = Vec<PieceArc>;

//if boundary is None, then the arc is the whole circle
#[derive(Clone, Copy, Debug)]
struct PieceArc {
    circle: Blade3,
    boundary: Option<Blade2>,
}
#[derive(Clone)]
struct PieceShape {
    bounds: BoundingCircles,
    border: BoundaryShape,
}

#[derive(Clone)]
struct Piece {
    shape: PieceShape,
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
    anim_left: f32,
}

struct PuzzlePrevData {
    name: String,
    turns: Vec<String>,
}
#[derive(Clone, Copy)]
struct Turn {
    circle: Blade3,
    rotation: Rotoflector,
}

//orientation: true means that the 'inside' of the arc is the 'inside' of the piece, false means the opposite
//checking if certain float-based variable types are approximately equal due to precision bs

fn aeq(f1: f64, f2: f64) -> bool {
    return f1.approx_eq(&f2, Precision::new_simple(20));
}
fn aleq(f1: f64, f2: f64) -> bool {
    return f1 < f2 || aeq(f1, f2);
}

fn alneq(f1: f64, f2: f64) -> bool {
    return aleq(f1, f2) && !aeq(f1, f2);
}

fn aeq_pos(p1: Pos2, p2: Pos2) -> bool {
    return aeq(p1.x as f64, p2.x as f64) && aeq(p1.y as f64, p2.y as f64);
}

// fn aeq_circ(c1: Circle, c2: Circle) -> bool {
//     return aeq_pos(c1.center, c2.center) && aeq(c1.radius, c2.radius);
// }

// fn aeq_shape(s1: &PieceShape, s2: &PieceShape) -> bool {
//     for circ in &s1.bounds {
//         let mut same = false;
//         for circ2 in &s2.bounds {
//             if aeq_circle(*circ, *circ2)
//                 && circle_orientation_euclid(*circ) == circle_orientation_euclid(*circ2)
//             {
//                 same = true;
//                 break;
//             }
//         }
//         if !same {
//             return false;
//         }
//     }
//     return true;
// }

fn aeq_piece(p1: &Piece, p2: &Piece) -> bool {
    // return p1.color.r() == p2.color.r()
    //     && p1.color.g() == p2.color.g()
    //     && p1.color.b() == p2.color.b()
    //     && aeq_shape(&p1.shape, &p2.shape);
    false
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

fn euc_center_rad(circ: Blade3) -> (Pos2, f32) {
    return match circ.unpack() {
        Circle::Circle { cx, cy, r, ori } => (pos2(cx as f32, cy as f32), r as f32),
        _ => {
            dbg!(circ);
            panic!("you passed a line!")
        }
    };
}

impl Turn {
    fn inverse(&self) -> Turn {
        Turn {
            circle: self.circle,
            rotation: self.rotation.rev(),
        }
    }
}

impl FloatIntern {
    fn intern_blade3(&mut self, b: &mut Blade3) {
        for mut t in b.terms() {
            self.intern(&mut t.coef);
        }
    }
    fn intern_blade2(&mut self, b: &mut Blade2) {
        for mut t in b.terms() {
            self.intern(&mut t.coef);
        }
    }
    fn intern(&mut self, f: &mut f64) {
        for float in &self.floats {
            if (*float - *f).abs() < LENIENCY {
                *f = *float;
                return;
            }
        }
        self.floats.push(*f);
    }
}

impl Puzzle {
    fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                self.intern.intern_blade3(&mut arc.circle);
                if let Some(bound) = arc.boundary.as_mut() {
                    self.intern.intern_blade2(bound);
                }
            }
            for circ in &mut piece.shape.bounds {
                self.intern.intern_blade3(circ);
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
            new_pieces.push(piece.turn(turn).ok_or(true)?);
        }
        self.pieces = new_pieces;
        self.anim_left = 1.0;
        self.animation_offset = turn.inverse();
        //self.intern_all();
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
            self.turn(*turn, true).or(Err(()))?;
        }
        for turn in cut.clone().into_iter().rev() {
            self.turn(turn.inverse(), false).or(Err(()))?;
        }
        Ok(())
    }
    fn color(&mut self, coloring: &Coloring) {
        for piece in &mut self.pieces {
            let mut color = true;
            for circle in &coloring.0 {
                let contains = piece.in_circle(*circle);
                if contains != Some(Contains::Inside) {
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
        detail: f32,
        outline_width: f32,
        scale_factor: f32,
        offset: Vec2,
    ) {
        let proper_offset = Turn {
            circle: self.animation_offset.circle,
            rotation: self.anim_left as f64 * self.animation_offset.rotation
                + (1.0 - self.anim_left) as f64 * Rotoflector::ident(),
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
    fn process_click(
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
                Circle::Circle { cx, cy, r, ori } => (pos2(cx as f32, cy as f32), r as f32),
                _ => panic!("not a circle lol!"),
            };
            if (alneq(good_pos.distance(center) as f64, min_dist as f64)
                || (aeq(good_pos.distance(center) as f64, min_dist as f64)
                    && alneq(radius as f64, min_rad as f64)))
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
    fn get_hovered(
        &self,
        rect: &Rect,
        pos: Pos2,
        scale_factor: f32,
        offset: Vec2,
    ) -> Option<Blade3> {
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset);
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_turn = NONE_TURN;
        for turn in self.turns.clone().values() {
            let (cent, rad) = euc_center_rad(turn.circle);
            if (alneq(good_pos.distance(cent) as f64, min_dist as f64)
                || (aeq(good_pos.distance(cent) as f64, min_dist as f64)
                    && alneq(rad as f64, min_rad as f64)))
                && good_pos.distance(cent) < rad
            {
                min_dist = good_pos.distance(cent);
                min_rad = rad;
                correct_turn = *turn;
            }
        }
        if min_rad == 10000.0 {
            return None;
        }
        //dbg!(correct_turn.circle.center.to_pos2());
        return Some(correct_turn.circle);
    }
    fn global_cut_by_circle(&mut self, circle: Blade3) -> Result<(), ()> {
        let mut new_pieces = Vec::new();
        for piece in &self.pieces {
            //dbg!(piece.shape.border.len());
            match piece.cut_by_circle(circle) {
                None => new_pieces.push(piece.clone()),
                Some(x) => new_pieces.extend(x),
            }
        }
        self.pieces = new_pieces;
        //self.intern_all();
        Ok(())
    }
}

//CGA NEED TO TEST SIGN
impl PieceArc {
    fn contains(&self, point: Blade1) -> Option<Contains> {
        if circle_contains(self.circle, point) != Contains::Border {
            return None;
        }
        if self.boundary == None {
            return Some(Contains::Inside);
        }
        Some(contains_from_metric(
            (-(self.boundary.unwrap() ^ point) << self.circle),
        ))
    }
    fn intersect_circle(&self, circle: Blade3) -> [Option<Blade1>; 2] {
        let Dipole::Real(int_points) = (self.circle & circle).unpack() else {
            return [None; 2];
        };
        return int_points.map(|a| match self.contains(a.into()) {
            None => None,
            Some(Contains::Outside) => None,
            Some(Contains::Border) | Some(Contains::Inside) => Some(a.into()),
        });
    }

    //result[0] inside
    //dont pass aeq circles
    //please, i beg of you, dont do it
    //dont you dare
    //if you pass aeq circles i will hunt you down
    //im not joking
    //will sort if passed an arc that doesnt intersect the circle
    fn cut_by_circle(&self, circle: Blade3) -> [Vec<PieceArc>; 2] {
        let mut sorted_arcs = [Vec::new(), Vec::new()];
        let mut segments = Vec::new();
        let mut new_points = Vec::new();
        match (circle & self.circle).unpack() {
            Dipole::Real(intersects) => {
                for intersect in intersects {
                    if self.contains(intersect.into()).unwrap() == Contains::Inside {
                        new_points.push(intersect.into());
                    }
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
                        segments.push(PieceArc {
                            circle: self.circle,
                            boundary: Some((new_points[i] ^ new_points[i + 1])),
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
                None => panic!("whats going on? who are you?"),
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
    fn inverse(&self) -> PieceArc {
        return PieceArc {
            circle: -self.circle,
            boundary: match self.boundary {
                None => None,
                Some(x) => Some(-x),
            },
        };
    }
    //helper for in_circle
    fn contains_either_properly(&self, pair: Blade2) -> bool {
        let points = match pair.unpack() {
            Dipole::Real(real) => real,
            _ => panic!("492830948234"),
        };
        for p in points {
            if self.contains(p.into()) == Some(Contains::Inside) {
                return true;
            }
        }
        (false)
    }
    fn rotate(&self, rot: Rotoflector) -> PieceArc {
        PieceArc {
            boundary: match self.boundary {
                None => None,
                Some(x) => Some(rot.sandwich(x)),
            },
            circle: rot.sandwich(self.circle),
        }
    }
    //None -- the arc crosses the circles boundary
    //Border -- the arc is on the circle
    //Inside/Outside -- arc endpoints can be on the boundary
    //potential useful precondition -- the arc does not cross the boundary, only touches it. should be sufficient for cutting, however not sufficient for bandaging reasons
    fn in_circle(&self, circle: Blade3) -> Option<Contains> {
        if circle
            == (Blade3 {
                mxy: 0.0,
                mpx: 0.0,
                mpy: 0.0,
                pxy: 0.0,
            })
        {
            return Some(Contains::Outside);
        }
        let arc_circle = self.circle;
        let circ = circle;
        if (circ.approx_eq(&arc_circle, Precision::new_simple(20)))
            || (circ.approx_eq(&-arc_circle, Precision::new_simple(20)))
        {
            return Some(Contains::Border);
        }
        let intersect = circ & arc_circle;
        match intersect.unpack() {
            Dipole::Real(real) => {
                if self.contains_either_properly(intersect) {
                    return None;
                }
                //FLIP SIGN MAYBE
                let bound_points = match self.boundary?.unpack() {
                    Dipole::Real(r) => r,
                    _ => {
                        dbg!(self.boundary.unwrap().mag2());
                        dbg!(self.boundary);
                        dbg!(self.boundary.unwrap().unpack());
                        panic!("schlimble")
                    }
                };
                let contains = [
                    circle_contains(circ, bound_points[0].into()),
                    circle_contains(circ, bound_points[1].into()),
                ];
                return match contains {
                    [Contains::Inside, Contains::Inside]
                    | [Contains::Inside, Contains::Border]
                    | [Contains::Border, Contains::Inside] => Some(Contains::Inside),
                    [Contains::Outside, Contains::Outside]
                    | [Contains::Outside, Contains::Border]
                    | [Contains::Border, Contains::Outside] => Some(Contains::Outside),
                    [Contains::Border, Contains::Border] => Some(
                        //SIGN NEEDS CHECKING
                        match real[0].approx_eq(
                            &match self.boundary?.unpack() {
                                Dipole::Real(real_boundary) => real_boundary[0],
                                _ => panic!("terrorism"),
                            },
                            Precision::new_simple(20),
                        ) {
                            false => Contains::Outside,
                            true => Contains::Inside,
                        },
                    ),
                    _ => {
                        dbg!(self);
                        dbg!(circ);
                        dbg!(
                            dbg!(
                                -(self.boundary.unwrap() ^ Into::<Blade1>::into(real[0]))
                                    << self.circle
                            )
                            .approx_eq(&0.0, Precision::new_simple(20))
                        );
                        dbg!(3.2195042811735317e-5.approx_eq(&0.0, Precision::new_simple(20)));
                        panic!("what have you done.")
                    }
                };
            }
            _ => Some(circ_border_inside_circ(circ, arc_circle)),
        }
    }
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
        if aeq(self.angle_euc() as f64, 0.0) {
            dbg!(self.angle_euc());
        }
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

//checks if C2 border is inside C1
fn circ_border_inside_circ(c1: Blade3, c2: Blade3) -> Contains {
    for point in [NI, NO] {
        let val = !(point ^ (c1 & c2) ^ !c2);
        if contains_from_metric(val) != Contains::Border {
            return contains_from_metric(val);
        }
    }
    return Contains::Border;
}

//will panic if passed a line
fn basic_turn(raw_circle: Blade3, angle: f64) -> Turn {
    if let Circle::Circle { cx, cy, r: _, ori } = raw_circle.unpack() {
        let p1 = point(cx, cy);
        let p2 = point(cx + 1.0, cy);
        let p3 = point(cx + (angle / 2.0).cos(), cy + (angle / 2.0).sin());
        return Turn {
            circle: raw_circle,
            rotation: Rotoflector::Rotor((p1 ^ p3 ^ NI) * (p1 ^ p2 ^ NI)),
        };
    } else {
        panic!("you passed a line!");
    }
}

//Blade3 Helpers
fn circle_contains(circ: Blade3, point: Blade1) -> Contains {
    contains_from_metric(!(circ ^ point))
}
fn blade2_almost_null(blade: Blade2) -> bool {
    for i in blade.terms() {
        if !aeq(i.coef, 0.0) {
            return false;
        }
    }
    true
}
//NOT FINISHED
//CHECK UP TO SCALING?
// fn aeq_circle(c1: Blade3, c2: Blade3) -> bool {
//     return blade2_almost_null(!(c1.normalize()) ^ !(c2.normalize()));
// }

// fn aeq_point(p1: Blade1, p2: Blade1) -> bool {
//     return blade2_almost_null(p1 ^ p2);
// }

//circles created from the circle() fn are counterclockwise and Contains::Inside
fn circle_orientation_euclid(circ: Blade3) -> Contains {
    circle_contains(-circ, NI)
}
//None - the boundary was not cut
//Some(1, 2) 1 is inside the circle, 2 is outside

//TO FIX: DONT PASS ARC_START OR ARC_END INTO CUT_ARC, OR MAKE CUT_ARC FIX THIS
fn cut_boundary(bound: &BoundaryShape, circle: Blade3) -> Option<[BoundaryShape; 2]> {
    let mut starts = Vec::new();
    let mut ends = Vec::new();
    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for i in 0..bound.len() {
        let arc = bound[i];
        if (arc.circle.approx_eq(&circle, Precision::new_simple(20))
            || arc.circle.approx_eq(&-circle, Precision::new_simple(20)))
        {
            return None;
        }
        let mut cut_points = Vec::new();
        let int = arc.intersect_circle(circle);
        if int[0].is_some()
            && !(arc.boundary.is_some()
                && arc
                    .boundary
                    .unwrap()
                    .mag2()
                    .approx_sign(Precision::new_simple(20))
                    != Sign::Zero
                && int[0].unwrap().approx_eq(
                    &match arc.boundary.unwrap().unpack() {
                        Dipole::Real(real) => real[0].into(),
                        _ => panic!("JIM???? I HAVENT SEEN YOU IN YEARS!"),
                    },
                    Precision::new_simple(20),
                )
                && (next_arc(&bound, arc).unwrap().circle & circle)
                    .mag2()
                    .approx_sign(Precision::new_simple(20))
                    == Sign::Positive)
        {
            starts.push(int[0].unwrap());
            cut_points.push(int[0].unwrap());
        }
        if int[1].is_some()
            && !(arc.boundary.is_some()
                && arc
                    .boundary
                    .unwrap()
                    .mag2()
                    .approx_sign(Precision::new_simple(20))
                    != Sign::Zero
                && int[1].unwrap().approx_eq(
                    &match arc.boundary.unwrap().unpack() {
                        Dipole::Real(real) => real[1].into(),
                        _ => panic!("JIM???? I HAVENT SEEN YOU IN YEARS!"),
                    },
                    Precision::new_simple(20),
                )
                && (next_arc(&bound, arc).unwrap().circle & circle)
                    .mag2()
                    .approx_sign(Precision::new_simple(20))
                    == Sign::Positive)
        {
            ends.push(int[1].unwrap());
            cut_points.push(int[1].unwrap());
        }
        let [add_inside, add_outside] = arc.cut_by_circle(circle);
        inside.extend(add_inside);
        outside.extend(add_outside);
    }
    if (inside.is_empty()) || (outside.is_empty()) {
        return None;
    }
    for arc in inner_circle_arcs(starts, ends, circle) {
        inside.push(arc);
        outside.push(arc.inverse());
    }
    return Some([inside, outside]);
}

//return the collapsed CCP representation
fn collapse_shape_and_add(
    bounding_circles: &BoundingCircles,
    new_circle: Blade3,
) -> BoundingCircles {
    let mut new_bounding_circles = Vec::new();
    for circ in bounding_circles {
        if !circle_excludes(*circ, new_circle) {
            new_bounding_circles.push(*circ);
        }
    }
    new_bounding_circles.push(new_circle);
    new_bounding_circles
}

impl PieceShape {
    fn cut_by_circle(&self, circle: Blade3) -> Option<[PieceShape; 2]> {
        let shapes = cut_boundary(&self.border, circle)?;
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
    fn in_circle(&self, circle: Blade3) -> Option<Contains> {
        let mut inside = None;
        for arc in &self.border {
            let contained = arc.in_circle(circle)?;
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
    fn turn(&self, turn: Turn) -> Option<PieceShape> {
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
}

impl Piece {
    fn turn(&self, turn: Turn) -> Option<Piece> {
        return Some(Piece {
            shape: self.shape.turn(turn)?,
            color: self.color,
        });
    }
    fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        offset: Turn,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) {
        let true_offset = if self
            .in_circle(offset.circle)
            .is_some_and(|x| x == Contains::Inside)
        {
            offset
        } else {
            Turn {
                circle: offset.circle,
                rotation: Rotoflector::ident(),
            }
        };
        let true_piece = self.turn(true_offset).unwrap_or(self.clone());
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
    //None: the piece is inside and outside -- blocking
    fn in_circle(&self, circle: Blade3) -> Option<Contains> {
        let mut inside = None;
        for arc in &self.shape.border {
            let contained = arc.in_circle(circle)?;
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
    //return if the shape contains the point properly -- not on the border
    //should return false if the point is properly outside the piece and true if it is properly inside the piece -- behavior on border points is undefined, may panic or return either option.
    //essentially, check how many 'valid' points that are on the border of self and directly left (within leniency) of point, and then take that mod 2 to get the answer
    // fn contains_point(&self, point: Pos2F64) -> bool {
    //     let y = point.y;
    //     let mut intersects = 0;
    //     for i in 0..self.shape.len() {
    //         let arc = self.shape[i];
    //         let prev_arc = self.shape[match i {
    //             0 => self.shape.len() - 1,
    //             _ => i - 1,
    //         }];
    //         let points = arc.arc_points_directly_left(point); //get the points on the current arc directly left of the point
    //         for int_point in points {
    //             if aeq_pos(arc.start, int_point) {
    //                 if ((arc.initially_above_y(y)) != (prev_arc.invert().initially_above_y(y))) //tangent case -- basically we only add in this case if the arcs actually cross the y line
    //                     || aeq(arc.angle, 2.0 * PI as f64)
    //                 {
    //                     intersects += 1;
    //                 }
    //             } else {
    //                 if !aeq((arc.circle.center.y - y).abs(), arc.circle.radius) {
    //                     //throw out the case where the arc is tangent at this point to the y line
    //                     intersects += 1;
    //                 }
    //             }
    //         }
    //     }
    //     //dbg!(intersects);
    //     return (intersects % 2) != 0; //the point is inside the piece if the number of valid intersects is odd, and outside otherwise
    // }
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
    //[inside, outside]
    fn cut_by_circle(&self, circle: Blade3) -> Option<[Piece; 2]> {
        let shapes = self.shape.cut_by_circle(circle)?;
        Some([
            Piece {
                shape: shapes[0].clone(),
                color: self.color,
            },
            Piece {
                shape: shapes[1].clone(),
                color: self.color,
            },
        ])
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
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) {
        for arc in &self.shape.border {
            arc.draw(ui, rect, detail, outline_size, scale_factor, offset_pos);
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
    #[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(target_arch = "wasm32")]
    fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        self.data = Vec::new();
        static PUZZLE_DEFINITIONS: include_dir::Dir<'_> =
            include_dir::include_dir!("$CARGO_MANIFEST_DIR/Puzzles/Definitions");
        let mut paths = Vec::new();
        for file in PUZZLE_DEFINITIONS.files() {
            paths.push(file.path().to_str().unwrap());
        }
        for path in paths {
            let data = read_file_to_string(&(String::from(def_path) + &path))
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

//CGA MAYBE NEEDS TESTING, I THINK I GOT IT?
//undefined when A aeq B
//point aeq to base is minimal
fn comp_points_on_circle(base: Blade1, a: Blade1, b: Blade1, circ: Blade3) -> Ordering {
    if a.approx_eq(&base, Precision::new_simple(20)) {
        return Ordering::Less;
    }
    if b.approx_eq(&base, Precision::new_simple(20)) {
        return Ordering::Greater;
    }
    cmp_f64(((base ^ b) ^ a) << circ, 0.0)
}

//CGA NEEDS TESTING
//returns if c1 fully excludes c2 (if the inside of c2 is fully contained in c1)
//WRONG!!! straight up doesnt work lol the math is bad
fn circle_excludes(c1: Blade3, c2: Blade3) -> bool {
    circ_border_inside_circ(c1, c2) == Contains::Inside
        && circle_orientation_euclid(c1) == circle_orientation_euclid(c2)
}

//CGA NEEDS TESTING
//gives the 'inside of the circle' arcs, ideally
fn inner_circle_arcs(
    mut starts: Vec<Blade1>,
    mut ends: Vec<Blade1>,
    circ: Blade3,
) -> Vec<PieceArc> {
    if (starts.len()) != (ends.len()) {
        panic!("inequal number of starts and ends passed");
    }
    if starts.is_empty() {
        return Vec::new();
    }
    let mut arcs = Vec::new();
    ends.sort_by(|a, b| comp_points_on_circle(starts[0], *a, *b, circ));
    starts.sort_by(|a, b| comp_points_on_circle(*ends.last().unwrap(), *a, *b, circ));
    for i in 0..starts.len() {
        if starts[i].approx_eq(&ends[i], Precision::new_simple(20)) {
            continue;
        } else {
            arcs.push(PieceArc {
                circle: circ,
                boundary: Some((starts[i] ^ ends[i])),
            });
        }
    }
    return arcs;
}

fn next_arc(bound: &BoundaryShape, curr: PieceArc) -> Option<PieceArc> {
    for arc in bound {
        if let Some(boundary) = arc.boundary
            && let Dipole::Real(real) = boundary.unpack()
            && let Dipole::Real(real_curr) = curr.boundary?.unpack()
            && (real_curr[1].approx_eq(&real[0], Precision::new_simple(20)))
        {
            return Some(*arc);
        }
    }
    None
}

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
// fn circle_intersection(circle1: Circle, circle2: Circle) -> Option<Vec<Pos2F64>> {
//     if aeq_circ(circle1, circle2) {
//         return None;
//     }
//     if alneq(
//         circle1.radius + circle2.radius,
//         circle1.center.distance(circle2.center),
//     ) || alneq(
//         circle1.center.distance(circle2.center) + circle2.radius,
//         circle1.radius,
//     ) || alneq(
//         circle1.center.distance(circle2.center) + circle1.radius,
//         circle2.radius,
//     ) {
//         return Some(Vec::new());
//     }
//     if aeq(
//         circle1.center.distance(circle2.center),
//         circle1.radius + circle2.radius,
//     ) || aeq(
//         circle1.center.distance(circle2.center) + circle2.radius,
//         circle1.radius,
//     ) {
//         return Some(vec![
//             (circle1.center + (circle1.radius * (circle2.center - circle1.center).normalized()?)),
//         ]);
//     }
//     if aeq(
//         circle1.center.distance(circle2.center) + circle1.radius,
//         circle2.radius,
//     ) {
//         return Some(vec![
//             (circle2.center + (circle2.radius * (circle1.center - circle2.center).normalized()?)),
//         ]);
//     }
//     let dist_sq = circle1.center.distance_sq(circle2.center);
//     let angle = ((dist_sq + (circle1.radius * circle1.radius) - (circle2.radius * circle2.radius))
//         / (2.0 * circle1.radius * circle1.center.distance(circle2.center)))
//     .acos();
//     let difference = circle2.center - circle1.center;
//     let unit_difference = difference.normalized()?;
//     let arc_point = circle1.center + (circle1.radius * unit_difference);
//     let point1 = rotate_about(circle1.center, arc_point, -1.0 * angle);
//     let point2 = rotate_about(circle1.center, arc_point, angle);
//     return Some(vec![point1, point2]);
// }

// fn collapse_shape(shape: &Vec<Arc>) -> Option<Vec<Arc>> {
//     let mut new_shape: Vec<Arc> = vec![shape[0]];
//     for i in 1..shape.len() {
//         let arc = shape[i];
//         if aeq_circ(new_shape.last()?.circle, arc.circle) {
//             new_shape.last_mut()?.angle += arc.angle;
//         } else {
//             new_shape.push(arc);
//         }
//     }
//     if aeq_circ(new_shape[0].circle, new_shape.last()?.circle) {
//         new_shape.last_mut()?.angle += new_shape[0].angle;
//         new_shape.remove(0);
//     }
//     return Some(new_shape);
// }

fn make_basic_puzzle(disks: Vec<Blade3>) -> Result<Vec<Piece>, ()> {
    let mut pieces = Vec::new();
    let mut old_disks = Vec::new();
    for disk in &disks {
        let mut disk_piece = Piece {
            shape: PieceShape {
                bounds: vec![*disk],
                border: vec![PieceArc {
                    circle: *disk,
                    boundary: None,
                }],
            },
            color: NONE_COLOR,
        };
        for old_disk in &old_disks {
            disk_piece = match disk_piece.cut_by_circle(*old_disk) {
                None => disk_piece.clone(),
                Some(x) => x[1].clone(),
            };
        }
        old_disks.push(*disk);
        pieces.push(disk_piece);
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
fn oriented_circle(cent: Blade1, rad: f64, inside: bool) -> Blade3 {
    let circ = circle(cent, rad);
    return if (circle_orientation_euclid(circ) == Contains::Outside) == inside {
        -circ
    } else {
        circ
    };
}

fn parse_kdl(string: &str) -> Option<Puzzle> {
    let mut puzzle = Puzzle {
        name: String::new(),
        authors: Vec::new(),
        pieces: Vec::new(),
        turns: HashMap::new(),
        stack: Vec::new(),
        animation_offset: NONE_TURN,
        intern: FloatIntern { floats: Vec::new() },
        depth: 500,
        solved_state: Vec::new(),
        solved: true,
        anim_left: 0.0,
    };
    let mut def_stack = Vec::new();
    let doc: KdlDocument = match string.parse() {
        Ok(real) => real,
        Err(_err) => return None,
    };
    let mut circles: HashMap<&str, Blade3> = HashMap::new();
    let mut twists: HashMap<&str, Turn> = HashMap::new();
    let mut real_twists: HashMap<&str, Turn> = HashMap::new();
    let mut colors: HashMap<String, Color32> = get_default_color_hash();
    let mut compounds: HashMap<&str, Vec<Turn>> = HashMap::new();
    let mut ctx = meval::Context::new();
    let mut twist_orders: HashMap<&str, isize> = HashMap::new();
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
                for created_circle in node.children()?.nodes() {
                    circles.insert(
                        created_circle.name().value(),
                        oriented_circle(
                            point(
                                match created_circle.get("x")?.is_string() {
                                    true => meval::eval_str_with_context(
                                        created_circle.get("x")?.as_string()?,
                                        &ctx,
                                    )
                                    .ok()?,
                                    false => match created_circle.get("x")?.is_float() {
                                        true => created_circle.get("x")?.as_float()?,
                                        false => created_circle.get("x")?.as_integer()? as f64,
                                    },
                                },
                                match created_circle.get("y")?.is_string() {
                                    true => meval::eval_str_with_context(
                                        created_circle.get("y")?.as_string()?,
                                        &ctx,
                                    )
                                    .ok()?,
                                    false => match created_circle.get("y")?.is_float() {
                                        true => created_circle.get("y")?.as_float()?,
                                        false => created_circle.get("y")?.as_integer()? as f64,
                                    },
                                },
                            ),
                            match created_circle.get("r")?.is_string() {
                                true => meval::eval_str_with_context(
                                    created_circle.get("r")?.as_string()?,
                                    &ctx,
                                )
                                .ok()?,
                                false => match created_circle.get("r")?.is_float() {
                                    true => created_circle.get("r")?.as_float()?,
                                    false => created_circle.get("r")?.as_integer()? as f64,
                                },
                            },
                            true,
                        ),
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
                        basic_turn(
                            *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        ),
                    );
                    twist_orders.insert(
                        turn.name().value(),
                        turn.entries().get(1)?.value().as_integer()? as isize,
                    );
                    if turn.entries().len() == 2 {
                        real_twists.insert(
                            turn.name().value(),
                            basic_turn(
                                *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                                -2.0 * PI as f64
                                    / (turn.entries().get(1)?.value().as_integer()? as f64),
                            ),
                        );
                    }
                    compounds.insert(
                        turn.name().value(),
                        vec![basic_turn(
                            *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        )],
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
                                    let number = *twist_orders.get(real)?;
                                    let mut new_adds = Vec::new();
                                    for add in &turn_seqs {
                                        for i in 0..number {
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
                        None => *circles.get(circle)?,
                        Some(real) => -*circles.get(real)?,
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

#[cfg(not(target_arch = "wasm32"))]
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
    buffer.write_all(get_puzzle_string(def.clone(), stack).as_str().as_bytes())?;
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
    detail: f32,
    scale_factor: f32,
    offset: Vec2,
    cut_on_turn: bool,
    preview: bool,
    debug: usize,
}
impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut data_storer = DataStorer { data: Vec::new() };
        let _ = data_storer.load_puzzles(&String::from("Puzzles/Definitions/"));
        let mut p =
            load_puzzle_and_def_from_file(&String::from("Puzzles/Definitions/44squares.kdl"))
                .unwrap();
        let rel_piece = p.0.pieces[0].clone();
        let c1 = rel_piece.shape.border[0].circle;
        let c2 = p.0.turns["A"].circle;
        dbg!(dbg!(c1).approx_eq(&dbg!(c2), Precision::new_simple(20)));

        // for arc in &rel_piece.shape.border {
        //     dbg!(dbg!(arc.circle).approx_eq(dbg!(&p.0.turns["A"].circle), Precision::new_simple(20)));
        //     dbg!(
        //         arc.circle
        //             .approx_eq(&dbg!(-p.0.turns["A"].circle), Precision::new_simple(20))
        //     );
        // }
        // p.0.pieces = vec![rel_piece];
        return Self {
            data_storer,
            puzzle: p.0,
            def_string: p.1,
            log_path: String::from("logfile"),
            curr_msg: String::new(),
            animation_speed: ANIMATION_SPEED,
            last_frame_time: web_time::Instant::now(),
            outline_width: 5.0,
            detail: 50.0,
            scale_factor: SCALE_FACTOR,
            offset: vec2(0.0, 0.0),
            cut_on_turn: true,
            preview: false,
            debug: 0,
        };
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // dbg!(self.puzzle.pieces.len());
            // dbg!(circle_orienation_euclid(
            //     self.puzzle.pieces[0].shape.border[0].circle
            // ));
            // let a = point(0.0, 0.0);
            // let c1 = inside_circle(a, 1.0);
            // let b = point(1.0, 0.0);
            // let c = point(0.0, 1.0);
            // //dbg!(circle_orienation_euclid(c1));
            // let arc = PieceArc {
            //     circle: c1,
            //     boundary: Some(b ^ c),
            // };

            // let mut sum = 0;
            // for piece in &self.puzzle.pieces {
            //     sum += piece.shape.border.len();
            // }
            //dbg!(sum);
            // dbg!(arc.angle_euc());
            let rect = ui.available_rect_before_wrap();
            // dbg!(sum);
            // arc.draw(
            //     ui,
            //     &rect,
            //     good_detail,
            //     NONE_TURN,
            //     self.outline_width,
            //     self.scale_factor,
            //     self.offset,
            // );
            // dbg!(self.puzzle.pieces[0].shape.bounds.len());
            if !self.preview {
                // self.puzzle.render(
                //     ui,
                //     &rect,
                //     self.detail,
                //     self.outline_width,
                //     self.scale_factor,
                //     self.offset,
                // );
                let a = PieceArc {
                    boundary: Some(Blade2 {
                        mp: -0.0293736,
                        mx: -0.03444056,
                        px: 0.024306666,
                        my: -0.01304419,
                        py: 0.03105767,
                        xy: 0.02562106,
                    }),
                    circle: Blade3 {
                        mpx: 0.00000011,
                        mpy: 0.499999863,
                        mxy: 0.58625006,
                        pxy: -0.41374993,
                    },
                };
                let c = Blade3 {
                    mpx: 0.0,
                    mpy: -0.5,
                    mxy: 0.695000,
                    pxy: -0.304999999,
                };
                let ca = PieceArc {
                    boundary: None,
                    circle: c,
                };
                dbg!(a.contains(a.intersect_circle(c)[1].unwrap()));
                if let Dipole::Real(real) = a.boundary.unwrap().unpack() {
                    dbg!(real[1].approx_eq(
                        &a.intersect_circle(c)[1].unwrap().unpack().unwrap(),
                        Precision::new_simple(20)
                    ));
                }
                dbg!(a.in_circle(c));
                //dbg!(a.in_circle(c));
                for p in [a, ca] {
                    p.draw(
                        ui,
                        &rect,
                        self.detail,
                        self.outline_width,
                        self.scale_factor,
                        self.offset,
                    );
                }
                // self.puzzle.pieces[0].render(
                //     ui,
                //     &rect,
                //     NONE_TURN,
                //     self.detail,
                //     self.outline_width,
                //     self.scale_factor,
                //     self.offset,
                // );
                // self.curr_msg = self.puzzle.pieces[0].shape.border.len().to_string();
                // self.puzzle.pieces[0].shape.border[self.debug].draw(
                //     ui,
                //     &rect,
                //     self.detail,
                //     self.outline_width,
                //     self.scale_factor,
                //     self.offset,
                // );
            } else {
                for piece in &self.puzzle.solved_state {
                    piece.render(
                        ui,
                        &rect,
                        NONE_TURN,
                        self.detail,
                        self.outline_width,
                        self.scale_factor,
                        self.offset,
                    );
                }
            }
            // let arc = self.puzzle.pieces[1].shape.border[1];
            // let arc2 = self.puzzle.pieces[1].shape.border[0];
            // dbg!(circle_orientation_euclid(arc.circle));
            // dbg!(circle_orientation_euclid(arc2.circle));
            // dbg!(self.puzzle.pieces[1].in_circle(self.puzzle.turns["A"].circle));
            // arc.draw(
            //     ui,
            //     &rect,
            //     good_detail,
            //     NONE_TURN,
            //     self.outline_width,
            //     self.scale_factor,
            //     self.offset,
            // );
            // arc2.draw(
            //     ui,
            //     &rect,
            //     good_detail,
            //     NONE_TURN,
            //     self.outline_width,
            //     self.scale_factor,
            //     self.offset,
            // );

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
            if self.puzzle.anim_left >= 0.0 {
                self.puzzle.anim_left = f32::max(
                    self.puzzle.anim_left
                        - (delta_time.as_secs_f32() * self.animation_speed as f32),
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
            if ui
                .add(egui::Button::new("INCREMENT DEBUG COUNTER"))
                .clicked()
            {
                self.debug += 1;
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
                (self.scale_factor, self.offset) = (SCALE_FACTOR, vec2(0.0, 0.0))
            }
            ui.label("Log File Path");
            ui.add(egui::TextEdit::singleline(&mut self.log_path));
            #[cfg(not(target_arch = "wasm32"))]
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
            #[cfg(not(target_arch = "wasm32"))]
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
            if self.puzzle.anim_left != 0.0 {
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
                if let Some(real_circle) = hovered_circle {
                    if let Circle::Circle {
                        cx: x,
                        cy: y,
                        r,
                        ori,
                    } = real_circle.unpack()
                    {
                        ui.painter().circle_stroke(
                            to_egui_coords(
                                pos2(x as f32, y as f32),
                                &rect,
                                self.scale_factor,
                                self.offset,
                            ),
                            r as f32 * self.scale_factor * (rect.width() / 1920.0),
                            (10.0, Color32::WHITE),
                        );
                    }
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
                let good_delta = vec2(
                    delta.x / self.scale_factor,
                    -1.0 * (delta.y / self.scale_factor),
                );
                self.offset += good_delta;
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
    let circ = circle(point(0.0, 0.0), 5.0);
    let c2 = circle(point(7.0, 0.0), 1.0);
    let bound = point(7.0, 1.0) ^ point(7.0, -1.0);
    let arc = PieceArc {
        circle: c2,
        boundary: Some(bound),
    };
    dbg!(arc.in_circle(circ));
    dbg!(arc.midpoint_euc());
    // let [i1, i2] = intersect_blade3(circ, c2)
    //     .unwrap()
    //     .unpack_point_pair()
    //     .unwrap();
    // let arc = PieceArc {
    //     circle: c2,
    //     boundary: Some(i1 ^ i2),
    // };
    // let mut arc1 = PieceArc {
    //     circle: c2,
    //     boundary: Some(a ^ b),
    // };
    // let t = basic_turn(circ, 1.72);
    // for i in 0..100000 {
    //     arc1 = arc1.rotate(t.rotation);
    // }
    // dbg!(arc1.circle);
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
