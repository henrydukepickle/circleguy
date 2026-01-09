use kdl::KdlDocument;

use crate::{hps::data_storer::data_storer::DataStorer, puzzle::puzzle::Puzzle};

pub struct PuzzleIOData {
    pub name: String,
    pub scramble: Option<Vec<String>>,
    pub stack: Vec<(String, isize)>,
}

impl Puzzle {
    pub fn to_io_data(&self) -> PuzzleIOData {
        PuzzleIOData {
            name: self.name.clone(),
            scramble: self.scramble.clone(),
            stack: self.stack.clone(),
        }
    }
    pub fn from_io_data(data: PuzzleIOData, ds: &mut DataStorer) -> Option<Puzzle> {
        let mut p = Puzzle::new(
            ds.puzzles
                .lock()
                .unwrap()
                .get(&data.name)?
                .load(&mut ds.rt, ds.keybinds.get_keybinds_for_puzzle(&data.name))
                .ok()?,
        );
        if let Some(scramb) = &data.scramble {
            for s in scramb {
                p.turn_id(&s, false, 1).ok()?;
            }
        }
        p.stack = Vec::new();
        for (s, m) in data.stack {
            p.turn_id(&s, false, m).ok()?;
        }
        p.scramble = data.scramble;
        p.animation_offset = None;
        p.anim_left = 0.0;
        Some(p)
    }
}

impl PuzzleIOData {
    pub fn to_string(&self) -> String {
        let mut string = String::new();
        string += &format!("name \"{}\"\n", self.name);
        if let Some(s) = &self.scramble {
            string += "scramble {\n";
            for t in s {
                string += &format!("\tturn \"{}\"\n", t);
            }
            string += "}\n";
        }
        string += "solve {\n";
        for (t, m) in &self.stack {
            string += &format!("\tturn \"{}\" {}\n", t, m)
        }
        string += "}";
        string
    }
    pub fn from_string(string: String) -> Option<Self> {
        let kdl = string.parse::<KdlDocument>().ok()?;
        Some(Self {
            name: kdl
                .get("name")?
                .entries()
                .get(0)?
                .value()
                .as_string()?
                .to_string(),
            scramble: if let Some(node) = kdl.get("scramble") {
                let mut scramb = Vec::new();
                for c in node.children()?.nodes() {
                    scramb.push(c.entries().get(0)?.value().as_string()?.to_string());
                }
                Some(scramb)
            } else {
                None
            },
            stack: {
                let mut stack = Vec::new();
                for c in kdl.get("solve")?.children()?.nodes() {
                    stack.push((
                        c.entries().get(0)?.value().as_string()?.to_string(),
                        c.entries().get(1)?.value().as_integer()? as isize,
                    ));
                }
                stack
            },
        })
    }
}
