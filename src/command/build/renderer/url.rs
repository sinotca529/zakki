use serde::Serialize;
use std::fmt::Display;

#[derive(Default, Clone)]
pub struct Url(Vec<String>);

impl Url {
    pub fn join(&self, seg: &str) -> Self {
        let mut s = self.clone();
        s.push(seg);
        s
    }

    pub fn push(&mut self, seg: &str) -> &mut Self {
        self.0.push(Self::encode_path_segment(seg));
        self
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
        if self.0.is_empty() {
            f.write_str(".")
        } else {
            f.write_str(&self.0.join("/"))
        }
    }
}

impl Serialize for Url {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
