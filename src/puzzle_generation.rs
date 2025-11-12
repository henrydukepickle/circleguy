use crate::POOL_PRECISION;
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
type Region = Vec<Blade3>;
type Coloring = (Region, Color32);
type Compound = Vec<Turn>;
type List = Vec<Compound>;
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

#[derive(Debug, Clone, Copy)]
enum Commands {
    Name,
    Author,
    Vars,
    Circles,
    Base,
    Regions,
    Twists,
    Compounds,
    List,
    Cut,
    Colors,
    Color,
    Twist,
    Undo,
    Along,
    Foreach,
}

impl Commands {
    fn name(&self) -> String {
        match self {
            Commands::Author => "author".to_string(),
            Commands::Name => "name".to_string(),
            Commands::Vars => "vars".to_string(),
            Commands::Circles => "circles".to_string(),
            Commands::Regions => "regions".to_string(),
            Commands::Base => "base".to_string(),
            Commands::Twists => "twists".to_string(),
            Commands::Compounds => "compounds".to_string(),
            Commands::List => "list".to_string(),
            Commands::Cut => "cut".to_string(),
            Commands::Colors => "colors".to_string(),
            Commands::Color => "color".to_string(),
            Commands::Twist => "twist".to_string(),
            Commands::Undo => "undo".to_string(),
            Commands::Along => "along".to_string(),
            Commands::Foreach => "foreach".to_string(),
        }
    }
}

struct DefData {
    circles: HashMap<String, Blade3>,
    twists: HashMap<String, Turn>,
    real_twists: HashMap<String, Turn>,
    colors: HashMap<String, Color32>,
    compounds: HashMap<String, Compound>,
    regions: HashMap<String, Region>,
    lists: HashMap<String, List>,
    twist_orders: HashMap<String, isize>,
    def_stack: List,
}
impl Default for DefData {
    fn default() -> Self {
        Self {
            circles: HashMap::new(),
            twists: HashMap::new(),
            real_twists: HashMap::new(),
            colors: get_default_color_hash(),
            compounds: HashMap::new(),
            regions: HashMap::new(),
            lists: HashMap::new(),
            twist_orders: HashMap::new(),
            def_stack: Vec::new(),
        }
    }
}

impl PieceShape {
    fn in_region(&self, region: &Region) -> Result<Option<Contains>, String> {
        let mut none = false;
        for circ in region {
            let cont = self.in_circle(*circ)?;
            match cont {
                None => {
                    none = true;
                }
                Some(Contains::Outside) => return Ok(Some(Contains::Outside)),
                _ => {}
            }
        }
        if none {
            Ok(None)
        } else {
            Ok(Some(Contains::Inside))
        }
    }
}

impl Puzzle {
    fn cut(&mut self, cut: &Cut, region: &Region) -> Result<Result<(), bool>, String> {
        let mut p2 = self.clone();
        let mut new_pieces = Vec::new();
        let mut old_pieces = Vec::new();
        for piece in &p2.pieces {
            if piece
                .shape
                .in_region(&region)?
                .ok_or("Puzzle.cut failed: cut region was bandaged!".to_string())?
                == Contains::Inside
            {
                new_pieces.push(piece.clone());
            } else {
                old_pieces.push(piece.clone());
            }
        }
        p2.pieces = new_pieces;
        for turn in cut {
            if let Err(x) = p2.turn(*turn, true)? {
                return Ok(Err(x));
            };
        }
        for turn in cut.clone().into_iter().rev() {
            if let Err(x) = p2.turn(turn.inverse(), false)? {
                return Ok(Err(x));
            };
        }
        self.pieces = p2.pieces;
        self.pieces.extend(old_pieces);
        Ok(Ok(()))
    }
    fn color(&mut self, coloring: &Coloring) -> Result<(), String> {
        //dbg!("HI");
        for piece in &mut self.pieces {
            let mut color = true;
            for circle in coloring.0.clone().into_iter() {
                let contains = piece.shape.in_circle(circle)?;
                if contains != Some(Contains::Inside) {
                    color = false;
                    break;
                }
            }
            if color {
                piece.color = coloring.1;
            }
        }
        Ok(())
    }
    fn compound_turn(
        &mut self,
        compound: &Compound,
        cut: bool,
    ) -> Result<Result<(), bool>, String> {
        for turn in compound {
            if let Err(x) = self.turn(*turn, cut)? {
                return Ok(Err(x));
            };
        }
        Ok(Ok(()))
    }
}
pub fn basic_turn(raw_circle: Blade3, angle: f64) -> Result<Turn, String> {
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
        return Ok(Turn {
            circle: raw_circle.rescale_oriented(),
            rotation: Rotoflector::Rotor(((p1 ^ p3 ^ NI) * (p1 ^ p2 ^ NI)).rescale_oriented()),
        });
    } else {
        return Err("basic_turn failed: A line was passed!".to_string());
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
fn make_basic_puzzle(disks: Vec<Blade3>) -> Result<Result<Vec<Piece>, ()>, String> {
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
            if let Some(x) = &disk_piece.cut_by_circle(*old_disk)?[1] {
                disk_piece = x.clone();
            }
        }
        old_disks.push(*disk);
        pieces.push(disk_piece);
    }
    return Ok(Ok(pieces));
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
            puzzle.turn_id(String::from(turn), false).ok()?.ok()?;
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

