use crate::arc::*;
use crate::circle_utils::*;
use crate::io::*;
use crate::piece::*;
use crate::piece_shape::*;
use crate::puzzle::*;
use crate::turn::*;
use cga2d::*;
use egui::*;
use kdl::*;
use std::collections::HashMap;
use std::f32::consts::PI;

const NONE_COLOR: Color32 = Color32::GRAY;

type Cut = Vec<Turn>;
type Coloring = (BoundingCircles, Color32);
fn get_default_color_hash() -> HashMap<String, Color32> {
    let mut hash = HashMap::new();
    hash.insert("RED".to_string(), Color32::RED);
    hash.insert("BLUE".to_string(), Color32::BLUE);
    hash.insert("GREEN".to_string(), Color32::GREEN);
    hash.insert("YELLOW".to_string(), Color32::YELLOW);
    hash.insert("PURPLE".to_string(), Color32::PURPLE);
    hash.insert("GRAY".to_string(), Color32::GRAY);
    hash.insert("BLACK".to_string(), Color32::BLACK);
    hash.insert("BROWN".to_string(), Color32::BROWN);
    hash.insert("CYAN".to_string(), Color32::CYAN);
    hash.insert("WHITE".to_string(), Color32::WHITE);
    hash.insert("DARK_BLUE".to_string(), Color32::DARK_BLUE);
    hash.insert("DARK_GREEN".to_string(), Color32::DARK_GREEN);
    hash.insert("DARK_GRAY".to_string(), Color32::DARK_GRAY);
    hash.insert("DARK_RED".to_string(), Color32::DARK_RED);
    hash.insert("LIGHT_BLUE".to_string(), Color32::LIGHT_BLUE);
    hash.insert("LIGHT_GRAY".to_string(), Color32::LIGHT_GRAY);
    hash.insert("LIGHT_GREEN".to_string(), Color32::LIGHT_GREEN);
    hash.insert("LIGHT_YELLOW".to_string(), Color32::LIGHT_YELLOW);
    hash.insert("LIGHT_RED".to_string(), Color32::LIGHT_RED);
    hash.insert("KHAKI".to_string(), Color32::KHAKI);
    hash.insert("GOLD".to_string(), Color32::GOLD);
    hash.insert("MAGENTA".to_string(), Color32::MAGENTA);
    hash.insert("ORANGE".to_string(), Color32::ORANGE);
    hash
}
impl Puzzle {
    fn cut(&mut self, cut: &Cut) -> Result<(), ()> {
        for turn in cut {
            self.turn(*turn, true).or(Err(()))?;
        }
        for turn in cut.clone().into_iter().rev() {
            self.turn(turn.inverse(), false).or(Err(()))?;
        }
        Ok(())
    }
    fn color(&mut self, coloring: &Coloring) {
        for piece in &mut self.pieces {
            let mut color = true;
            for circle in &coloring.0 {
                let contains = piece.shape.in_circle(*circle);
                if contains != Some(Contains::Inside) {
                    color = false;
                    break;
                }
            }
            if color {
                piece.color = coloring.1;
            }
        }
    }
}
pub fn basic_turn(raw_circle: Blade3, angle: f64) -> Turn {
    if let Circle::Circle {
        cx,
        cy,
        r: _,
        ori: _,
    } = raw_circle.unpack()
    {
        let p1 = point(cx, cy);
        let p2 = point(cx + 1.0, cy);
        let p3 = point(cx + (angle / 2.0).cos(), cy + (angle / 2.0).sin());
        return Turn {
            circle: raw_circle,
            rotation: Rotoflector::Rotor((p1 ^ p3 ^ NI) * (p1 ^ p2 ^ NI)),
        };
    } else {
        panic!("you passed a line!");
    }
}
fn multiply_turns(a: isize, compound: &Vec<Turn>) -> Vec<Turn> {
    if a < 0 {
        return invert_compound_turn(&multiply_turns(-1 * a, compound));
    } else if a > 0 {
        let mut multiply_turns = multiply_turns(a - 1, compound);
        multiply_turns.extend(compound);
        return multiply_turns;
    } else {
        return Vec::new();
    }
}

