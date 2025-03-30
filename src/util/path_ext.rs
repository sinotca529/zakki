use std::path::{Path, PathBuf};

pub trait PathExt {
    /// 拡張子が ext かどうかを確認します。
    fn extension_is(&self, ext: &str) -> bool;

    /// パスから、パスの起点への相対パスを返します。
    /// self はディレクトリへのパスであることを仮定していますが、そのことを検証はしません。
    fn dir_path_to_origin_unchecked(&self) -> PathBuf;

    /// 子孫ファイルのパスの一覧を返します。
    fn descendants_file_paths(&self) -> std::io::Result<Vec<PathBuf>>;

    /// ディレクトリ直下に file_name のファイルを持つか確かめます
    fn has_file(&self, file_name: &str) -> std::io::Result<bool>;
}

impl PathExt for Path {
    fn extension_is(&self, ext: &str) -> bool {
        self.extension().is_some_and(|e| e == ext)
    }

    fn dir_path_to_origin_unchecked(&self) -> PathBuf {
        let mut base = PathBuf::new();
        self.iter().for_each(|_| base.push(".."));
        base
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

    fn has_file(&self, file_name: &str) -> std::io::Result<bool> {
        let has_file = std::fs::read_dir(self)?
            .filter_map(|f| f.ok())
            .map(|f| f.file_name())
            .any(|f| &f == file_name);

        Ok(has_file)
    }
}
