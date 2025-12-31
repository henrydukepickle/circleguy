use std::{collections::HashMap, f64::consts::PI};

use approx_collections::FloatPool;
use egui::Color32;

use crate::{
    PRECISION,
    complex::{
        arc::Arc,
        complex_circle::{ComplexCircle, Contains, OrientedCircle},
    },
    puzzle::{
        color::Color,
        piece::Piece,
        piece_shape::PieceShape,
        puzzle::Puzzle,
        turn::{OrderedTurn, Turn},
    },
};

#[derive(Clone, Debug)]
pub struct HPSPuzzleData {
    pub name: String,
    pub authors: Vec<String>,
    pub pieces: Vec<Piece>,
    pub turns: HashMap<String, OrderedTurn>,
    pub stack: Vec<(String, isize)>,
    pub intern: FloatPool,
    pub disks: Vec<ComplexCircle>,
}

impl HPSPuzzleData {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            authors: vec![],
            pieces: vec![],
            turns: HashMap::new(),
            stack: vec![],
            intern: FloatPool::new(PRECISION),
            disks: vec![],
        }
    }
    pub fn to_puzzle(&self) -> Puzzle {
        Puzzle {
            name: self.name.clone(),
            authors: self.authors.clone(),
            pieces: self.pieces.clone(),
            turns: self.turns.clone(),
            stack: vec![],
            scramble: None,
            animation_offset: None,
            intern: self.intern.clone(),
            depth: 500,
            solved_state: Some(self.pieces.clone()),
            solved: false,
            anim_left: 0.0,
            def: String::new(),
        }
    }
    pub fn add_disk(&mut self, disk: ComplexCircle) -> bool {
        let mut disk_piece = full_circle_piece(disk);
        for old_disk in &self.disks {
            if let Some((_, o)) = disk_piece.cut_by_circle(*old_disk) {
                disk_piece = o;
            } else {
                if disk_piece.in_circle(*old_disk) != Some(Contains::Outside) {
                    return false;
                }
            }
        }
        self.disks.push(disk);
        self.pieces.push(disk_piece);
        return true;
    }
    pub fn turn(&mut self, turn: OrderedTurn, cut: bool) -> Result<bool, String> {
        let mut new_pieces = Vec::new(); //make a list of new pieces to populate
        if cut {
            //if cut is true, cut
            for piece in &self.pieces {
                for turned in turn.turn_cut_piece(piece)? {
                    //cut each piece
                    new_pieces.push(turned); //add it to the list
                }
            }
        } else {
            for piece in &self.pieces {
                new_pieces.push(match turn.turn_piece(piece) {
                    None => return Ok(false),
                    Some(x) => x,
                }); //otherwise, just turn each piece
            }
        }
        self.pieces = new_pieces;
        //self.intern_all(); //intern everything
        Ok(true)
    }
    pub fn cut(&mut self, cut: &Vec<OrderedTurn>) -> Result<(), String> {
        for turn in cut {
            self.turn(*turn, true)?;
        }
        for turn in cut.into_iter().rev() {
            self.turn(turn.inverse(), false)?;
        }
        Ok(())
    }
    pub fn color(&mut self, region: &Vec<OrientedCircle>, color: Color) {
        for piece in &mut self.pieces {
            if piece.in_region(region) {
                piece.color = color;
            }
        }
    }
}

fn full_circle_piece(circ: ComplexCircle) -> Piece {
    Piece {
        shape: PieceShape {
            bounds: vec![OrientedCircle {
                circ: circ,
                ori: Contains::Inside,
            }],
            border: vec![Arc {
                circle: circ,
                start: circ.right_point(),
                angle: 2.0 * PI,
            }],
        },
        color: Color::None,
    }
}
