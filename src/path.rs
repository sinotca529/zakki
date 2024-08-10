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
    ($fname:literal, $to:literal) => {{
        let path = std::env::current_dir()?.join($to).join($fname);
        let result: Result<()> = if path.exists() {
            Ok(())
        } else {
            write_file(
                path,
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname)),
            )
        }
        .map_err(Into::into);
        result
    }};
}
