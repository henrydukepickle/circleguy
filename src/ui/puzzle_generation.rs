use crate::POOL_PRECISION;
use crate::complex::arc::*;
use crate::complex::c64::C64;
use crate::complex::c64::Point;
use crate::complex::c64::Scalar;
use crate::complex::complex_circle::Circle;
use crate::complex::complex_circle::Contains;
use crate::complex::complex_circle::OrientedCircle;
use crate::hps::hps::parse_hps;
use crate::puzzle::piece::*;
use crate::puzzle::piece_shape::*;
use crate::puzzle::puzzle::Puzzle;
use crate::puzzle::turn::*;
use crate::ui::io::*;
use approx_collections::FloatPool;
use egui::*;
use kdl::*;
use std::array;
use std::collections::HashMap;
use std::f64::consts::PI;

///the color used for pieces that haven't yet been colored
const NONE_COLOR: Color32 = Color32::GRAY;
///a series of turns that the algorithm should cut along (and then undo)
type Cut = Vec<Turn>;
///a region of space to apply cuts or a coloring to
type Region = Vec<OrientedCircle>;
///a coloring, which is a region along with a color
type Coloring = (Region, Color32);
///a compound of turns
type Compound = Vec<Turn>;
///a list of compounds
type List = Vec<Compound>;
///a getter to get the default colors
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
///the different commands that are used in puzzle generation
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
    Scramble,
    Solve,
    LogFileBreak,
}

impl Commands {
    ///get the string representation of a command for parsing
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
            Commands::Scramble => "scramble".to_string(),
            Commands::Solve => "solve".to_string(),
            Commands::LogFileBreak => "logfilebreak".to_string(),
        }
    }
}

///all the data in a puzzle definition
struct DefData {
    circles: HashMap<String, OrientedCircle>,
    twists: HashMap<String, Turn>,
    real_twists: HashMap<String, Turn>,
    colors: HashMap<String, Color32>,
    compounds: HashMap<String, Compound>,
    regions: HashMap<String, Region>,
    lists: HashMap<String, List>,
    twist_orders: HashMap<String, isize>,
    def_stack: List, //used during the definition for the sake of the 'undo' command
    stack: Vec<(String, isize)>,
    scramble: Option<[String; 500]>,
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
            stack: Vec::new(),
            scramble: None,
        }
    }
}

impl PieceShape {
    ///determines if a pieceshape falls in a region
    fn in_region(&self, region: &Region) -> Result<Option<Contains>, String> {
        //essentially checks if it falls in every circle of the region
        let mut inside = true;
        for circ in region {
            let cont = self.in_circle(circ.circ);
            if cont == None {
                return Ok(None);
            }
            if cont != Some(circ.ori) {
                inside = false;
            }
        }
        if inside {
            Ok(Some(Contains::Inside))
        } else {
            Ok(Some(Contains::Outside))
        }
    }
}

