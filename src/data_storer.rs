use crate::io::*;
use kdl::*;
use std::fs::*;
#[derive(Debug, Clone)]
pub struct DataStorer {
    pub data: Vec<(String, String)>, //puzzle preview string, puzzle data string
}
#[derive(Debug, Clone)]
struct PuzzlePrevData {
    name: String,
    turns: Vec<String>,
}
fn get_preview_string(data: &String) -> String {
    let data = match prev_parse_kdl(data.as_str()) {
        None => return String::from("Could not parse preview!"),
        Some(real) => real,
    };
    return data.name + ": " + &data.turns.join(",");
}

fn prev_parse_kdl(string: &str) -> Option<PuzzlePrevData> {
    let mut data = PuzzlePrevData {
        name: String::new(),
        turns: Vec::new(),
    };
    let mut numbers = Vec::new();
    let doc: KdlDocument = string.parse().ok()?;
    for node in doc.nodes() {
        match node.name().value() {
            "name" => {
                data.name = String::from(node.entries().get(0)?.value().as_string()?);
            }
            "twists" => {
                for twist in node.children()?.nodes() {
                    if twist.entries().len() == 2 {
                        numbers.push(twist.entries().get(1)?.value().as_integer()?)
                    }
                }
            }
            _ => {}
        }
    }
    numbers.sort();
    numbers.reverse();
    for turn in &numbers {
        data.turns.push(turn.to_string());
    }
    return Some(data);
}
impl DataStorer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        self.data = Vec::new();
        let paths = read_dir(def_path).or(Err(())).unwrap().into_iter();
        for path in paths {
            let data = read_file_to_string(
                &(String::from(def_path)
                    + (&path
                        .or(Err(()))
                        .unwrap()
                        .file_name()
                        .into_string()
                        .or(Err(()))
                        .unwrap())),
            )
            .or(Err(()))
            .unwrap();
            self.data.push((get_preview_string(&data), data));
        }
        self.data.sort_by_key(|a| a.0.clone());
        Ok(())
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
