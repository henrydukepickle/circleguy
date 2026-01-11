use crate::{
    hps::{
        builtins::{circleguy_builtins, circleguy_hps_builtins, loading_builtins},
        custom_values::hpspuzzle::HPSPuzzle,
        data_storer::{
            def_entry::DefEntry, io::*, keybind_data::KeybindData, puzzle_io::PuzzleIOData,
        },
    },
    puzzle::puzzle::{Puzzle, PuzzleData},
};
use hyperpuzzlescript::{
    BUILTIN_SPAN, CustomValue, EvalCtx, FnValue, FullDiagnostic, List, Map, Runtime, Scope, Spanned,
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
pub type PuzzlesMap = Arc<Mutex<DefEntry>>;

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
    pub path: String,
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
            .to_puzzle_data(&self.path);
        puz.authors = self.authors.clone();
        puz.name = self.name.clone();
        puz.depth = self.scramble;
        puz.keybinds = keybinds;
        puz.path = self.path.clone();
        Ok(puz)
    }
}

impl DataStorer {
    pub fn new(exp: bool) -> Result<Self, FullDiagnostic> {
        let mut rt = Runtime::new();
        rt.with_builtins(circleguy_hps_builtins)?;
        rt.with_builtins(circleguy_builtins)?;
        let puzzles = DefEntry::Folder(("Definitions".to_string(), HashMap::new()));
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
        self.rt.modules.add_from_directory(Path::new(def_path));
        self.rt.exec_all_files();
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
