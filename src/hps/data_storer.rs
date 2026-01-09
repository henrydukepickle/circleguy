use crate::{
    hps::{
        builtins::{circleguy_builtins, loading_builtins},
        custom_values::hpspuzzle::HPSPuzzle,
    },
    puzzle::puzzle::{Puzzle, PuzzleData},
    ui::io::*,
};
use hyperpuzzlescript::{
    BUILTIN_SPAN, CustomValue, EvalCtx, FnValue, FullDiagnostic, List, Map, Runtime, Scope,
    Spanned, builtins::define_base_in,
};
use kdl::{KdlDocument, KdlNode};
use std::{
    collections::HashMap,
    fs::*,
    path::Path,
    sync::{Arc, Mutex},
};
pub const TOP: usize = 3;
// #[derive(Debug, Clone)]
// pub struct UIPuzzleData {
//     pub preview: String,
//     pub data: String,
//     pub keybinds: Option<String>,
//     pub keybind_groups: Option<String>,
// }
pub type PuzzlesMap = Arc<Mutex<HashMap<String, PuzzleLoadingData>>>;

#[derive(Debug)]

pub struct KeybindData {
    pub defaults: HashMap<egui::Key, (String, isize)>,
    pub overrides: HashMap<String, HashMap<egui::Key, (String, isize)>>,
}

impl KeybindData {
    pub fn new() -> Self {
        Self {
            defaults: HashMap::new(),
            overrides: HashMap::new(),
        }
    }
    pub fn load_from_string(data: String) -> Option<Self> {
        fn parse_turn_node(node: &KdlNode) -> Option<(egui::Key, (String, isize))> {
            Some((
                egui::Key::from_name(node.name().value())?,
                (
                    node.entries().get(0)?.value().as_string()?.to_string(),
                    node.entries().get(1)?.value().as_integer()? as isize,
                ),
            ))
        }
        let mut binds = HashMap::new();
        let mut overrides = HashMap::new();
        let kdl = data.parse::<KdlDocument>().ok()?;
        for node in kdl.nodes() {
            match node.name().value() {
                "binds" => {
                    for c in node.children()?.nodes() {
                        let t = parse_turn_node(c)?;
                        binds.insert(t.0, t.1);
                    }
                }
                "override" => {
                    let name = node.entries().get(0)?.value().as_string()?;
                    let mut over = HashMap::new();
                    for c in node.children()?.nodes() {
                        let t = parse_turn_node(c)?;
                        over.insert(t.0, t.1);
                    }
                    overrides.insert(name.to_string(), over);
                }
                _ => {}
            }
        }
        Some(Self {
            defaults: binds,
            overrides,
        })
    }
    pub fn get_keybinds_for_puzzle(&self, name: &str) -> HashMap<egui::Key, (String, isize)> {
        let mut binds = HashMap::new();
        for (k, v) in &self.defaults {
            binds.insert(*k, v.clone());
        }
        if let Some(b) = self.overrides.get(name) {
            for (k, v) in b {
                binds.insert(*k, v.clone());
            }
        }
        binds
    }
}

#[derive(Debug)]
///stores the data for loading puzzles (definitions and basic info for preview)
pub struct DataStorer {
    pub puzzles: PuzzlesMap,
    pub rt: Runtime,
    pub keybinds: KeybindData,
}

#[derive(Debug, Clone)]
pub struct PuzzleLoadingData {
    pub name: String,
    pub authors: Vec<String>,
    pub scramble: usize,
    pub constructor: Spanned<Arc<FnValue>>,
}

impl PuzzleLoadingData {
    pub fn load(
        &self,
        rt: &mut Runtime,
        keybinds: HashMap<egui::Key, (String, isize)>,
    ) -> Result<PuzzleData, FullDiagnostic> {
        let mut scope = Scope::default();
        scope.special.puz = HPSPuzzle::new().clone().at(BUILTIN_SPAN);
        let arc_scope = Arc::new(scope);
        self.constructor.0.call(
            self.constructor.1,
            &mut EvalCtx {
                scope: &arc_scope,
                runtime: rt,
                caller_span: BUILTIN_SPAN,
                exports: &mut None,
                stack_depth: 0,
            },
            List::new(),
            Map::new(),
        )?;
        let mut puz = arc_scope
            .special
            .puz
            .as_ref::<HPSPuzzle>()?
            .0
            .lock()
            .unwrap()
            .to_puzzle_data();
        puz.authors = self.authors.clone();
        puz.name = self.name.clone();
        puz.depth = self.scramble;
        puz.keybinds = keybinds;
        Ok(puz)
    }
}

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

impl DataStorer {
    pub fn new(exp: bool) -> Result<Self, FullDiagnostic> {
        let mut rt = Runtime::new();
        rt.with_builtins(define_base_in)?;
        rt.with_builtins(circleguy_builtins)?;
        let puzzles = HashMap::new();
        let puzzles_arc = Arc::new(Mutex::new(puzzles));
        let mut ds = Self {
            puzzles: puzzles_arc.clone(),
            rt,
            keybinds: KeybindData::new(),
        };
        loading_builtins(&mut ds.rt, puzzles_arc.clone(), exp).unwrap();
        Ok(ds)
    }
    pub fn reset(&mut self, exp: bool) -> Result<(), FullDiagnostic> {
        *self = Self::new(exp)?;
        Ok(())
    }
    pub fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        let paths = read_dir(def_path).or(Err(()))?;
        for path in paths {
            let filename = path.or(Err(()))?.file_name().into_string().or(Err(()))?;
            let data = read_file_to_string(&(String::from(def_path) + (&filename))).or(Err(()))?;
            self.rt.modules.add_file(Path::new(&filename), data);
            self.rt.exec_all_files();
        }
        Ok(())
    }
    pub fn load_keybinds(&mut self, kb_path: &str) -> Result<(), ()> {
        let data = read_file_to_string(kb_path).ok().ok_or(())?;
        self.keybinds = KeybindData::load_from_string(data).ok_or(())?;
        Ok(())
    }
    pub fn load_save(&mut self, path: &str) -> Option<Puzzle> {
        Puzzle::from_io_data(
            PuzzleIOData::from_string(
                read_file_to_string(&format!("Puzzles/Logs/{}.kdl", path)).ok()?,
            )?,
            self,
        )
    }
    pub fn save(&self, path: &str, puzzle: &Puzzle) -> Result<(), String> {
        write_string_to_file(
            &format!("Puzzles/Logs/{}.kdl", path),
            &puzzle.to_io_data().to_string(),
        )
        .ok()
        .ok_or("Error loading file!".to_string())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        self.data = Vec::new();
        static PUZZLE_DEFINITIONS: include_dir::Dir<'_> =
            include_dir::include_dir!("$CARGO_MANIFEST_DIR/Puzzles/Definitions");
        let mut paths = Vec::new();
        for file in PUZZLE_DEFINITIONS.files() {
            paths.push(file.path().to_str().unwrap());
        }
        for path in paths {
            let data = read_file_to_string(&(String::from(def_path) + &path))
                .or(Err(()))
                .unwrap();
            self.data.push((get_preview_string(&data), data));
        }
        self.data.sort_by_key(|a| a.0.clone());
        Ok(())
    }
}
