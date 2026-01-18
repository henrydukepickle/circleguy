use std::collections::HashMap;

use crate::hps::data_storer::data_storer::PuzzleLoadingData;

#[derive(Clone, Debug)]
pub enum DefEntry {
    Def(PuzzleLoadingData),
    Folder((String, HashMap<String, DefEntry>)),
}

impl DefEntry {
    pub fn add_puzzle(&mut self, puzzle: PuzzleLoadingData, path: &str) -> Result<(), ()> {
        if let Self::Folder(ref mut curr) = *self {
            let mut curr = curr;
            let mut list_path = path.split("\\").collect::<Vec<&str>>();
            list_path.pop();
            for fold in list_path {
                if !curr.1.contains_key(fold) {
                    curr.1.insert(
                        fold.to_string(),
                        Self::Folder((fold.to_string(), HashMap::new())),
                    );
                }
                curr = if let Self::Folder(x) = curr.1.get_mut(fold).ok_or(())? {
                    x
                } else {
                    return Err(());
                };
            }
            curr.1.insert(puzzle.name.clone(), DefEntry::Def(puzzle));
        } else {
            return Err(());
        }
        Ok(())
    }
    pub fn get(&self, path: &str) -> Option<PuzzleLoadingData> {
        if let Self::Folder(ref curr) = *self {
            let mut curr = curr;
            let mut list_path = path.split("\\").collect::<Vec<&str>>();
            let path_end = list_path.pop()?;
            for fold in list_path {
                curr = if let Self::Folder(x) = curr.1.get(fold)? {
                    x
                } else {
                    return None;
                };
            }
            Some(if let DefEntry::Def(x) = curr.1.get(path_end)? {
                x.clone()
            } else {
                return None;
            })
        } else {
            None
        }
    }
}
