use std::f64::consts::PI;

use approx_collections::ApproxEq;

use crate::{
    PRECISION,
    complex::{arc::Arc, c64::C64, complex_circle::Contains, point::Point},
    puzzle::{piece::Piece, piece_shape::PieceShape, turn::Turn},
};

const DETAIL_SCALE: f64 = 0.5;

///the amount more detailed the outlines are than the interiors
const DETAIL_FACTOR: f64 = 3.0;

///leniency for degenerate triangles
const LENIENCY: f64 = 0.1;

pub struct Component {
    pub shape: Vec<Arc>,
}

#[derive(Debug, Clone)]
pub struct Triangulation {
    pub inside: Vec<[Point; 3]>,
    pub border: Vec<TriangulatedArc>,
}

pub type TriangulatedArc = Vec<Point>;

#[derive(Debug, Clone)]
pub struct RenderPiece {
    pub piece: Piece,
    pub triangulations: Vec<Triangulation>,
}

pub fn make_triangles(points: Vec<Point>, center: Point) -> Vec<[Point; 3]> {
    let mut triangles = Vec::new();
    for i in 0..(points.len() - 1) {
        let t = [center, points[i], points[i + 1]];
        if !almost_degenerate(&t, LENIENCY) {
            triangles.push(t);
        }
    }
    triangles
}

///take in a triangle and return if its 'almost degenerate' within some leniency (i.e. its points are 'almost colinear')
fn almost_degenerate(triangle: &[Point; 3], leniency: f64) -> bool {
    let angle_1 = (triangle[1] - triangle[0]).angle() - (triangle[1] - triangle[2]).angle(); //get the relevant (smallest/largest) angle of the triangle, by construction
    let close = angle_1.abs().min((PI - angle_1).abs()); //find how close it is to either extreme (0 or PI)
    if close < leniency {
        return true;
    }
    false
}

impl Arc {
    pub fn get_polygon(&self, detail: f64) -> Vec<Point> {
        let divisions = (self.circle.r() * self.angle.abs() * detail).max(2.0) as usize;
        let mut points = Vec::new();
        for i in 0..=divisions {
            points.push(self.start.rotate_about(
                self.circle.center,
                self.angle * (i as f64 / divisions as f64),
            ));
        }
        points
    }
}
impl Component {
    pub fn triangulate_component(&self, detail: f64) -> Triangulation {
        let mut triangles = Vec::new();
        let mut borders = Vec::new();
        let bary = self.barycenter();
        for arc in &self.shape {
            triangles.extend(make_triangles(arc.get_polygon(detail * DETAIL_SCALE), bary));
            borders.push(arc.get_polygon(detail * DETAIL_SCALE * DETAIL_FACTOR));
        }
        Triangulation {
            inside: triangles,
            border: borders,
        }
    }
    pub fn barycenter(&self) -> Point {
        let mut center = Point(C64 { re: 0.0, im: 0.0 });
        let n = self.shape.len() as f64;
        for arc in &self.shape {
            center.0.re += arc.midpoint().0.re / n;
            center.0.im += arc.midpoint().0.im / n;
        }
        center
    }
}

impl PieceShape {
    pub fn calculate_components(&self) -> Vec<Component> {
        fn find_next_arc(start: Point, arcs: &mut Vec<Arc>) -> Option<Arc> {
            let mut index = None;
            for i in 0..arcs.len() {
                if arcs[i].start.approx_eq(&start, PRECISION) {
                    index = Some(i);
                }
            }
            if let Some(ind) = index {
                Some(arcs.remove(ind))
            } else {
                None
            }
        }
        let mut comps = Vec::new();
        let mut arcs = self.border.clone();
        loop {
            if let Some(arc) = arcs.pop() {
                let mut curr_comp = vec![arc];
                comps.push(Component {
                    shape: loop {
                        if arcs.is_empty() {
                            return vec![Component {
                                shape: self.border.clone(),
                            }];
                        }
                        if let Some(next) =
                            find_next_arc(curr_comp.last().unwrap().end(), &mut arcs)
                        {
                            curr_comp.push(next);
                            if next.end().approx_eq(&curr_comp[0].start, PRECISION) {
                                break curr_comp;
                            }
                        } else {
                            return vec![Component {
                                shape: self.border.clone(),
                            }];
                        }
                    },
                });
            } else {
                break comps;
            }
        }
    }
}

impl Piece {
    pub fn triangulate(self, detail: f64) -> RenderPiece {
        RenderPiece {
            triangulations: self
                .shape
                .calculate_components()
                .iter()
                .map(|x| x.triangulate_component(detail))
                .collect(),
            piece: self,
        }
    }
}

pub fn rot_triangulations(tri: Vec<Triangulation>, turn: Turn) -> Vec<Triangulation> {
    tri.iter()
        .map(|x| Triangulation {
            inside: x
                .inside
                .iter()
                .map(|y| y.map(|z| turn.rot_point(z)))
                .collect(),
            border: x
                .border
                .iter()
                .map(|y| y.iter().map(|z| turn.rot_point(*z)).collect())
                .collect(),
        })
        .collect()
}

impl Turn {
    pub fn turn_render_piece(&self, piece: &RenderPiece) -> Option<RenderPiece> {
        let (shape, triangles) = if piece.piece.shape.in_circle(self.circle)? != Contains::Outside {
            (
                self.rot_pieceshape(&piece.piece.shape),
                rot_triangulations(piece.triangulations.clone(), *self),
            )
        } else {
            (piece.piece.shape.clone(), piece.triangulations.clone())
        };
        Some(RenderPiece {
            piece: Piece {
                shape,
                color: piece.piece.color,
            },
            triangulations: triangles,
        })
    }
    pub fn turn_cut_render_piece(
        &self,
        piece: &RenderPiece,
        detail: f64,
    ) -> Result<Vec<RenderPiece>, String> {
        let cut_pieces = self.turn_cut_piece(&piece.piece)?;
        let mut returns = Vec::new();
        for cut_piece in cut_pieces {
            returns.push(cut_piece.triangulate(detail));
        }
        Ok(returns)
    }
}
