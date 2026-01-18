use approx_collections::ApproxEq;

use crate::{
    PRECISION,
    puzzle::{piece::Piece, piece_shape::PieceShape, puzzle::Puzzle},
};

impl ApproxEq for PieceShape {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        compare_vecs(&self.border, &other.border, |x, y| x.approx_eq(y, prec))
    }
}

pub fn same_pieces(first: &Vec<Piece>, second: &Vec<Piece>) -> bool {
    compare_vecs(first, second, |x, y| {
        x.color == y.color && x.shape.approx_eq(&y.shape, PRECISION)
    })
}

impl Puzzle {
    pub fn is_solved(&self) -> bool {
        same_pieces(
            &self.pieces.iter().map(|x| x.piece.clone()).collect(),
            &self.data.pieces,
        )
    }
}

pub fn compare_vecs<T: Clone, F: Fn(&T, &T) -> bool>(
    first: &Vec<T>,
    second: &Vec<T>,
    eq: F,
) -> bool {
    if first.len() != second.len() {
        return false;
    }
    let (mut one, mut two) = (first.clone(), second.clone());
    loop {
        let mut index = None;
        for i in 0..two.len() {
            if eq(&one[0], &two[i]) {
                index = Some(i);
                break;
            }
        }
        if let Some(i) = index {
            two.remove(i);
            one.remove(0);
            if one.len() == 0 {
                break true;
            }
        } else {
            break false;
        }
    }
}
