use std::env;
use std::io;
use std::path::{PathBuf};

pub fn find_app_root() -> io::Result<(PathBuf, PathBuf)> {
    let mut dir = env::current_dir()?;
    let original = dir.clone();

    loop {
        let marker = dir.join("encore.app");
        if marker.is_file() {
            let rel_path = original.strip_prefix(&dir).unwrap_or(&PathBuf::from(".")).to_path_buf();
            return Ok((dir, rel_path));
        }
        if !dir.pop() {
            break;
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no encore.app found in directory (or any of the parent directories)",
    ))
}