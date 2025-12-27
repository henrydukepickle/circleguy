use std::collections::HashMap;

use egui::Key;
use kdl::KdlDocument;

pub type Keybinds = HashMap<Key, (String, isize)>;
pub type Group = HashMap<Key, isize>;
pub type Groups = HashMap<String, Group>;

///parse a keybind group from a string
fn parse_group_kdl(kdl: &String) -> Option<Groups> {
    let mut groups = HashMap::new();
    let doc: KdlDocument = kdl.parse().ok()?;
    let node = doc.get("groups")?;
    for group_node in node.children()?.nodes() {
        let name = group_node.name().value().to_string();
        let mut group = HashMap::new();
        for entry in group_node.entries() {
            group.insert(
                Key::from_name(entry.name()?.value())?,
                entry.value().as_integer()? as isize,
            );
        }
        groups.insert(name, group);
    }
    Some(groups)
}

///parse keybinds for a puzzle from a string, given the groups
fn parse_keybinds_kdl(kdl: &String, groups: &Groups) -> Option<Keybinds> {
    let mut binds = Keybinds::new();
    let doc: KdlDocument = kdl.parse().ok()?;
    let node = doc.get("bind")?;
    for bind in node.entries() {
        let name = bind.name()?.value();
        let group = bind.value().as_string()?;
        let group_real = groups.get(group)?;
        for (k, v) in group_real {
            binds.insert(*k, (name.to_string(), *v));
        }
    }
    Some(binds)
}

///load the keybinds using the above functions
pub fn load_keybinds(kdl_binds: &String, kdl_groups: &String) -> Option<Keybinds> {
    parse_keybinds_kdl(kdl_binds, &parse_group_kdl(kdl_groups)?)
}
