use anyhow::{Context, bail};
use serde::Serialize;
use std::fmt::Display;
use std::path::{Component::*, Path};

#[derive(Clone)]
pub struct Url(String);

impl Url {
    /// '.' に対応する URL を返します。
    pub fn single_dot() -> Self {
        Self(".".to_string())
    }

    /// 相対パスから URL を作成します。
    pub fn from_relative_path(rel: &Path) -> anyhow::Result<Self> {
        let mut segs = vec![];

        for c in rel.components() {
            match c {
                Normal(s) => {
                    let s = s.to_str().context("ファイル名が UTF-8 ではありません")?;
                    segs.push(Self::encode_path_segment(s));
                }
                CurDir => {}
                Prefix(_) | RootDir => bail!("絶対パスは URL に変換できません"),
                ParentDir => {
                    segs.push("..".to_string());
                }
            }
        }
        Ok(Self(segs.join("/")))
    }

    /// URL の 1 区画として安全な形にします。
    /// 非 ASCII はそのまま残します。理由は encode_query_value と同じです。
    fn encode_path_segment(s: &str) -> String {
        // WHATWG の path percent-encode set に %、\、/ を足したもの。
        // % はエスケープの起点、\ はブラウザが / と同じに扱う、
        // / は区切りなので、いずれも素通しにできません。
        const META_CHARS: &str = r#" "<>`#?{}%\/"#;
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            if META_CHARS.contains(c) || c.is_control() {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).as_bytes() {
                    out.push_str(&format!("%{b:02X}"));
                }
            } else {
                out.push(c);
            }
        }
        out
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Url {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
