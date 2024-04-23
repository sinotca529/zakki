use anyhow::Result;
use std::{
    io::{Read, Write},
    path::Path,
};

pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = std::fs::File::create(path)?;
    file.write_fmt(format_args!("{content}"))?;
    Ok(())
}

pub fn read_file(path: impl AsRef<Path>) -> Result<String> {
    let mut buf = String::new();
    std::fs::File::open(path)?.read_to_string(&mut buf)?;
    Ok(buf)
}
