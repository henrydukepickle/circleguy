use std::collections::HashMap;

use egui::Key;

pub type Keybinds = HashMap<Key, String>;
pub type Group = HashMap<Key, usize>;
