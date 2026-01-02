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
        puzzle::{Puzzle, PuzzleData},
        turn::{OrderedTurn, Turn},
    },
};

const NAMES: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

#[derive(Clone, Debug)]
pub struct HPSPuzzleData {
    pub name: String,
    pub authors: Vec<String>,
    pub pieces: Vec<Piece>,
    pub turns: HashMap<String, OrderedTurn>,
    pub stack: Vec<OrderedTurn>,
    pub intern: FloatPool,
    pub disks: Vec<ComplexCircle>,
    pub scramble: usize,
}

impl HPSPuzzleData {
    ///intern all the relevant floats in the puzzle into the float pool
    pub fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                self.intern.intern_in_place(&mut arc.circle.center.0.re);
                self.intern.intern_in_place(&mut arc.circle.center.0.im);
                self.intern.intern_in_place(&mut arc.circle.r_sq);
                self.intern.intern_in_place(&mut arc.start.0.re);
                self.intern.intern_in_place(&mut arc.start.0.im);
                self.intern.intern_in_place(&mut arc.angle);
            }
            for circle in &mut piece.shape.bounds {
                self.intern.intern_in_place(&mut circle.circ.center.0.re);
                self.intern.intern_in_place(&mut circle.circ.center.0.im);
                self.intern.intern_in_place(&mut circle.circ.r_sq);
            }
        }
    }
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
            scramble: 0,
        }
    }
    pub fn to_puzzle_data(&self) -> PuzzleData {
        PuzzleData {
            name: self.name.clone(),
            authors: self.authors.clone(),
            pieces: self.pieces.clone(),
            turns: self.turns.clone(),
            intern: self.intern.clone(),
            scramble: self.scramble,
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
        self.intern_all();
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
        self.stack.push(turn);
        self.intern_all(); //intern everything
        Ok(true)
    }
    pub fn cut(&mut self, cut: &Vec<OrderedTurn>) -> Result<(), String> {
        for turn in cut {
            self.turn(*turn, true)?;
        }
        self.undo_num(cut.len());
        Ok(())
    }
    ///returns true if something was there to be undone
    pub fn undo(&mut self) -> bool {
        if let Some(t) = self.stack.pop() {
            self.turn(t.inverse(), false);
            self.stack.pop();
            true
        } else {
            false
        }
    }
    pub fn undo_num(&mut self, mut num: usize) {
        while num > 0 && self.undo() {
            num -= 1;
        }
    }
    pub fn undo_all(&mut self) {
        while self.undo() {}
    }
    pub fn color(&mut self, region: &Vec<OrientedCircle>, color: Color) {
        for piece in &mut self.pieces {
            if piece.in_region(region) {
                piece.color = color;
            }
        }
    }
    pub fn next_turn_name(&self) -> Option<String> {
        for ch in NAMES {
            if !self.turns.contains_key(ch) {
                return Some(ch.to_string());
            }
        }
        None
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
