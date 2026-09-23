mod date;
mod file_io;
mod path_ext;

pub use date::Date;
pub use file_io::{copy_file, write_file};
pub use path_ext::PathExt;

/// 本プロジェクトの asset ディレクトリ下にあるファイルの内容を読み込みます
#[macro_export]
macro_rules! include_asset {
    ($fname:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname))
    };
}

/// 本プロジェクトの testdata ディレクトリ下にあるファイルの内容を読み込みます
/// Rust 側と JS 側が同じ表を読むため、パスは呼び出し元の位置に依存させません
#[cfg(test)]
#[macro_export]
macro_rules! include_testdata {
    ($fname:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/", $fname))
    };
}

/// 本プロジェクトの asset ディレクトリ下にあるファイルの内容をコピーします
/// ファイルの内容はコンパイル時にバイナリに埋め込まれます
#[macro_export]
macro_rules! copy_asset {
    ($fname:expr, $to:expr) => {{
        let path = $to.join($fname);
        if path.exists() {
            Ok(())
        } else {
            $crate::util::write_file(
                path,
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname)),
            )
            .with_context(|| anyhow!("Failed to copy {}", $fname))
        }
    }};
}
