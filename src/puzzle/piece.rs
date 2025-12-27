use egui::Color32;

use crate::puzzle::piece_shape::PieceShape;

#[derive(Clone, Debug)]
pub struct Piece {
    pub shape: PieceShape,
    pub color: Color32,
}

impl Piece {}