fn parse_compound(val: &KdlEntry, compounds: &HashMap<String, Compound>) -> Option<Compound> {
    Some(match val.value().as_string()?.strip_suffix("'") {
        None => match strip_number_end(val.value().as_string()?) {
            None => compounds.get(val.value().as_string()?)?.clone(),
            Some(real) => multiply_turns(
                real.1.parse::<isize>().ok()?,
                compounds.get(real.0.as_str())?,
            ),
        },
        Some(real) => match strip_number_end(real) {
            None => invert_compound_turn(compounds.get(real)?),
            Some(inside) => invert_compound_turn(&multiply_turns(
                inside.1.parse::<isize>().ok()?,
                compounds.get(inside.0.as_str())?,
            )),
        },
    })
}

fn parse_value_as_float(val: &KdlValue, ctx: &meval::Context) -> Result<f64, String> {
    Ok(match val.is_string() {
        true => meval::eval_str_with_context(val.as_string().unwrap(), ctx).or(Err(
            "parse_value_as_float failed: value was a string that could not be parsed by meval!"
                .to_string(),
        ))?,
        false => match val.is_integer() {
            true => val.as_integer().unwrap() as f64,
            false => val.as_float().unwrap(),
        },
    })
}

fn parse_node(
    node: &KdlNode,
    kind: Commands,
    data: &mut DefData,
    puzzle: &mut Puzzle,
    ctx: &mut meval::Context,
) -> Option<()> {
    match kind {
        Commands::Name => puzzle.name = String::from(node.entries().get(0)?.value().as_string()?),
        Commands::Author => puzzle
            .authors
            .push(String::from(node.entries().get(0)?.value().as_string()?)),
        Commands::Vars => {
            for var in node.children()?.nodes() {
                let val = var.entries().get(0)?.value();
                ctx.var(var.name().value(), parse_value_as_float(val, &ctx).ok()?);
            }
        }
        Commands::Circles => {
            for created_circle in node.children()?.nodes() {
                let name = created_circle.name().value();
                let circ = oriented_circle(
                    point(
                        parse_value_as_float(created_circle.get("x")?, &ctx).ok()?,
                        parse_value_as_float(created_circle.get("y")?, &ctx).ok()?,
                    ),
                    parse_value_as_float(created_circle.get("r")?, &ctx).ok()?,
                    true,
                )
                .rescale_oriented();
                let name2 = "!".to_string() + name;
                data.circles.insert(name.to_string(), circ);
                data.regions.insert(name.to_string(), vec![circ]);
                data.regions.insert(name2.to_string(), vec![-circ]);
            }
        }
        Commands::Regions => {
            for created_region in node.children()?.nodes() {
                let mut region = Vec::new();
                let name = created_region.name().value();
                for subregion in node.entries().into_iter() {
                    region.extend(data.regions.get(subregion.value().as_string()?)?);
                }
                data.regions.insert(name.to_string(), region);
            }
        }
        Commands::Base => {
            let mut disks = Vec::new();
            for disk in node.entries().into_iter() {
                disks.push(*data.circles.get(disk.value().as_string()?)?);
            }
            puzzle.pieces = make_basic_puzzle(disks).ok()?.ok()?;
        }
        Commands::Twists => {
            for turn in node.children()?.nodes() {
                data.twists.insert(
                    turn.name().value().to_string(),
                    basic_turn(
                        *data
                            .circles
                            .get(turn.entries().get(0)?.value().as_string()?)?,
                        -2.0 * PI as f64 / (turn.entries().get(1)?.value().as_integer()? as f64),
                    )
                    .ok()?,
                );
                data.twist_orders.insert(
                    turn.name().value().to_string(),
                    turn.entries().get(1)?.value().as_integer()? as isize,
                );
                if turn.entries().len() == 2 {
                    data.real_twists.insert(
                        turn.name().value().to_string(),
                        basic_turn(
                            *data
                                .circles
                                .get(turn.entries().get(0)?.value().as_string()?)?,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        )
                        .ok()?,
                    );
                }
                data.compounds.insert(
                    turn.name().value().to_string(),
                    vec![
                        basic_turn(
                            *data
                                .circles
                                .get(turn.entries().get(0)?.value().as_string()?)?,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        )
                        .ok()?,
                    ],
                );
            }
        }
        Commands::Compounds => {
            let mut compound_adds: Vec<Vec<Turn>> = vec![Vec::new()];
            for compound in node.children()?.nodes() {
                for val in compound.entries() {
                    for compound_add in &mut compound_adds {
                        compound_add.extend(parse_compound(val, &data.compounds)?);
                    }
                }
                for compound_add in &compound_adds {
                    data.compounds
                        .insert(compound.name().value().to_string(), compound_add.clone());
                }
            }
        }
        Commands::List => {
            let mut list = List::new();
            let name = node.entries().get(0)?.value().as_string()?;
            for element in node.children()?.nodes() {
                if element.name().value() != "-" {
                    return None;
                }
                let num = element.entries().get(0)?.value().as_integer()?;
                let mut compound = Compound::new();
                for val in element.entries().into_iter().skip(1) {
                    compound.extend(parse_compound(val, &data.compounds)?);
                }
                for _ in 0..num {
                    list.push(compound.clone());
                }
            }
            data.lists.insert(name.to_string(), list);
        }
        Commands::Cut => {
            let mut turn_seqs = vec![Vec::new()];
            let mut extend = Vec::new();
            let region = match node.get("region") {
                None => Vec::new(),
                Some(x) => data.regions.get(x.as_string()?)?.clone(),
            };
            for val in node.entries() {
                if val.name().is_some() {
                    continue;
                }
                match val.value().as_string()?.strip_suffix("*") {
                    None => extend = parse_compound(val, &data.compounds)?,
                    Some(real) => {
                        let turn = *data.twists.get(real)?;
                        let number = *data.twist_orders.get(real)?;
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
                }
                // match val.value().as_string()?.strip_suffix("'") {
                //     None => match strip_number_end(val.value().as_string()?) {
                //         None => match val.value().as_string()?.strip_suffix("*") {
                //             None => extend = compounds.get(val.value().as_string()?)?.clone(),
                //             Some(real) => {
                //                 let turn = *twists.get(real)?;
                //                 let number = *twist_orders.get(real)?;
                //                 let mut new_adds = Vec::new();
                //                 for add in &turn_seqs {
                //                     for i in 0..number {
                //                         let mut new_add = add.clone();
                //                         new_add.push(i * turn);
                //                         new_adds.push(new_add);
                //                     }
                //                 }
                //                 turn_seqs.extend(new_adds);
                //             }
                //         },
                //         Some(real) => {
                //             extend = multiply_turns(
                //                 real.1.parse::<isize>().ok()?,
                //                 compounds.get(real.0.as_str())?,
                //             );
                //         }
                //     },
                //     Some(real) => match strip_number_end(real) {
                //         None => {
                //             extend = invert_compound_turn(compounds.get(real)?);
                //         }
                //         Some(inside) => {
                //             extend = invert_compound_turn(&multiply_turns(
                //                 inside.1.parse::<isize>().ok()?,
                //                 compounds.get(inside.0.as_str())?,
                //             ))
                //         }
                //     },
                // }
                for turns in &mut turn_seqs {
                    turns.extend(extend.clone());
                }
            }
            for turns in &turn_seqs {
                (puzzle.cut(turns, &region)).ok()?.ok()?;
                // puzzle.pieces.len();
            }
        }
        Commands::Twist => {
            let mut sequence = Vec::new();
            for val in node.entries() {
                let extend = parse_compound(val, &data.compounds)?;
                sequence.extend(extend);
            }
            let mut add_seq = Vec::new();
            for turn in &sequence {
                puzzle.turn(*turn, false).ok()?.ok()?;
                add_seq.push(turn.clone());
            }
            data.def_stack.push(add_seq);
        }
        Commands::Colors => {
            for color in node.children()?.nodes() {
                data.colors.insert(
                    color.name().value().to_string(),
                    Color32::from_rgb(
                        color.entries().get(0)?.value().as_integer()? as u8,
                        color.entries().get(1)?.value().as_integer()? as u8,
                        color.entries().get(2)?.value().as_integer()? as u8,
                    ),
                );
            }
        }
        Commands::Color => {
            //dbg!("TEST?");
            let color = *data.colors.get(node.entries()[0].value().as_string()?)?;
            let mut coloring_circles = Region::new();
            for i in 1..node.entries().len() {
                let circle = node.entries().get(i)?.value().as_string()?;
                coloring_circles.extend(data.regions.get(circle)?.clone());
            }
            puzzle.color(&(coloring_circles, color)).ok()?;
        }
        Commands::Undo => {
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
                if let Some(turns) = data.def_stack.pop() {
                    for turn in invert_compound_turn(&turns) {
                        puzzle.turn(turn, false).ok()?.ok()?;
                    }
                } else {
                    break;
                }
            }
        }
        Commands::Foreach => {
            let list = data
                .lists
                .get(node.entries().get(0)?.value().as_string()?)?
                .clone();
            for comp in &list {
                puzzle.compound_turn(&comp, true).ok()?.ok()?;
                parse_nodes(
                    node.children()?.nodes(),
                    data,
                    puzzle,
                    ctx,
                    &vec![
                        Commands::Twist,
                        Commands::Undo,
                        Commands::Cut,
                        Commands::Color,
                        Commands::Along,
                        Commands::Foreach,
                    ],
                )?;
                puzzle
                    .compound_turn(&invert_compound_turn(&comp), false)
                    .ok()?
                    .ok()?;
            }
        }
        Commands::Along => {
            let list = data
                .lists
                .get(node.entries().get(0)?.value().as_string()?)?
                .clone();
            for comp in &list {
                puzzle.compound_turn(&comp, true).ok()?.ok()?;
                parse_nodes(
                    node.children()?.nodes(),
                    data,
                    puzzle,
                    ctx,
                    &vec![
                        Commands::Twist,
                        Commands::Undo,
                        Commands::Cut,
                        Commands::Color,
                        Commands::Along,
                        Commands::Foreach,
                    ],
                )?;
            }
            for comp in list.clone().into_iter().rev() {
                puzzle
                    .compound_turn(&invert_compound_turn(&comp), false)
                    .ok()?
                    .ok()?;
            }
        }
    }
    Some(())
}

