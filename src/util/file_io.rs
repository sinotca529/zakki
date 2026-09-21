use std::{io, path::Path};

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();

    // 親ディレクトリは、書けなかったときだけ作る。
    // 記事ごとに作りに行くと、ほとんどの回が既にある確認だけで終わる。
    match std::fs::write(path, contents) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(path, contents)
        }
        result => result,
    }
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    match std::fs::copy(from, to) {
        Err(e) if e.kind() == io::ErrorKind::NotFound && from.exists() => {
            std::fs::create_dir_all(to.parent().unwrap())?;
            std::fs::copy(from, to)
        }
        result => result,
    }
}