fn invert_compound_turn(compound: &Vec<Turn>) -> Vec<Turn> {
    let mut turns = Vec::new();
    for turn in compound.into_iter().rev() {
        turns.push(turn.inverse());
    }
    return turns;
}
fn make_basic_puzzle(disks: Vec<Blade3>) -> Result<Vec<Piece>, ()> {
    let mut pieces = Vec::new();
    let mut old_disks = Vec::new();
    for disk in &disks {
        let mut disk_piece = Piece {
            shape: PieceShape {
                bounds: vec![*disk],
                border: vec![Arc {
                    circle: *disk,
                    boundary: None,
                }],
            },
            color: NONE_COLOR,
        };
        for old_disk in &old_disks {
            disk_piece = match disk_piece.cut_by_circle(*old_disk) {
                None => disk_piece.clone(),
                Some(x) => x[1].clone(),
            };
        }
        old_disks.push(*disk);
        pieces.push(disk_piece);
    }
    return Ok(pieces);
}

fn puzzle_from_string(string: String) -> Option<Puzzle> {
    let components = string
        .split("--LOG FILE \n")
        .into_iter()
        .collect::<Vec<&str>>();
    let mut puzzle = parse_kdl(components.get(0)?)?;
    if components.len() > 1 {
        let turns = components.get(1)?.split(",");
        for turn in turns {
            puzzle.turn_id(String::from(turn), false).ok()?;
        }
    }
    return Some(puzzle);
}

pub fn load_puzzle_and_def_from_file(path: &String) -> Option<(Puzzle, String)> {
    let contents = read_file_to_string(path).ok()?;
    return Some((
        puzzle_from_string(contents.clone())?,
        String::from(
            contents
                .split("--LOG FILE")
                .into_iter()
                .collect::<Vec<&str>>()[0],
        ),
    ));
}

// fn load(to_load: PuzzleDef, to_set: &mut PuzzleDef) -> Puzzle {
//     *to_set = to_load;
//     return puzzle_from_two_circles(to_load);
// }

fn strip_number_end(str: &str) -> Option<(String, String)> {
    let chars = str.chars();
    let end = chars
        .rev()
        .take_while(|x| x.is_numeric())
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect::<String>();
    return match end.is_empty() {
        true => None,
        false => Some((String::from(str.strip_suffix(&end)?), end)),
    };
}

fn oriented_circle(cent: Blade1, rad: f64, inside: bool) -> Blade3 {
    let circ = circle(cent, rad);
    return if (circle_orientation_euclid(circ) == Contains::Outside) == inside {
        -circ
    } else {
        circ
    };
}

