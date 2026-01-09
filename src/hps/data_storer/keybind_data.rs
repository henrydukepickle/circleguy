use std::collections::HashMap;

use kdl::{KdlDocument, KdlNode};

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
