use anyhow::{Context, bail};

use crate::{command::build::renderer::url::Url, path::ProjectPaths, util::PathExt as _};
use std::path::{Component::*, Path, PathBuf};

pub struct PageLocs<'a> {
    /// 変換元の md ファイル
    pub src_path: &'a Path,
    /// 書き出す先の html ファイル
    pub build_path: PathBuf,
    /// サイトのルートから見たこのページの位置。URL として使う
    pub url_path: Url,
    /// このページからサイトのルートへ戻る相対パス。URL の前置きに使う
    pub url_to_root: Url,
}

impl<'a> PageLocs<'a> {
    pub fn new(src_path: &'a Path, pj_paths: &ProjectPaths) -> anyhow::Result<Self> {
        let build_path = pj_paths.build_path_of(src_path);

        let out_path = build_path
            .strip_prefix(pj_paths.build_dir())
            .unwrap()
            .to_path_buf();
        let url_path = Self::to_url(&out_path)?;

        let url_to_root = Self::to_url(&out_path.parent().unwrap().dir_path_to_origin_unchecked())?;

        Ok(Self {
            src_path,
            build_path,
            url_path,
            url_to_root,
        })
    }

    /// build_dir からの相対パスを、サイトのルートから見た URL にします。
    fn to_url(rel: &Path) -> anyhow::Result<Url> {
        let mut url = Url::default();
        for c in rel.components() {
            match c {
                Normal(s) => {
                    let s = s.to_str().context("ファイル名が UTF-8 ではありません")?;
                    url.push(s);
                }
                CurDir => {}
                Prefix(_) | RootDir => bail!("絶対パスは URL に変換できません"),
                ParentDir => {
                    url.push("..");
                }
            }
        }
        Ok(url)
    }
}
