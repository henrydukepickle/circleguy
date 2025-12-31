use crate::ui::io::*;
use kdl::*;
use std::{cmp::Ordering, collections::HashMap, fs::*};
pub const TOP: usize = 3;
#[derive(Debug, Clone)]
pub struct PuzzleData {
    pub preview: String,
    pub data: String,
    pub keybinds: Option<String>,
    pub keybind_groups: Option<String>,
}
#[derive(Debug, Clone)]
///stores the data for loading puzzles (definitions and basic info for preview)
pub struct DataStorer {
    pub puzzles: HashMap<String, PuzzleData>,
    pub prev_data: Vec<PuzzlePrevData>,
    pub top: Vec<(String, usize)>,
    pub sorted_puzzles: Vec<PuzzleData>,
}
#[derive(Debug, Clone)]
///puzzle preview data
pub struct PuzzlePrevData {
    name: String,
    turns: Vec<String>,
    author: String,
}
///parses the preview data for the puzzle, and puts it in a string
fn get_preview_string(data: &String) -> Option<String> {
    let data = prev_parse_kdl(data.as_str())?;
    Some(data.name + ": " + &data.turns.join(",")) //make the string
}
///parses the preview data for the puzzle from the kdl
fn prev_parse_kdl(string: &str) -> Option<PuzzlePrevData> {
    let mut data = PuzzlePrevData {
        name: String::new(),
        turns: Vec::new(),
        author: String::new(),
    }; //initialize a new data
    let mut numbers = Vec::new(); //turn orders
    let doc: KdlDocument = string.parse().ok()?; //the kdl doc
    for node in doc.nodes() {
        //iterate over nodes
        match node.name().value() {
            //we only check 3 types of commands here; the rest are irrelevant
            "name" => {
                data.name = String::from(node.entries().get(0)?.value().as_string()?);
            }
            "twists" => {
                for twist in node.children()?.nodes() {
                    if twist.entries().len() == 2 {
                        //so that we only check accessible turns (not SYM, for instance)
                        numbers.push(twist.entries().get(1)?.value().as_integer()?) //add the turn order
                    }
                }
            }
            "author" => {
                data.author = String::from(node.entries().get(0)?.value().as_string()?);
            }
            _ => {}
        }
    }
    numbers.sort();
    numbers.reverse(); //sort the numbers in descending order for consistency
    for turn in &numbers {
        data.turns.push(turn.to_string());
    }
    return Some(data);
}
impl DataStorer {
    #[cfg(not(target_arch = "wasm32"))]
    ///load the puzzle definitions into the DataStorer
    pub fn load_puzzles(
        &mut self,
        def_path: &str,
        kb_path: &str,
        kb_group_path: &str,
    ) -> Result<(), ()> {
        self.puzzles = HashMap::new();
        self.sorted_puzzles = Vec::new();
        let paths = read_dir(def_path).or(Err(()))?.into_iter(); //get the paths to puzzles
        for path in paths {
            let filename = path.or(Err(()))?.file_name().into_string().or(Err(()))?;
            let data = read_file_to_string(&(String::from(def_path) + (&filename))).or(Err(()))?; //get the data from the puzzle
            let keybind_data = read_file_to_string(&(String::from(kb_path) + (&filename))).ok(); //read the keybind data
            let puzzle_data = PuzzleData {
                preview: if let Some(x) = get_preview_string(&data) {
                    x
                } else {
                    filename.clone()
                },
                data: data.clone(),
                keybinds: keybind_data,
                keybind_groups: read_file_to_string(&kb_group_path.to_string()).ok(),
            }; //parse the data and push it to the DataStorer
            //self.prev_data.push(prev_parse_kdl(&data).ok_or(())?); //also add the data not in string format
            self.puzzles.insert(filename, puzzle_data.clone());
            self.sorted_puzzles.push(puzzle_data);
        } //sort the puzzle alphabetically by name
        self.sorted_puzzles.sort_by_key(|x| x.preview.clone());
        Ok(())
    }
    ///gets the top authors of puzzles, for fun
    pub fn get_top_authors<const N: usize>(&self) -> Result<[(String, usize); N], ()> {
        let mut authors: HashMap<String, usize> = HashMap::new();
        for p in &self.prev_data {
            //get the authors by iterating through the loaded puzzles
            let a = p.author.clone();
            if authors.contains_key(&a) {
                //add 1 to the number of puzzles the author has made if they already exist
                *authors.get_mut(&a).ok_or(())? += 1;
            } else {
                authors.insert(a, 1); //otherwise initialize them with 1 puzzle
            }
        }
        let mut top = authors.into_iter().collect::<Vec<(String, usize)>>(); //collect into an iter
        top.sort_by(|x, y| {
            //sort it by number of puzzles made in descending order. break ties by name (unfortunate but the most practical)
            let c = y.1.cmp(&x.1);
            if c == Ordering::Equal {
                y.0.cmp(&x.0)
            } else {
                c
            }
        });
        top.first_chunk::<N>().ok_or(()).cloned() //take the first N of them
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
