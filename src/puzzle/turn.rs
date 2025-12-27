use crate::{
    complex::{
        arc::Arc,
        c64::{C64, Point, Scalar},
        complex_circle::{Circle, Contains, OrientedCircle},
    },
    puzzle::{piece::Piece, piece_shape::PieceShape},
};

pub type Rot = C64;

#[derive(Clone, Debug, Copy)]
pub struct Turn {
    pub circle: Circle,
    pub rot: Rot,
}

impl Turn {
    pub fn inverse(&self) -> Self {
        Self {
            circle: self.circle,
            rot: self.rot.conj(),
        }
    }
    pub fn mult(&self, mult: Scalar) -> Self {
        Self {
            circle: self.circle,
            rot: (Rot::from_angle(self.rot.angle() * mult)),
        }
    }
    pub fn rot_point(&self, point: Point) -> Point {
        (self.rot * (point - self.circle.center)) + self.circle.center
    }
    pub fn rot_circle(&self, circle: Circle) -> Circle {
        Circle {
            center: self.rot_point(circle.center),
            r_sq: circle.r_sq,
        }
    }
    pub fn rot_arc(&self, arc: Arc) -> Arc {
        Arc {
            circle: self.rot_circle(arc.circle),
            start: self.rot_point(arc.start),
            angle: arc.angle,
        }
    }
    pub fn rot_pieceshape(&self, shape: &PieceShape) -> PieceShape {
        PieceShape {
            bounds: shape
                .bounds
                .iter()
                .map(|x| OrientedCircle {
                    circ: self.rot_circle(x.circ),
                    ori: x.ori,
                })
                .collect(),
            border: shape.border.iter().map(|x| self.rot_arc(*x)).collect(),
        }
    }
    pub fn turn_pieceshape(&self, shape: &PieceShape) -> Option<PieceShape> {
        match shape.in_circle(self.circle) {
            None => None,
            Some(Contains::Inside) | Some(Contains::Border) => Some(self.rot_pieceshape(shape)),
            Some(Contains::Outside) => Some(shape.clone()),
        }
    }
    pub fn turn_cut_pieceshape(&self, shape: &PieceShape) -> Vec<PieceShape> {
        match dbg!(shape.in_circle(self.circle)) {
            None => {
                let (i, o) = shape.cut_by_circle(self.circle).unwrap();
                vec![self.rot_pieceshape(&i), o]
            }
            Some(Contains::Inside) | Some(Contains::Border) => {
                vec![self.rot_pieceshape(shape)]
            }
            Some(Contains::Outside) => vec![shape.clone()],
        }
    }
    pub fn turn_piece(&self, piece: &Piece) -> Option<Piece> {
        Some(Piece {
            shape: self.turn_pieceshape(&piece.shape)?,
            color: piece.color,
        })
    }
    pub fn turn_cut_piece(&self, piece: &Piece) -> Vec<Piece> {
        self.turn_cut_pieceshape(&piece.shape)
            .iter()
            .map(|x| Piece {
                shape: x.clone(),
                color: piece.color,
            })
            .collect()
    }
}
