use crate::util::PathExt;
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// 設定ファイルの名前。
/// `include_bytes!` の `concat!` がリテラルを要求するため、定数ではなくマクロで定義します。
#[macro_export]
macro_rules! config_file_name {
    () => {
        "zakki.toml"
    };
}

macro_rules! getter {
    ($field:ident, $type:ty) => {
        pub fn $field(&self) -> $type {
            &self.$field
        }
    };
}

pub struct ProjectPaths {
    root_dir: PathBuf,
    src_dir: PathBuf,
    src_public_dir: PathBuf,
    src_private_dir: PathBuf,
    src_draft_dir: PathBuf,
    build_dir: PathBuf,
    config_path: PathBuf,
}

impl ProjectPaths {
    getter!(root_dir, &Path);
    getter!(src_dir, &Path);
    getter!(src_public_dir, &Path);
    getter!(src_private_dir, &Path);
    getter!(src_draft_dir, &Path);
    getter!(build_dir, &Path);
    getter!(config_path, &Path);

    /// root_dir を起点としたパス情報を返します (設定ファイルの探索・作成は実施しません)
    pub fn new(root_dir: PathBuf) -> Self {
        let src_dir = root_dir.join("src");

        Self {
            src_public_dir: src_dir.join("public"),
            src_private_dir: src_dir.join("private"),
            src_draft_dir: src_dir.join("draft"),
            src_dir,
            build_dir: root_dir.join("build"),
            config_path: root_dir.join(config_file_name!()),
            root_dir,
        }
    }

    /// 設定ファイルのあるディレクトリをルートとしたパス情報を返します。
    /// 祖先方向に設定ファイルを探索します。
    pub fn find(from: &Path) -> Result<Self> {
        let mut dir: Option<&Path> = Some(from);

        while let Some(d) = dir {
            let is_zakki_root = d.has_file(config_file_name!())?;
            if is_zakki_root {
                return Ok(Self::new(d.to_owned()));
            }
            dir = d.parent();
        }

        bail!(
            "ディレクトリ {} は zakki 用のもの、またはその配下ではありません",
            from.display()
        );
    }

    pub fn build_path_of(&self, src_path: impl AsRef<Path>) -> PathBuf {
        let src_path = src_path.as_ref();
        let rel = src_path.strip_prefix(self.src_dir()).unwrap();

        if rel.extension_is("md") {
            self.build_dir.join(rel.with_extension("html"))
        } else {
            self.build_dir.join(rel)
        }
    }

    pub fn is_draft(&self, src_path: &Path) -> bool {
        src_path.starts_with(self.src_draft_dir())
    }

    pub fn is_private(&self, src_path: &Path) -> bool {
        src_path.starts_with(self.src_private_dir())
    }

    /// サブページか否かを返す。
    /// サブページとは、 public, private, draft 直下になく、かつ、名前が index ではないファイルである。
    /// サブページはトップページの記事一覧に表示されない。
    pub fn is_subpage(&self, src_path: &Path) -> bool {
        !Self::is_index(src_path) && self.is_in_sub_dir(src_path)
    }

    /// 記事をまとめるディレクトリの `index.md` か否かを返す。
    /// サブページと同じ位置にある index である。トップページの記事一覧には表示され、
    /// カードに group クラスが付く。
    pub fn is_group(&self, src_path: &Path) -> bool {
        Self::is_index(src_path) && self.is_in_sub_dir(src_path)
    }

    fn is_index(src_path: &Path) -> bool {
        src_path.file_stem().map(|s| s == "index").unwrap_or(false)
    }

    /// public, private, draft 直下より深い位置にあるか否かを返す。
    fn is_in_sub_dir(&self, src_path: &Path) -> bool {
        let src_rel_path = src_path.strip_prefix(self.src_dir()).unwrap();
        src_rel_path.components().count() >= 3
    }
}

#[cfg(test)]
mod test {
    use super::ProjectPaths;
    use std::path::PathBuf;

    /// サブページと、記事をまとめる index は同じ位置にあります。
    /// 分けるのは名前だけなので、両方を並べて確かめます。
    #[test]
    fn classifies_pages_by_place_and_name() {
        let paths = ProjectPaths::new(PathBuf::from("site"));
        let src = |rel: &str| PathBuf::from("site/src").join(rel);

        let cases = [
            // (パス, サブページか, まとめる index か)
            ("public/index.md", false, false),
            ("public/article.md", false, false),
            ("public/series/index.md", false, true),
            ("public/series/010-first.md", true, false),
            ("private/series/index.md", false, true),
        ];

        for (rel, is_subpage, is_group) in cases {
            let path = src(rel);
            assert_eq!(paths.is_subpage(&path), is_subpage, "サブページ: {rel}");
            assert_eq!(paths.is_group(&path), is_group, "まとめる index: {rel}");
        }
    }
}
