use egui::Color32;

use crate::{
    complex::complex_circle::{Circle, ComplexCircle, Contains, OrientedCircle},
    puzzle::{color::Color, piece_shape::PieceShape},
};

#[derive(Clone, Debug)]
pub struct Piece {
    pub shape: PieceShape,
    pub color: Color,
}

impl Piece {
    ///cut a piece by a circle. returns None if no cut was made and otherwise (inside, outside)
    pub fn cut_by_circle(&self, circle: Circle) -> Option<(Piece, Piece)> {
        if let Some((i, o)) = self.shape.cut_by_circle(circle) {
            Some((
                Piece {
                    shape: i,
                    color: self.color,
                },
                Piece {
                    shape: o,
                    color: self.color,
                },
            ))
        } else {
            None
        }
    }
    ///see if a point lies inside a bunch of oriented circles (a CCP representation)
    pub fn in_region(&self, region: &Vec<OrientedCircle>) -> bool {
        for circ in region {
            let inside = self.shape.in_circle(circ.circ);
            if inside != Some(circ.ori) && inside != Some(Contains::Border) {
                return false;
            }
        }
        true
    }
    pub fn in_circle(&self, circle: ComplexCircle) -> Option<Contains> {
        self.shape.in_circle(circle)
    }
}