fn parse_nodes(
    nodes: &[KdlNode],
    data: &mut DefData,
    puzzle: &mut Puzzle,
    ctx: &mut meval::Context,
    allowed_commands: &Vec<Commands>,
) -> Option<()> {
    for node in nodes {
        for command in allowed_commands {
            if node.name().value() == command.name() {
                parse_node(node, *command, data, puzzle, ctx)?;
            }
        }
    }
    Some(())
}

pub fn parse_kdl(string: &str) -> Option<Puzzle> {
    let mut puzzle = Puzzle {
        name: String::new(),
        authors: Vec::new(),
        pieces: Vec::new(),
        turns: HashMap::new(),
        stack: Vec::new(),
        animation_offset: None,
        intern_2: approx_collections::FloatPool::new(POOL_PRECISION),
        intern_3: approx_collections::FloatPool::new(POOL_PRECISION),
        depth: 500,
        solved_state: Vec::new(),
        solved: false,
        anim_left: 0.0,
        def: string.to_string(),
    };
    let doc: KdlDocument = match string.parse() {
        Ok(real) => real,
        Err(_err) => return None,
    };
    let mut data = DefData::default();
    // let mut circles: HashMap<String, Blade3> = HashMap::new();
    // let mut twists: HashMap<String, Turn> = HashMap::new();
    // let mut real_twists: HashMap<String, Turn> = HashMap::new();
    // let mut colors: HashMap<String, Color32> = get_default_color_hash();
    // let mut compounds: HashMap<String, Compound> = HashMap::new();
    // let mut regions: HashMap<String, Region> = HashMap::new();
    // let mut lists: HashMap<String, List> = HashMap::new();
    let mut ctx = meval::Context::new();
    let all_commands = vec![
        Commands::Name,
        Commands::Author,
        Commands::Vars,
        Commands::Circles,
        Commands::Regions,
        Commands::Base,
        Commands::Twists,
        Commands::Compounds,
        Commands::List,
        Commands::Cut,
        Commands::Colors,
        Commands::Color,
        Commands::Twist,
        Commands::Undo,
        Commands::Along,
        Commands::Foreach,
    ];
    // let mut twist_orders: HashMap<String, isize> = HashMap::new();
    parse_nodes(doc.nodes(), &mut data, &mut puzzle, &mut ctx, &all_commands)?;
    for turn in data.real_twists {
        puzzle.turns.insert(turn.0.clone(), turn.1);
        puzzle.turns.insert(turn.0.clone() + "'", turn.1.inverse());
    }
    puzzle.solved_state = puzzle.pieces.clone();
    puzzle.animation_offset = None;
    puzzle.stack = Vec::new();
    return Some(puzzle);
}
