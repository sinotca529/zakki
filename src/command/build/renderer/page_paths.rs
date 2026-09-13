use crate::{path::ProjectPaths, util::PathExt as _};
use std::path::{Path, PathBuf};

pub struct PagePaths<'a> {
    /// 変換元の md ファイル
    pub src_path: &'a Path,
    /// 書き出す先の html ファイル
    pub build_path: PathBuf,
    /// サイトのルートから見たこのページの位置。URL として使う
    pub url_path: PathBuf,
    /// このページからサイトのルートへ戻る相対パス。URL の前置きに使う
    pub url_to_root: PathBuf,
}

impl<'a> PagePaths<'a> {
    pub fn new(src_path: &'a Path, pj_paths: &ProjectPaths) -> Self {
        let build_path = pj_paths.build_path_of(src_path);
        let url_path = build_path
            .strip_prefix(pj_paths.build_dir())
            .unwrap()
            .to_path_buf();

        let url_to_root = url_path.parent().unwrap().dir_path_to_origin_unchecked();

        Self {
            src_path,
            build_path,
            url_path,
            url_to_root,
        }
    }
}
