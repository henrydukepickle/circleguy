use crate::{
    hps::{
        builtins::{circleguy_builtins, loading_builtins},
        custom_values::{hpspuzzle::HPSPuzzle, hpspuzzledata::HPSPuzzleData},
    },
    puzzle::puzzle::PuzzleData,
    ui::io::*,
};
use hyperpuzzlescript::{
    BUILTIN_SPAN, CustomValue, EvalCtx, FnValue, FullDiagnostic, List, Map, Runtime, Scope,
    Spanned, builtins::define_base_in,
};
use kdl::*;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::*,
    path::{Path, PathBuf},
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
///stores the data for loading puzzles (definitions and basic info for preview)
pub struct DataStorer {
    pub puzzles: PuzzlesMap,
    pub top: Vec<(String, usize)>,
    pub rt: Runtime,
}

#[derive(Debug, Clone)]
pub struct PuzzleLoadingData {
    pub name: String,
    pub authors: Vec<String>,
    pub scramble: usize,
    pub constructor: Spanned<Arc<FnValue>>,
}

impl PuzzleLoadingData {
    pub fn load(&self, rt: &mut Runtime) -> Result<PuzzleData, FullDiagnostic> {
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
        puz.scramble = self.scramble;
        Ok(puz)
    }
}
// ///parses the preview data for the puzzle, and puts it in a string
// fn get_preview_string(data: &String) -> Option<String> {
//     let data = prev_parse_kdl(data.as_str())?;
//     Some(data.name + ": " + &data.turns.join(",")) //make the string
// }
// ///parses the preview data for the puzzle from the kdl
// fn prev_parse_kdl(string: &str) -> Option<PuzzlePrevData> {
//     let mut data = PuzzlePrevData {
//         name: String::new(),
//         turns: Vec::new(),
//         author: String::new(),
//     }; //initialize a new data
//     let mut numbers = Vec::new(); //turn orders
//     let doc: KdlDocument = string.parse().ok()?; //the kdl doc
//     for node in doc.nodes() {
//         //iterate over nodes
//         match node.name().value() {
//             //we only check 3 types of commands here; the rest are irrelevant
//             "name" => {
//                 data.name = String::from(node.entries().get(0)?.value().as_string()?);
//             }
//             "twists" => {
//                 for twist in node.children()?.nodes() {
//                     if twist.entries().len() == 2 {
//                         //so that we only check accessible turns (not SYM, for instance)
//                         numbers.push(twist.entries().get(1)?.value().as_integer()?) //add the turn order
//                     }
//                 }
//             }
//             "author" => {
//                 data.author = String::from(node.entries().get(0)?.value().as_string()?);
//             }
//             _ => {}
//         }
//     }
//     numbers.sort();
//     numbers.reverse(); //sort the numbers in descending order for consistency
//     for turn in &numbers {
//         data.turns.push(turn.to_string());
//     }
//     return Some(data);
// }

impl DataStorer {
    pub fn new() -> Self {
        let mut rt = Runtime::new();
        rt.with_builtins(define_base_in).unwrap();
        rt.with_builtins(circleguy_builtins).unwrap();
        let mut puzzles = HashMap::new();
        let puzzles_arc = Arc::new(Mutex::new(puzzles));
        let mut ds = Self {
            puzzles: puzzles_arc.clone(),
            top: Vec::new(),
            rt,
        };
        loading_builtins(&mut ds.rt, puzzles_arc.clone()).unwrap();
        ds
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    // #[cfg(not(target_arch = "wasm32"))]
    // ///load the puzzle definitions into the DataStorer
    // pub fn load_puzzles(
    //     &mut self,
    //     def_path: &str,
    //     kb_path: &str,
    //     kb_group_path: &str,
    // ) -> Result<(), ()> {
    //     self.puzzles = HashMap::new();
    //     self.sorted_puzzles = Vec::new();
    //     let paths = read_dir(def_path).or(Err(()))?.into_iter(); //get the paths to puzzles
    //     for path in paths {
    //         let filename = path.or(Err(()))?.file_name().into_string().or(Err(()))?;
    //         let data = read_file_to_string(&(String::from(def_path) + (&filename))).or(Err(()))?; //get the data from the puzzle
    //         let keybind_data = read_file_to_string(&(String::from(kb_path) + (&filename))).ok(); //read the keybind data
    //         let puzzle_data = UIPuzzleData {
    //             preview: if let Some(x) = get_preview_string(&data) {
    //                 x
    //             } else {
    //                 filename.clone()
    //             },
    //             data: data.clone(),
    //             keybinds: keybind_data,
    //             keybind_groups: read_file_to_string(&kb_group_path.to_string()).ok(),
    //         }; //parse the data and push it to the DataStorer
    //         //self.prev_data.push(prev_parse_kdl(&data).ok_or(())?); //also add the data not in string format
    //         self.puzzles.insert(filename, puzzle_data.clone());
    //         self.sorted_puzzles.push(puzzle_data);
    //     } //sort the puzzle alphabetically by name
    //     self.sorted_puzzles.sort_by_key(|x| x.preview.clone());
    //     Ok(())
    // }
    pub fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        let paths = read_dir(def_path).or(Err(()))?.into_iter();
        for path in paths {
            let filename = path.or(Err(()))?.file_name().into_string().or(Err(()))?;
            let data = read_file_to_string(&(String::from(def_path) + (&filename))).or(Err(()))?;
            self.rt.modules.add_file(&Path::new(&filename), data);
            self.rt.exec_all_files();
        }
        Ok(())
    }
    ///gets the top authors of puzzles, for fun
    // pub fn get_top_authors<const N: usize>(&self) -> Result<[(String, usize); N], ()> {
    //     let mut authors: HashMap<String, usize> = HashMap::new();
    //     for p in &self.prev_data {
    //         //get the authors by iterating through the loaded puzzles
    //         let a = p.author.clone();
    //         if authors.contains_key(&a) {
    //             //add 1 to the number of puzzles the author has made if they already exist
    //             *authors.get_mut(&a).ok_or(())? += 1;
    //         } else {
    //             authors.insert(a, 1); //otherwise initialize them with 1 puzzle
    //         }
    //     }
    //     let mut top = authors.into_iter().collect::<Vec<(String, usize)>>(); //collect into an iter
    //     top.sort_by(|x, y| {
    //         //sort it by number of puzzles made in descending order. break ties by name (unfortunate but the most practical)
    //         let c = y.1.cmp(&x.1);
    //         if c == Ordering::Equal {
    //             y.0.cmp(&x.0)
    //         } else {
    //             c
    //         }
    //     });
    //     top.first_chunk::<N>().ok_or(()).cloned() //take the first N of them
    // }
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
