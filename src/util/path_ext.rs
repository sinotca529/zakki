use std::path::{Path, PathBuf};

pub trait PathExt {
    /// 拡張子が ext かどうかを確認します。
    fn extension_is(&self, ext: &str) -> bool;

    /// from からの相対パスを返します。
    fn relative_path(&self, from: impl AsRef<Path>) -> Option<PathBuf>;

    /// 子孫ファイルのパスの一覧を返します。
    fn descendants_file_paths(&self) -> std::io::Result<Vec<PathBuf>>;
}

impl PathExt for Path {
    fn extension_is(&self, ext: &str) -> bool {
        self.extension().is_some_and(|e| e == ext)
    }

    fn relative_path(&self, from: impl AsRef<Path>) -> Option<PathBuf> {
        let from = from.as_ref();
        pathdiff::diff_paths(self, from)
    }

    fn descendants_file_paths(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let mut work_list: Vec<PathBuf> = vec![self.to_owned()];
        while let Some(dir) = work_list.pop() {
            for e in std::fs::read_dir(dir)? {
                let path = e?.path();
                if path.is_dir() {
                    work_list.push(path);
                } else {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }
}
