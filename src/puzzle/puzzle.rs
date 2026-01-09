use crate::DETAIL;
use crate::POOL_PRECISION;
use crate::puzzle::piece::*;
use crate::puzzle::render_piece::RenderPiece;
use crate::puzzle::turn::*;
use approx_collections::FloatPool;
use rand::SeedableRng;
use rand::prelude::IteratorRandom;
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Instant;
#[derive(Debug, Clone)]
pub struct Puzzle {
    pub name: String,
    pub authors: Vec<String>,
    pub pieces: Vec<RenderPiece>,
    pub turns: HashMap<String, OrderedTurn>,
    pub stack: Vec<(String, isize)>,
    pub scramble: Option<Vec<String>>,
    pub animation_offset: Option<Turn>, //the turn of the puzzle that the animation is currently doing
    pub intern: FloatPool,
    pub depth: u16,
    pub solved: bool,
    pub anim_left: f32, //the amount of animation left
    pub data: PuzzleData,
    pub keybinds: HashMap<egui::Key, (String, isize)>,
}
#[derive(Debug, Clone)]
pub struct PuzzleData {
    pub name: String,
    pub authors: Vec<String>,
    pub pieces: Vec<Piece>,
    pub turns: HashMap<String, OrderedTurn>,
    pub intern: FloatPool,
    pub depth: usize,
    pub keybinds: HashMap<egui::Key, (String, isize)>,
}

impl Puzzle {
    pub fn new(data: PuzzleData) -> Self {
        Self {
            name: data.name.clone(),
            authors: data.authors.clone(),
            pieces: data
                .pieces
                .iter()
                .map(|x| x.clone().triangulate(DETAIL))
                .collect(),
            turns: data.turns.clone(),
            stack: vec![],
            scramble: None,
            animation_offset: None,
            intern: FloatPool::new(POOL_PRECISION),
            depth: 500,
            solved: true,
            anim_left: 0.0,
            keybinds: data.keybinds.clone(),
            data,
        }
    }
    ///checks if self is solved and updates self.is_solved accordingly
    pub fn check(&mut self) {
        self.solved = self.is_solved();
    }
    ///turns the puzzle around a turn. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true)
    ///if the turn was bandaged (and cut was false), returns Ok(false)
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn(&mut self, turn: OrderedTurn, cut: bool) -> Result<bool, String> {
        let mut new_pieces = Vec::new(); //make a list of new pieces to populate
        if cut {
            //if cut is true, cut
            for piece in &self.pieces {
                for turned in turn.turn.turn_cut_render_piece(piece, DETAIL)? {
                    //cut each piece
                    new_pieces.push(turned); //add it to the list
                }
            }
        } else {
            for piece in &self.pieces {
                new_pieces.push(match turn.turn.turn_render_piece(piece) {
                    None => return Ok(false),
                    Some(x) => x,
                }); //otherwise, just turn each piece
            }
        }
        self.pieces = new_pieces;
        self.anim_left = 1.0; //set the animation to run
        self.animation_offset = Some(turn.turn.inverse());
        self.intern_all(); //intern everything
        self.solved = false;
        Ok(true)
    }
    ///turns the puzzle around a turn, given by an id. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true).
    ///if the turn was bandaged (and cut was false), returns Ok(false).
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn_id(&mut self, id: &str, cut: bool, mult: isize) -> Result<bool, String> {
        let turn = self
            .turns
            .get(id)
            .ok_or("No turn found with ID!".to_string())?
            .mult(mult);
        if !self.turn(turn, cut)? {
            return Ok(false);
        }
        self.stack.push((id.to_string(), mult));
        Ok(true)
    }
    ///undoes the last turn.
    ///Ok(true) means that the move was undone successfully
    ///Ok(false) means that the stack was empty
    ///Err(e) means that an error was encountered
    pub fn undo(&mut self) -> Result<bool, String> {
        if let Some(last) = &self.stack.pop() {
            let last_turn = self.turns[&last.0]; //try to find the last turn
            if !self.turn(last_turn.inverse().mult(last.1), false)? {
                return Err(String::from("Puzzle.undo failed: undo turn was bandaged!"));
            };
            Ok(true)
        } else {
            Ok(false)
        }
    }
    ///scramble the puzzle 500 moves
    pub fn scramble(&mut self, cut: bool) -> Result<(), String> {
        self.reset()?;
        let mut scramble = Vec::new();
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let bytes = h.finish().to_ne_bytes();
        let mut rng = rand::rngs::StdRng::from_seed(
            //initialize the rng from a seed. this is needed for web reasons
            [bytes; 4]
                .as_flattened()
                .try_into()
                .expect("error casting [[u8; 8]; 4] to [u8; 32]"),
        );
        for _ in 0..self.depth {
            //choose a random turn and do it
            let key = self
                .turns
                .keys()
                .choose(&mut rng)
                .ok_or("Puzzle.scramble failed: rng choosing a turn failed!".to_string())?
                .clone();
            self.turn(self.turns[&key], cut)?;
            scramble.push(key);
        }
        self.animation_offset = None;
        self.scramble = Some(scramble); //set the scramble to Some
        Ok(())
    }
    ///reset the puzzle, using the stored definition
    pub fn reset(&mut self) -> Result<(), String> {
        *self = Puzzle::new(self.data.clone());
        Ok(())
    }
}
