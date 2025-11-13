use crate::piece_shape::*;
use crate::turn::*;
use cga2d::*;
use egui::Color32;
#[derive(Debug, Clone)]
///a piece. stores a shape and a color
pub struct Piece {
    pub shape: PieceShape,
    pub color: Color32,
}
///a connected component of a piece, storing a shape and a color
pub struct Component {
    pub shape: ComponentShape,
    pub color: Color32,
}

impl Piece {
    ///turn a piece around a turn, without cutting
    pub fn turn(&self, turn: Turn) -> Result<Option<Piece>, String> {
        return Ok(Some(Piece {
            //essentially just call self.shape.turn and keep the color the same
            shape: match self.shape.turn(turn)? {
                None => return Ok(None),
                Some(x) => x,
            },
            color: self.color,
        }));
    }
    ///turn a piece around a turn, with cutting
    pub fn turn_cut(&self, turn: Turn) -> Result<[Option<Piece>; 2], String> {
        //essentially just call self.shape.turn_cut and keep the color the same
        return Ok(self.shape.turn_cut(turn)?.map(|x| match x {
            None => None,
            Some(x) => Some(Piece {
                shape: x,
                color: self.color,
            }),
        }));
    }
    ///cut a piece by a circle. for more specifics about the return type see PieceShape.cut_by_circle
    pub fn cut_by_circle(&self, circle: Blade3) -> Result<[Option<Piece>; 2], String> {
        //essentially just cut the shape by a circle and keep the colors the same
        let shapes = self.shape.cut_by_circle(circle)?;
        Ok(shapes.map(|x| match x {
            None => None,
            Some(x) => Some(Piece {
                shape: x,
                color: self.color,
            }),
        }))
    }
    ///get the components of the piece for fine rendering
    pub fn get_components(&self, correct: bool) -> Result<Vec<Component>, String> {
        //essentially just get the component shapes of self.shape
        Ok(self
            .shape
            .get_components(correct)?
            .iter()
            .map(|shape| Component {
                shape: shape.clone(),
                color: self.color,
            })
            .collect())
    }
}
