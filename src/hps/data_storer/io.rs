#[cfg(not(target_arch = "wasm32"))]
const DEV: bool = false;

#[cfg(not(target_arch = "wasm32"))]
pub fn write_string_to_file(path: &str, data: &str) -> Result<(), std::io::Error> {
    use std::fs;

    let curr_path = match DEV {
        //where the path is depends on if the program is being compiled or run in an EXE removed from the original folder. the DEV constant handles this
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
    let real_path = curr_path + path; //add the path to the base path
    fs::write(real_path, data.as_bytes())?;
    Ok(())
}
#[cfg(not(target_arch = "wasm32"))]
///read a file to a string for loading purposes
pub fn read_file_to_string(path: &str) -> std::io::Result<String> {
    let curr_path = match DEV {
        //where the path is depends on if the program is being compiled or run in an EXE removed from the original folder. the DEV constant handles this
        false => String::from(
            std::env::current_exe()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .split("circleguy.exe")
                .collect::<Vec<&str>>()[0],
        ),
        true => String::new(),
    };
    std::fs::read_to_string(curr_path + path) //read to a string
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