pub fn parse_kdl(string: &str) -> Option<Puzzle> {
    let mut puzzle = Puzzle {
        name: String::new(),
        authors: Vec::new(),
        pieces: Vec::new(),
        turns: HashMap::new(),
        stack: Vec::new(),
        animation_offset: None,
        intern_2: approx_collections::FloatPool::new(Precision::new_simple(20)),
        intern_3: approx_collections::FloatPool::new(Precision::new_simple(20)),
        depth: 500,
        solved_state: Vec::new(),
        solved: true,
        anim_left: 0.0,
    };
    let mut def_stack = Vec::new();
    let doc: KdlDocument = match string.parse() {
        Ok(real) => real,
        Err(_err) => return None,
    };
    let mut circles: HashMap<&str, Blade3> = HashMap::new();
    let mut twists: HashMap<&str, Turn> = HashMap::new();
    let mut real_twists: HashMap<&str, Turn> = HashMap::new();
    let mut colors: HashMap<String, Color32> = get_default_color_hash();
    let mut compounds: HashMap<&str, Vec<Turn>> = HashMap::new();
    let mut ctx = meval::Context::new();
    let mut twist_orders: HashMap<&str, isize> = HashMap::new();
    for node in doc.nodes() {
        match node.name().value() {
            "name" => puzzle.name = String::from(node.entries().get(0)?.value().as_string()?),
            "author" => puzzle
                .authors
                .push(String::from(node.entries().get(0)?.value().as_string()?)),
            "vars" => {
                for var in node.children()?.nodes() {
                    let val = var.entries().get(0)?.value();
                    let float_val = match val.is_string() {
                        true => meval::eval_str_with_context(val.as_string()?, &ctx).ok()?,
                        false => match val.is_integer() {
                            true => val.as_integer()? as f64,
                            false => val.as_float()?,
                        },
                    };
                    ctx.var(var.name().value(), float_val);
                }
            }
            "circles" => {
                for created_circle in node.children()?.nodes() {
                    circles.insert(
                        created_circle.name().value(),
                        oriented_circle(
                            point(
                                match created_circle.get("x")?.is_string() {
                                    true => meval::eval_str_with_context(
                                        created_circle.get("x")?.as_string()?,
                                        &ctx,
                                    )
                                    .ok()?,
                                    false => match created_circle.get("x")?.is_float() {
                                        true => created_circle.get("x")?.as_float()?,
                                        false => created_circle.get("x")?.as_integer()? as f64,
                                    },
                                },
                                match created_circle.get("y")?.is_string() {
                                    true => meval::eval_str_with_context(
                                        created_circle.get("y")?.as_string()?,
                                        &ctx,
                                    )
                                    .ok()?,
                                    false => match created_circle.get("y")?.is_float() {
                                        true => created_circle.get("y")?.as_float()?,
                                        false => created_circle.get("y")?.as_integer()? as f64,
                                    },
                                },
                            ),
                            match created_circle.get("r")?.is_string() {
                                true => meval::eval_str_with_context(
                                    created_circle.get("r")?.as_string()?,
                                    &ctx,
                                )
                                .ok()?,
                                false => match created_circle.get("r")?.is_float() {
                                    true => created_circle.get("r")?.as_float()?,
                                    false => created_circle.get("r")?.as_integer()? as f64,
                                },
                            },
                            true,
                        ),
                    );
                }
            }
            "base" => {
                let mut disks = Vec::new();
                for disk in node.entries().into_iter() {
                    disks.push(*circles.get(disk.value().as_string()?)?);
                }
                puzzle.pieces = make_basic_puzzle(disks).ok()?;
            }
            "twists" => {
                for turn in node.children()?.nodes() {
                    twists.insert(
                        turn.name().value(),
                        basic_turn(
                            *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        ),
                    );
                    twist_orders.insert(
                        turn.name().value(),
                        turn.entries().get(1)?.value().as_integer()? as isize,
                    );
                    if turn.entries().len() == 2 {
                        real_twists.insert(
                            turn.name().value(),
                            basic_turn(
                                *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                                -2.0 * PI as f64
                                    / (turn.entries().get(1)?.value().as_integer()? as f64),
                            ),
                        );
                    }
                    compounds.insert(
                        turn.name().value(),
                        vec![basic_turn(
                            *circles.get(turn.entries().get(0)?.value().as_string()?)?,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        )],
                    );
                }
            }
            "compounds" => {
                let mut compound_adds: Vec<Vec<Turn>> = vec![Vec::new()];
                let mut extend: Vec<Turn>;
                for compound in node.children()?.nodes() {
                    for val in compound.entries() {
                        match val.value().as_string()?.strip_suffix("'") {
                            None => match strip_number_end(val.value().as_string()?) {
                                None => extend = compounds.get(val.value().as_string()?)?.clone(),
                                Some(real) => {
                                    extend = multiply_turns(
                                        real.1.parse::<isize>().unwrap(),
                                        compounds.get(real.0.as_str())?,
                                    );
                                }
                            },
                            Some(real) => match strip_number_end(real) {
                                None => {
                                    extend = invert_compound_turn(compounds.get(real)?);
                                }
                                Some(inside) => {
                                    extend = invert_compound_turn(&multiply_turns(
                                        inside.1.parse::<isize>().unwrap(),
                                        compounds.get(inside.0.as_str())?,
                                    ));
                                }
                            },
                        }
                        for compound_add in &mut compound_adds {
                            compound_add.extend(extend.clone());
                        }
                    }
                    for compound_add in &compound_adds {
                        compounds.insert(compound.name().value(), compound_add.clone());
                    }
                }
            }

            "cut" => {
                let mut turn_seqs = vec![Vec::new()];
                let mut extend = Vec::new();
                for val in node.entries() {
                    match val.value().as_string()?.strip_suffix("'") {
                        None => match strip_number_end(val.value().as_string()?) {
                            None => match val.value().as_string()?.strip_suffix("*") {
                                None => extend = compounds.get(val.value().as_string()?)?.clone(),
                                Some(real) => {
                                    let turn = *twists.get(real)?;
                                    let number = *twist_orders.get(real)?;
                                    let mut new_adds = Vec::new();
                                    for add in &turn_seqs {
                                        for i in 0..number {
                                            let mut new_add = add.clone();
                                            new_add.push(i * turn);
                                            new_adds.push(new_add);
                                        }
                                    }
                                    turn_seqs.extend(new_adds);
                                }
                            },
                            Some(real) => {
                                extend = multiply_turns(
                                    real.1.parse::<isize>().unwrap(),
                                    compounds.get(real.0.as_str())?,
                                );
                            }
                        },
                        Some(real) => match strip_number_end(real) {
                            None => {
                                extend = invert_compound_turn(compounds.get(real)?);
                            }
                            Some(inside) => {
                                extend = invert_compound_turn(&multiply_turns(
                                    inside.1.parse::<isize>().unwrap(),
                                    compounds.get(inside.0.as_str())?,
                                ))
                            }
                        },
                    }
                    for turns in &mut turn_seqs {
                        turns.extend(extend.clone());
                    }
                }
                for turns in &turn_seqs {
                    puzzle.cut(turns).ok()?;
                    puzzle.pieces.len();
                }
            }
            "colors" => {
                for color in node.children()?.nodes() {
                    colors.insert(
                        color.name().value().to_string(),
                        Color32::from_rgb(
                            color.entries().get(0)?.value().as_integer()? as u8,
                            color.entries().get(1)?.value().as_integer()? as u8,
                            color.entries().get(2)?.value().as_integer()? as u8,
                        ),
                    );
                }
            }
            "color" => {
                let color = *colors.get(node.entries()[0].value().as_string()?)?;
                let mut coloring_circles = Vec::new();
                for i in 1..node.entries().len() {
                    let circle = node.entries().get(i)?.value().as_string()?;
                    coloring_circles.push(match circle.strip_prefix("!") {
                        None => *circles.get(circle)?,
                        Some(real) => -*circles.get(real)?,
                    });
                }
                puzzle.color(&(coloring_circles, color));
            }
            "twist" => {
                let mut sequence = Vec::new();
                for val in node.entries() {
                    let extend;
                    match val.value().as_string()?.strip_suffix("'") {
                        None => match strip_number_end(val.value().as_string()?) {
                            None => {
                                extend = compounds.get(val.value().as_string()?)?.clone();
                            }
                            Some(real) => {
                                extend = multiply_turns(
                                    real.1.parse::<isize>().unwrap(),
                                    compounds.get(real.0.as_str())?,
                                );
                            }
                        },
                        Some(real) => match strip_number_end(real) {
                            None => {
                                extend = invert_compound_turn(compounds.get(real)?);
                            }
                            Some(inside) => {
                                extend = invert_compound_turn(&multiply_turns(
                                    inside.1.parse::<isize>().unwrap(),
                                    compounds.get(inside.0.as_str())?,
                                ))
                            }
                        },
                    }
                    sequence.extend(extend);
                }
                let mut add_seq = Vec::new();
                for turn in &sequence {
                    puzzle.turn(*turn, false).ok()?;
                    add_seq.push(turn.clone());
                }
                def_stack.push(add_seq);
            }
            "undo" => {
                let mut number;
                if node.entries().is_empty() {
                    number = 1;
                } else {
                    let entry = &node.entries().get(0)?;
                    match entry.value().as_integer() {
                        None => {
                            number = -1;
                        }
                        Some(num) => {
                            number = num;
                        }
                    }
                }
                while number != 0 {
                    number -= 1;
                    if let Some(turns) = def_stack.pop() {
                        for turn in invert_compound_turn(&turns) {
                            puzzle.turn(turn, false).ok()?;
                        }
                    } else {
                        break;
                    }
                }
            }
            _ => (),
        }
    }
    for turn in real_twists {
        puzzle.turns.insert(String::from(turn.0), turn.1);
        puzzle
            .turns
            .insert(String::from(turn.0) + "'", turn.1.inverse());
    }
    puzzle.solved_state = puzzle.pieces.clone();
    puzzle.animation_offset = None;
    puzzle.stack = Vec::new();
    return Some(puzzle);
}
