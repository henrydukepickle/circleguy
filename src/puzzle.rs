use crate::piece::*;
use crate::puzzle_generation::parse_kdl;
use crate::turn::*;
use approx_collections::*;
use rand::SeedableRng;
use rand::prelude::IteratorRandom;
use std::array;
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Instant;
#[derive(Debug, Clone)]
pub struct Puzzle {
    pub name: String,
    pub authors: Vec<String>,
    pub pieces: Vec<Piece>,
    pub turns: HashMap<String, Turn>,
    pub stack: Vec<String>,
    pub scramble: Option<[String; 500]>,
    pub animation_offset: Option<Turn>,
    pub intern_2: FloatPool,
    pub intern_3: FloatPool,
    pub depth: u16,
    pub solved_state: Vec<Piece>,
    pub solved: bool,
    pub anim_left: f32,
    pub def: String,
}
impl Puzzle {
    ///turns the puzzle around a turn. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true)
    ///if the turn was bandaged (and cut was false), returns Ok(false)
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn(&mut self, turn: Turn, cut: bool) -> Result<bool, String> {
        let mut new_pieces = Vec::new();
        if cut {
            for piece in &self.pieces {
                for possible in piece.turn_cut(turn)? {
                    if let Some(x) = possible {
                        new_pieces.push(x);
                    }
                }
            }
        } else {
            for piece in &self.pieces {
                new_pieces.push(match piece.turn(turn)? {
                    None => return Ok(false),
                    Some(x) => x,
                });
            }
        }
        self.pieces = new_pieces;
        self.anim_left = 1.0;
        self.animation_offset = Some(turn.inverse());
        self.intern_all();
        Ok(true)
    }
    pub fn turn_id(&mut self, id: String, cut: bool) -> Result<bool, String> {
        let turn = self.turns[&id];
        if !self.turn(turn, cut)? {
            return Ok(false);
        }
        self.stack.push(id);
        //self.check();
        Ok(true)
    }
    ///undoes the last turn.
    ///Ok(true) means that the move was undone successfully
    ///Ok(false) means that the stack was empty
    ///Err(e) means that an error was encountered
    pub fn undo(&mut self) -> Result<bool, String> {
        if self.stack.len() == 0 {
            return Ok(false);
        }
        let last_turn = self.turns[&self.stack.pop().unwrap()];
        if !self.turn(last_turn.inverse(), false)? {
            return Err(String::from("Puzzle.undo failed: undo turn was bandaged!"));
        };
        //self.check();
        Ok(true)
    }
    pub fn scramble(&mut self, cut: bool) -> Result<(), String> {
        self.reset()?;
        let mut scramble = array::from_fn(|_| "".to_string());
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let bytes = h.finish().to_ne_bytes();
        let mut rng = rand::rngs::StdRng::from_seed(
            [bytes; 4]
                .as_flattened()
                .try_into()
                .expect("error casting [[u8; 8]; 4] to [u8; 32]"),
        );
        for i in 0..self.depth {
            let key = self
                .turns
                .keys()
                .choose(&mut rng)
                .ok_or("Puzzle.scramble failed: rng choosing a turn failed!".to_string())?
                .clone();
            // dbg!(&key);
            self.turn(self.turns[&key], cut)?;
            scramble[i as usize] = key;
        }
        self.animation_offset = None;
        self.scramble = Some(scramble);
        //self.check();
        Ok(())
    }
    pub fn reset(&mut self) -> Result<(), String> {
        *self =
            parse_kdl(&self.def).ok_or("Puzzle.reset failed: parsing kdl failed!".to_string())?;
        Ok(())
    }
    // pub fn global_cut_by_circle(&mut self, circle: Blade3) -> Result<(), ()> {
    //     let mut new_pieces = Vec::new();
    //     for piece in &self.pieces {
    //         //dbg!(piece.shape.border.len());
    //         match piece.cut_by_circle(circle) {
    //             None => new_pieces.push(piece.clone()),
    //             Some(x) => new_pieces.extend(x),
    //         }
    //     }
    //     self.pieces = new_pieces;
    //     //self.intern_all();
    //     Ok(())
    // }
}
