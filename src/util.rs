mod aes_256_cbc;
mod bloom_filter;
mod file_io;
mod fxhash;
mod path_ext;
mod segmenter;
mod vec_ext;

pub use aes_256_cbc::*;
pub use bloom_filter::*;
pub use file_io::*;
pub use path_ext::*;
pub use segmenter::*;
pub use vec_ext::*;

/// 本プロジェクトの asset ディレクトリ下にあるファイルの内容を読み込みます
#[macro_export]
macro_rules! include_asset {
    ($fname:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname))
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
            write_file(
                path,
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname)),
            )
            .with_context(|| anyhow!("Failed to copy {}", $fname))
        }
    }};
}