impl Puzzle {
    ///cuts the puzzle according to a cut sequence, then undoes the turns
    ///Ok() means that the cut was completed successfully
    ///Err(e) means that the cutting failed somehow
    ///only pieces within 'region' are cut
    fn cut(&mut self, cut: &Cut, region: &Region) -> Result<(), String> {
        let mut new_pieces = Vec::new();
        let mut old_pieces = Vec::new();
        for piece in &self.pieces {
            //cut each piece
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
        //set the pieces to the pieces within the relevant region
        self.pieces = new_pieces;
        for turn in cut {
            if !self.turn(*turn, true)? {
                return Err(String::from(
                    "Puzzle.cut failed: turn was cut but still bandaged! (1)",
                ));
            };
        } //perform all the cut turns on the relevant pieces
        for turn in cut.clone().into_iter().rev() {
            if !self.turn(turn.inverse(), false)? {
                return Err(String::from(
                    "Puzzle.cut failed: undoing the cut turns ran into bandaging!",
                ));
            };
        } //undo them
        self.pieces.extend(old_pieces); //add back the old pieces
        Ok(())
    }
    ///colors the puzzle according to a coloring
    fn color(&mut self, coloring: &Coloring) -> Result<(), String> {
        for piece in &mut self.pieces {
            if piece.shape.in_region(&coloring.0)? == Some(Contains::Inside)
                || piece.shape.in_region(&coloring.0)? == Some(Contains::Border)
            {
                piece.color = coloring.1; //if the piece falls in the region, color it
            }
        }
        Ok(())
    }
    ///performs a compound turn on the puzzle
    ///'cut' determines whether the turns are cutting or not
    ///Ok(true) means the compound was completed succesfully
    ///Ok(false) means that the puzzle ran into bandaging
    ///Err(e) means that an error was encountered
    fn compound_turn(&mut self, compound: &Compound, cut: bool) -> Result<bool, String> {
        for turn in compound {
            //just do all the turns in the compound
            if !self.turn(*turn, cut)? {
                return Ok(false);
            };
        }
        Ok(true)
    }
}
///makes a basic turn in CGA, given a circle and an angle
pub fn basic_turn(circle: Circle, angle: f64) -> Result<Turn, String> {
    Ok(Turn {
        circle,
        rot: Rot::from_angle(angle),
    })
}
///multiply a compound (by repetition)
fn multiply_turns(a: isize, compound: &Vec<Turn>) -> Vec<Turn> {
    if a < 0 {
        return invert_compound_turn(&multiply_turns(-1 * a, compound)); //invert it to get to the positive case
    } else if a > 0 {
        let mut multiply_turns = multiply_turns(a - 1, compound); //recursive call
        multiply_turns.extend(compound);
        return multiply_turns;
    } else {
        return Vec::new();
    }
}
///invert a compound turn, i.e., invert all the turns and the order
fn invert_compound_turn(compound: &Vec<Turn>) -> Vec<Turn> {
    let mut turns = Vec::new();
    for turn in compound.into_iter().rev() {
        //invert all the turns and the order
        turns.push(turn.inverse());
    }
    return turns;
}
///makes the shape of a basic puzzle from a base
pub fn make_basic_puzzle(disks: Vec<Circle>) -> Result<Result<Vec<Piece>, ()>, String> {
    let mut pieces = Vec::new();
    let mut old_disks = Vec::new();
    for disk in &disks {
        let start = disk.center
            + C64 {
                re: disk.r(),
                im: 0.,
            };
        //for each disk we want to add to the base
        let mut disk_piece = Piece {
            //make a new piece that's just a circle
            shape: PieceShape {
                bounds: vec![OrientedCircle {
                    circ: *disk,
                    ori: Contains::Inside,
                }],
                border: vec![Arc {
                    circle: *disk,
                    start,
                    angle: 2.0 * PI,
                }],
            },
            color: NONE_COLOR,
        };
        for old_disk in &old_disks {
            if let Some(x) = &disk_piece.cut_by_circle(*old_disk) {
                //for each already existing disk, cut it by these disks and take the outside one
                disk_piece = x.clone().1;
            }
        }
        old_disks.push(*disk);
        pieces.push(disk_piece); //because of how the for loop was constructed, this piece is outside of all the other disks
    }
    return Ok(Ok(pieces));
}

///load a puzzle from a file
///
///needs a .kdl at the end, but is a relative path
pub fn load_puzzle_and_def_from_file(path: &str) -> Option<Puzzle> {
    let contents = read_file_to_string(path).ok()?;
    return Some(parse_hps(&contents.clone())?);
}

///strip the number from the end of a string for parsing reasons
fn strip_number_end(str: &str) -> Option<(String, String)> {
    let chars = str.chars();
    let end = chars
        .rev()
        .take_while(|x| x.is_numeric())
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect::<String>(); //iter bs
    return match end.is_empty() {
        true => None,
        false => Some((String::from(str.strip_suffix(&end)?), end)),
    };
}

///parse a compound turn from a string
fn parse_compound(val: &str, compounds: &HashMap<String, Compound>) -> Option<Compound> {
    Some(match val.strip_suffix("'") {
        //check for inverses
        None => match strip_number_end(val) {
            None => compounds.get(val)?.clone(),
            Some(real) => multiply_turns(
                //if theres a number, multiply it by that number
                real.1.parse::<isize>().ok()?,
                compounds.get(real.0.as_str())?,
            ),
        },
        Some(real) => match strip_number_end(real) {
            None => invert_compound_turn(compounds.get(real)?),
            Some(inside) => invert_compound_turn(&multiply_turns(
                //same as above
                inside.1.parse::<isize>().ok()?,
                compounds.get(inside.0.as_str())?,
            )),
        },
    })
}

///use meval to parse a value as a float in the given context
fn parse_value_as_float(val: &KdlValue, ctx: &meval::Context) -> Result<f64, String> {
    Ok(match val.is_string() {
        true => meval::eval_str_with_context(val.as_string().unwrap(), ctx).or(Err(
            "parse_value_as_float failed: value was a string that could not be parsed by meval!"
                .to_string(), //if its a string, use meval to parse it
        ))?,
        false => match val.is_integer() {
            true => val.as_integer().unwrap() as f64,
            false => val.as_float().unwrap(),
        },
    })
}

///parse a single node of a puzzle definition given what kind of command it is
fn parse_node(
    node: &KdlNode,
    kind: Commands,
    data: &mut DefData,
    puzzle: &mut Puzzle,
    ctx: &mut meval::Context,
) -> Option<()> {
    //here we modify data to build the puzzle
    match kind {
        Commands::Name => puzzle.name = String::from(node.entries().get(0)?.value().as_string()?), //set the name
        Commands::Author => puzzle //add the author
            .authors
            .push(String::from(node.entries().get(0)?.value().as_string()?)),
        Commands::Vars => {
            //add the new variables
            for var in node.children()?.nodes() {
                let val = var.entries().get(0)?.value();
                ctx.var(var.name().value(), parse_value_as_float(val, &ctx).ok()?);
            }
        }
        Commands::Circles => {
            //make and add the circles
            for created_circle in node.children()?.nodes() {
                let name = created_circle.name().value();
                let circ = OrientedCircle {
                    circ: Circle {
                        center: Point {
                            re: parse_value_as_float(created_circle.get("x")?, &ctx).ok()?,
                            im: parse_value_as_float(created_circle.get("y")?, &ctx).ok()?,
                        },
                        r_sq: parse_value_as_float(created_circle.get("r")?, &ctx)
                            .ok()?
                            .powi(2),
                    },
                    ori: Contains::Inside,
                };
                let name2 = "!".to_string() + name;
                data.circles.insert(name.to_string(), circ); //add it to the relevant hashmaps for later
                data.regions.insert(name.to_string(), vec![circ]);
                data.regions.insert(name2.to_string(), vec![-circ]);
            }
        }
        Commands::Regions => {
            //add the new regions
            for created_region in node.children()?.nodes() {
                let mut region = Vec::new();
                let name = created_region.name().value();
                for subregion in node.entries().into_iter() {
                    //parse the regions recursively so we can define regions in terms of other regions
                    region.extend(data.regions.get(subregion.value().as_string()?)?);
                }
                data.regions.insert(name.to_string(), region);
            }
        }
        Commands::Base => {
            //create the base of the puzzle using make_basic_puzzle
            let mut disks = Vec::new();
            for disk in node.entries().into_iter() {
                disks.push(*data.circles.get(disk.value().as_string()?)?);
            }
            puzzle.pieces = make_basic_puzzle(disks.iter().map(|x| x.circ).collect())
                .ok()?
                .ok()?;
        }
        Commands::Twists => {
            //add the new twists
            for turn in node.children()?.nodes() {
                data.twists.insert(
                    turn.name().value().to_string(),
                    basic_turn(
                        data.circles
                            .get(turn.entries().get(0)?.value().as_string()?)?
                            .circ,
                        -2.0 * PI as f64 / (turn.entries().get(1)?.value().as_integer()? as f64), //make the basic turn
                    )
                    .ok()?,
                );
                data.twist_orders.insert(
                    turn.name().value().to_string(),
                    turn.entries().get(1)?.value().as_integer()? as isize, //add the twist order for the * command
                );
                if turn.entries().len() == 2 {
                    //so that excluded ! turns are skipped
                    data.real_twists.insert(
                        turn.name().value().to_string(),
                        basic_turn(
                            data.circles
                                .get(turn.entries().get(0)?.value().as_string()?)?
                                .circ,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        )
                        .ok()?, //add it to the relevant hashmaps
                    );
                }
                data.compounds.insert(
                    turn.name().value().to_string(), //add the inverse of turn as a basic (1-turn) compound
                    vec![
                        basic_turn(
                            data.circles
                                .get(turn.entries().get(0)?.value().as_string()?)?
                                .circ,
                            -2.0 * PI as f64
                                / (turn.entries().get(1)?.value().as_integer()? as f64),
                        )
                        .ok()?,
                    ],
                );
            }
        }
        Commands::Compounds => {
            //parse the new compounds
            let mut compound_adds: Vec<Vec<Turn>> = vec![Vec::new()];
            for compound in node.children()?.nodes() {
                for val in compound.entries() {
                    for compound_add in &mut compound_adds {
                        compound_add
                            .extend(parse_compound(val.value().as_string()?, &data.compounds)?); //recursively read it in terms of other compounds
                    }
                }
                for compound_add in &compound_adds {
                    //add it to data
                    data.compounds
                        .insert(compound.name().value().to_string(), compound_add.clone());
                }
            }
        }
        Commands::List => {
            //make the lists
            let mut list = List::new();
            let name = node.entries().get(0)?.value().as_string()?;
            for element in node.children()?.nodes() {
                if element.name().value() != "-" {
                    //very temporary syntax to parse each line
                    return None;
                }
                let num = element.entries().get(0)?.value().as_integer()?;
                let mut compound = Compound::new();
                for val in element.entries().into_iter().skip(1) {
                    compound.extend(parse_compound(val.value().as_string()?, &data.compounds)?);
                }
                for _ in 0..num {
                    list.push(compound.clone()); //allow easily adding the same compound to a list repeatedly
                }
            }
            data.lists.insert(name.to_string(), list); //add it to data.lists
        }
        Commands::Cut => {
            //cut the puzzle
            let mut turn_seqs = vec![Vec::new()];
            let mut extend = Vec::new();
            let region = match node.get("region") {
                //get the region
                None => Vec::new(),
                Some(x) => data.regions.get(x.as_string()?)?.clone(),
            };
            for val in node.entries() {
                if val.name().is_some() {
                    continue;
                }
                match val.value().as_string()?.strip_suffix("*") {
                    None => extend = parse_compound(val.value().as_string()?, &data.compounds)?, //in this case just parse the compound
                    Some(real) => {
                        //in this case we create multiple turn sequences
                        let turn = *data.twists.get(real)?;
                        let number = *data.twist_orders.get(real)?;
                        let mut new_adds = Vec::new();
                        for add in &turn_seqs {
                            for i in 0..number {
                                //we use the order of the turn here
                                let mut new_add = add.clone();
                                new_add.push(turn.mult(i as Scalar)); //for each power of the turn, we add it in the middle of the turn sequence
                                new_adds.push(new_add);
                            }
                        }
                        turn_seqs.extend(new_adds); //add them all to turn_seqs
                    }
                }
                for turns in &mut turn_seqs {
                    turns.extend(extend.clone()); //add the turns to turns
                }
            }
            for turns in &turn_seqs {
                (puzzle.cut(turns, &region)).ok()?; //execute the cuts
            }
        }
        Commands::Twist => {
            //twist the puzzle
            let mut sequence = Vec::new();
            for val in node.entries() {
                let extend = parse_compound(val.value().as_string()?, &data.compounds)?; //parse the compounds
                sequence.extend(extend);
            }
            let mut add_seq = Vec::new();
            for turn in &sequence {
                //execute the turns
                puzzle.turn(*turn, false).ok()?;
                add_seq.push(turn.clone());
            }
            data.def_stack.push(add_seq);
        }
        Commands::Colors => {
            //define the new colors. this does overwrite defaults
            for color in node.children()?.nodes() {
                data.colors.insert(
                    color.name().value().to_string(),
                    Color32::from_rgb(
                        color.entries().get(0)?.value().as_integer()? as u8, //parse the rgb
                        color.entries().get(1)?.value().as_integer()? as u8,
                        color.entries().get(2)?.value().as_integer()? as u8,
                    ),
                );
            }
        }
        Commands::Color => {
            //color the region
            let color = *data.colors.get(node.entries()[0].value().as_string()?)?;
            let mut coloring_circles = Region::new();
            for i in 1..node.entries().len() {
                let circle = node.entries().get(i)?.value().as_string()?;
                coloring_circles.extend(data.regions.get(circle)?.clone()); //parse the region
            }
            puzzle.color(&(coloring_circles, color)).ok()?; //execute the color command
        }
        Commands::Undo => {
            //undo the relevant turns from data.def_stack
            let mut number;
            if node.entries().is_empty() {
                //if there are no entries, just undo 1 turn
                number = 1;
            } else {
                let entry = &node.entries().get(0)?;
                match entry.value().as_integer() {
                    None => {
                        //if its not a number, undo everything
                        number = -1;
                    }
                    Some(num) => {
                        //otherwise undo the number
                        number = num;
                    }
                }
            }
            while number != 0 {
                //undo the turns
                number -= 1;
                if let Some(turns) = data.def_stack.pop() {
                    for turn in invert_compound_turn(&turns) {
                        puzzle.turn(turn, false).ok()?;
                    }
                } else {
                    break;
                }
            }
        }
        Commands::Foreach => {
            //very temporary syntax not used in any current puzzle definitions. not worth commenting, will probably be removed/seriously modified
            let list = data
                .lists
                .get(node.entries().get(0)?.value().as_string()?)?
                .clone();
            for comp in &list {
                puzzle.compound_turn(&comp, true).ok()?;
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
                    .ok()?;
            }
        }
        Commands::Along => {
            //very temporary syntax not used in any current puzzle definitions. not worth commenting, will probably be removed/seriously modified
            let list = data
                .lists
                .get(node.entries().get(0)?.value().as_string()?)?
                .clone();
            for comp in &list {
                puzzle.compound_turn(&comp, true).ok()?;
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
                    .ok()?;
            }
        }
        Commands::Solve => {
            //stores the solve sequence for the puzzle. ideally only generated by the program
            if puzzle.solved_state.is_none() {
                //if these are the first real turns done on the puzzle, set the solved state first
                puzzle.solved_state = Some(puzzle.pieces.clone())
            }
            if node.entries().get(0)?.value().as_string()?.is_empty() {
                return Some(());
            }
            for val in node.entries().get(0)?.value().as_string()?.split(",") {
                //build the turn sequence
                let (twist, num) = match val.strip_suffix("'") {
                    None => match strip_number_end(&val) {
                        None => (val.to_string(), 1),
                        Some((a, b)) => (a, b.parse::<isize>().ok()?),
                    },
                    Some(x) => match strip_number_end(&x) {
                        None => (x.to_string(), -1),
                        Some((a, b)) => (a, -1 * b.parse::<isize>().ok()?),
                    },
                };
                data.stack.push((twist.clone(), num).clone()); //add these turns to data.stack, *not* data.def_stack
                puzzle
                    .turn(data.twists.get(&twist)?.mult(num as f64), false)
                    .ok()?;
            }
        }
        Commands::Scramble => {
            //stores the scramble for the puzzle
            if puzzle.solved_state.is_none() {
                //if these are the first real turns done on the puzzle, set the solved state first
                puzzle.solved_state = Some(puzzle.pieces.clone())
            }
            if node.entries().get(0)?.value().as_string()?.is_empty() {
                return Some(());
            }
            let mut scramb = array::from_fn(|_| "".to_string()); //make a new array
            let vals = node
                .entries()
                .get(0)?
                .value()
                .as_string()?
                .split(",")
                .map(|x| x.to_string())
                .collect::<Vec<String>>(); //parse into a bunch of turn id's
            if vals.is_empty() {
                return Some(());
            }
            let mut sequence = Vec::new();
            for i in 0..500 {
                let val = vals.get(i)?;
                let extend = parse_compound(val, &data.compounds)?;
                sequence.extend(extend);
                scramb[i] = val.clone(); //populate the scramble sequence
            }
            for turn in &sequence {
                puzzle.turn(*turn, false).ok()?; //execute the turns without adding them to the stack
            }
            data.scramble = Some(scramb);
        }
        Commands::LogFileBreak => {}
    }
    Some(())
}

///parses all the nodes in a document using parse_node
fn parse_nodes(
    nodes: &[KdlNode],
    data: &mut DefData,
    puzzle: &mut Puzzle,
    ctx: &mut meval::Context,
    allowed_commands: &Vec<Commands>,
) -> Option<()> {
    for node in nodes {
        //just iterate over the nodes and parse them all
        for command in allowed_commands {
            if node.name().value() == command.name() {
                parse_node(node, *command, data, puzzle, ctx)?;
            }
        }
    }
    Some(())
}

fn remove_solve_scramble(str: &str) -> String {
    if let Some((x, _)) = str.split_once("\nlogfilebreak") {
        x.to_string()
    } else {
        str.to_string()
    }
}
///parses a kdl document to give a puzzle using parse_nodes
pub fn parse_kdl(string: &str) -> Option<Puzzle> {
    let mut puzzle = Puzzle {
        name: String::new(),
        authors: Vec::new(),
        pieces: Vec::new(),
        base_turns: HashMap::new(),
        turn_orders: HashMap::new(),
        stack: Vec::new(),
        scramble: None,
        animation_offset: None,
        intern: FloatPool::new(POOL_PRECISION),
        depth: 500,
        solved_state: None,
        solved: false,
        anim_left: 0.0,
        def: remove_solve_scramble(string).to_string(),
    }; //initialize a new puzzle
    let doc: KdlDocument = match string.parse() {
        Ok(real) => real,
        Err(_err) => return None,
    }; //parse the document to kdl
    let mut data = DefData::default();
    let mut ctx = meval::Context::new(); //make a new meval context
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
        Commands::Scramble,
        Commands::Solve,
        Commands::LogFileBreak,
    ]; //make a list of all the commands, so we can find which command to use by iterating through
    parse_nodes(doc.nodes(), &mut data, &mut puzzle, &mut ctx, &all_commands)?; //parse the nodes
    for turn in data.real_twists {
        //add the turns to the puzzle. the 'real twists' are the ones that should be doable on the puzzle (not SYM)
        puzzle.base_turns.insert(turn.0.clone(), turn.1);
        puzzle
            .turn_orders
            .insert(turn.0.clone(), *data.twist_orders.get(&turn.0)?);
    }
    if puzzle.solved_state.is_none() {
        puzzle.solved_state = Some(puzzle.pieces.clone());
    } //set some values of the puzzle based on data and the puzzles current state
    puzzle.animation_offset = None;
    puzzle.stack = data.stack;
    puzzle.scramble = data.scramble;
    return Some(puzzle);
}
