use crate::piece::*;
use crate::puzzle_generation::parse_kdl;
use crate::turn::*;
use approx_collections::*;
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
    pub pieces: Vec<Piece>,
    pub turns: HashMap<String, Turn>,
    pub stack: Vec<String>,
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
    // fn intern_all(&mut self) {
    //     for piece in &mut self.pieces {
    //         for arc in &mut piece.shape.border {
    //             self.intern.intern_blade3(&mut arc.circle);
    //             if let Some(bound) = arc.boundary.as_mut() {
    //                 self.intern.intern_blade2(bound);
    //             }
    //         }
    //         for circ in &mut piece.shape.bounds {
    //             self.intern.intern_blade3(circ);
    //         }
    //     }
    // }
    //updates self.solved
    // fn check(&mut self) {
    //     self.solved = false;
    //     //TEMPORARY -- SOLVED CHECKING
    // }
    //returns if the turn could be completed
    //Err(true) means that the turn was bandaged
    //Err(false) means that the cutting failed
    pub fn turn(&mut self, turn: Turn, cut: bool) -> Result<Result<(), bool>, String> {
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
                    None => return Ok(Err(true)),
                    Some(x) => x,
                });
            }
        }
        self.pieces = new_pieces;
        self.anim_left = 1.0;
        self.animation_offset = Some(turn.inverse());
        self.intern_all();
        Ok(Ok(()))
    }
    pub fn turn_id(&mut self, id: String, cut: bool) -> Result<Result<(), bool>, String> {
        let turn = self.turns[&id];
        if let Err(x) = self.turn(turn, cut)? {
            return Ok(Err(x));
        }
        self.stack.push(id);
        //self.check();
        Ok(Ok(()))
    }
    pub fn undo(&mut self) -> Result<Result<(), bool>, String> {
        if self.stack.len() == 0 {
            return Ok(Err(true));
        }
        let last_turn = self.turns[&self.stack.pop().unwrap()];
        if let Err(x) = self.turn(last_turn.inverse(), false)? {
            return Ok(Err(x));
        };
        //self.check();
        Ok(Ok(()))
    }
    pub fn scramble(&mut self, cut: bool) -> Result<(), String> {
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let bytes = h.finish().to_ne_bytes();
        let mut rng = rand::rngs::StdRng::from_seed(
            [bytes; 4]
                .as_flattened()
                .try_into()
                .expect("error casting [[u8; 8]; 4] to [u8; 32]"),
        );
        for _i in 0..self.depth {
            let key = self
                .turns
                .keys()
                .choose(&mut rng)
                .ok_or("Puzzle.scramble failed: rng choosing a turn failed!".to_string())?
                .clone();
            // dbg!(&key);
            if self.turn(self.turns[&key], cut)?.is_err_and(|x| !x) {
                return Err("Puzzle.scramble failed: cutting failed while scrambling!".to_string());
            }
            self.stack.push(key);
        }
        self.animation_offset = None;
        //self.check();
        Ok(())
    }
    pub fn reset(&mut self) -> Result<(), ()> {
        *self = parse_kdl(&self.def).ok_or(())?;
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
