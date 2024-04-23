mod convert;

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use convert::md_to_html;

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    Build,
    Clean,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    subcommand: Command,
}

fn src_dir() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("src"))
}

fn build_dir() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("build"))
}

fn css_path() -> Result<PathBuf> {
    Ok(build_dir()?.join("style.css"))
}

fn init_zakki_dir() -> Result<()> {
    let src_dir: PathBuf = src_dir()?;
    let demo_md_path = src_dir.join("index.md");
    std::fs::create_dir(src_dir)?;

    let build_dir = build_dir()?;
    let demo_css_path = build_dir.join("style.css");
    std::fs::create_dir(build_dir)?;

    std::fs::File::create(demo_md_path)?;
    std::fs::File::create(demo_css_path)?;
    Ok(())
}

fn visit_files_recursively(
    dir: impl AsRef<Path>,
    operator: impl Fn(PathBuf) -> Result<()>,
) -> Result<()> {
    let dir = dir.as_ref();
    let mut work_list: Vec<PathBuf> = vec![dir.into()];
    while let Some(dir) = work_list.pop() {
        for e in std::fs::read_dir(&dir)? {
            let path = e?.path();
            if path.is_dir() {
                work_list.push(path);
            } else {
                operator(path)?;
            }
        }
    }
    Ok(())
}

fn html_path(md_path: impl AsRef<Path>) -> Result<PathBuf> {
    fn inner(md_path: &Path) -> Result<PathBuf> {
        let rel_path = md_path.strip_prefix(src_dir()?)?;
        let html_path = build_dir()?.join(rel_path).with_extension("html");
        Ok(html_path)
    }
    inner(md_path.as_ref())
}

fn read_file(path: impl AsRef<Path>) -> Result<String> {
    let mut buf = String::new();
    std::fs::File::open(path)?.read_to_string(&mut buf)?;
    Ok(buf)
}

fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = std::fs::File::create(path)?;
    file.write_fmt(format_args!("{content}"))?;
    Ok(())
}

fn relative_path_to_css(html_path: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(pathdiff::diff_paths(css_path()?, html_path.as_ref().parent().unwrap()).unwrap())
}

fn make_html_file(md_path: PathBuf) -> Result<()> {
    let md_content = read_file(&md_path)?;
    let html_path = html_path(&md_path)?;

    let path_to_css = relative_path_to_css(&html_path)?;
    let html_content = md_to_html(&md_content, path_to_css.to_str().unwrap());

    write_file(html_path, &html_content)?;

    Ok(())
}

fn build() -> Result<()> {
    visit_files_recursively(src_dir()?, make_html_file)
}

fn clean_zakki_dir() -> Result<()> {
    std::fs::remove_dir_all(build_dir()?)?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.subcommand {
        Command::Init => init_zakki_dir(),
        Command::Build => build(),
        Command::Clean => clean_zakki_dir(),
    }
}
