use egui::Color32;

use crate::{complex::complex_circle::Circle, puzzle::piece_shape::PieceShape};

#[derive(Clone, Debug)]
pub struct Piece {
    pub shape: PieceShape,
    pub color: Color32,
}

impl Piece {
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
}
