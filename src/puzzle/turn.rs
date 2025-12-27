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
///turn of a certain angle around a circle. only points within the circle should be affected.
pub struct Turn {
    pub circle: Circle,
    pub rot: Rot, //rotation is stored as a mag-1 complex number
}

impl Turn {
    ///take the inverse of a turn, which is around the same circle but with flipped sign on the angle
    pub fn inverse(&self) -> Self {
        Self {
            circle: self.circle,
            rot: self.rot.conj(),
        }
    }
    ///multiply a turn by a scalar.
    pub fn mult(&self, mult: Scalar) -> Self {
        Self {
            circle: self.circle,
            rot: (Rot::from_angle(self.rot.angle() * mult)), //multiply the angle by the scalar and recalculate the number
        }
    }
    ///rotate a point according to the turn. does not care whether the point is in/out of the circle
    pub fn rot_point(&self, point: Point) -> Point {
        (self.rot * (point - self.circle.center)) + self.circle.center
    }
    ///rotate a circle according to the turn. does not care whether the circle is in/out of the circle
    pub fn rot_circle(&self, circle: Circle) -> Circle {
        Circle {
            center: self.rot_point(circle.center),
            r_sq: circle.r_sq,
        }
    }
    ///rotate an arc according to the turn. does not care whether the arc is in/out of the circle
    pub fn rot_arc(&self, arc: Arc) -> Arc {
        Arc {
            circle: self.rot_circle(arc.circle),
            start: self.rot_point(arc.start),
            angle: arc.angle,
        }
    }
    ///rotate a shape according to the turn. does not care whether the shape is in/out of the circle
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
    ///turn a pieceshape according to the turn, without cutting. returns None if the turn is blocked by the piece.
    ///pieces outside turn.circle are unaffected
    pub fn turn_pieceshape(&self, shape: &PieceShape) -> Option<PieceShape> {
        match shape.in_circle(self.circle) {
            None => None,
            Some(Contains::Inside) | Some(Contains::Border) => Some(self.rot_pieceshape(shape)),
            Some(Contains::Outside) => Some(shape.clone()),
        }
    }
    ///turn a pieceshape according to the turn, with cutting. returns 1 or 2 pieces.
    ///if two shapes are returned, a cut was made and exactly one of the two shape was rotated.
    pub fn turn_cut_pieceshape(&self, shape: &PieceShape) -> Result<Vec<PieceShape>, String> {
        match shape.in_circle(self.circle) {
            None => {
                let (i, o) = shape
                    .cut_by_circle(self.circle)
                    .ok_or("Turn.turn_cut_pieceshape failed: shape crossed cut but was not cut!")?;
                Ok(vec![self.rot_pieceshape(&i), o])
            }
            Some(Contains::Inside) | Some(Contains::Border) => Ok(vec![self.rot_pieceshape(shape)]),
            Some(Contains::Outside) => Ok(vec![shape.clone()]),
        }
    }
    ///turn a piece according to the turn, without cutting. see turn_pieceshape().
    pub fn turn_piece(&self, piece: &Piece) -> Option<Piece> {
        Some(Piece {
            shape: self.turn_pieceshape(&piece.shape)?,
            color: piece.color,
        })
    }
    ///turn a piece according to the turn, with cutting. see turn_cut_pieceshape().
    pub fn turn_cut_piece(&self, piece: &Piece) -> Result<Vec<Piece>, String> {
        Ok(self
            .turn_cut_pieceshape(&piece.shape)?
            .iter()
            .map(|x| Piece {
                shape: x.clone(),
                color: piece.color,
            })
            .collect())
    }
}
