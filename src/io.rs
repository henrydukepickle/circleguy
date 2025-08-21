#[cfg(not(target_arch = "wasm32"))]
const DEV: bool = true;

use std::fs::*;
use std::io::Write;
pub fn get_puzzle_string(def: String, stack: &Vec<String>) -> String {
    if stack.is_empty() {
        return def;
    };
    return def + "\n --LOG FILE \n" + &stack.join(",");
}
#[cfg(not(target_arch = "wasm32"))]
pub fn read_file_to_string(path: &String) -> std::io::Result<String> {
    let curr_path = match DEV {
        false => String::from(
            std::env::current_exe()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .split("circleguy.exe")
                .into_iter()
                .collect::<Vec<&str>>()[0],
        ),
        true => String::new(),
    };
    std::fs::read_to_string(curr_path + &path)
}

#[cfg(target_arch = "wasm32")]
pub fn read_file_to_string(path: &str) -> Result<String, &'static str> {
    static PUZZLE_DEFINITIONS: include_dir::Dir<'_> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/Puzzles");
    let path = path.strip_prefix("Puzzles/").unwrap_or(path);
    Ok(PUZZLE_DEFINITIONS
        .get_file(path)
        .ok_or("no such file")?
        .contents_utf8()
        .ok_or("invalid UTF-8")?
        .to_string())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn write_to_file(def: &String, stack: &Vec<String>, path: &str) -> Result<(), std::io::Error> {
    let curr_path = match DEV {
        false => String::from(
            std::env::current_exe()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .split("circleguy.exe")
                .into_iter()
                .collect::<Vec<&str>>()[0],
        ),
        true => String::new(),
    };
    let real_path = curr_path + path;
    let mut buffer = OpenOptions::new()
        .write(true)
        .create(true)
        .open(real_path)?;
    buffer.write_all(get_puzzle_string(def.clone(), stack).as_str().as_bytes())?;
    Ok(())
}
