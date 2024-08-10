use std::path::{Path, PathBuf};

pub trait PathExt {
    /// 拡張子が ext かどうかを確認します。
    fn extension_is(&self, ext: &str) -> bool;

    /// from からの相対パスを返却します。
    fn relative_path(&self, from: impl AsRef<Path>) -> Option<PathBuf>;
}

impl PathExt for Path {
    fn extension_is(&self, ext: &str) -> bool {
        self.extension().is_some_and(|e| e == ext)
    }

    fn relative_path(&self, from: impl AsRef<Path>) -> Option<PathBuf> {
        let from = from.as_ref();
        pathdiff::diff_paths(self, from)
    }
}
