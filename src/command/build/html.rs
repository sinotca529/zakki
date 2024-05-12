use crate::{
    command::build::pass::{
        adjust_link_to_md, convert_math, get_h1, highlight_code, read_yaml_header,
    },
    path::{DstPath, SrcPath},
    util::{encode_with_password, write_file},
};
use anyhow::Result;
use base64::prelude::*;
use derive_builder::Builder;
use indoc::formatdoc;
use pulldown_cmark::Options;
use serde::Serialize;
use std::{path::PathBuf, sync::OnceLock};
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Builder, Serialize)]
pub struct PageMetadata {
    date: String,
    #[serde(rename = "path")]
    dst_rel_path: PathBuf,
    title: String,
    tags: Vec<String>,
    crypto: bool,
}

impl PageMetadataBuilder {
    pub fn read_yaml(&mut self, yaml: &str) -> &mut Self {
        let yaml = YamlLoader::load_from_str(yaml)
            .ok()
            .and_then(|mut y| y.pop())
            .and_then(|y| y.into_hash());

        let Some(yaml) = yaml else {
            return self;
        };

        let date = yaml
            .get(&Yaml::String("date".to_owned()))
            .and_then(|date| date.as_str().map(str::to_string));

        if let Some(date) = date {
            self.date(date);
        }

        let tags: Vec<String> = yaml
            .get(&Yaml::String("tag".to_owned()))
            .and_then(|tags| tags.as_vec())
            .iter()
            .flat_map(|tags| tags.iter())
            .filter_map(|t| t.as_str().map(str::to_string))
            .collect();

        self.tags(tags);

        let crypto = yaml
            .get(&Yaml::String("crypto".to_owned()))
            .and_then(|date| date.as_bool())
            .unwrap_or(false);

        self.crypto(crypto);

        self
    }
}

pub struct Page {
    body: String,
    dst_path: DstPath,
    metadata: PageMetadata,
}

impl Page {
    pub fn from_md_file(src_path: &SrcPath) -> Result<Self> {
        assert!(src_path.is_md());
        let md_content = std::fs::read(src_path.get_ref())?;
        let md_content = std::str::from_utf8(&md_content)?;

        let dst_path = src_path.to_dst_path();

        let mut body = String::new();
        let mut title = String::new();

        let mut metadata_builder = PageMetadataBuilder::default();

        let parser = pulldown_cmark::Parser::new_ext(md_content, Options::all())
            .map(adjust_link_to_md)
            .map(convert_math)
            .map(highlight_code)
            .map(|e| get_h1(e, &mut title))
            .map(|e| read_yaml_header(e, &mut metadata_builder));

        pulldown_cmark::html::push_html(&mut body, parser);

        if title.is_empty() {
            title.push_str("(NoTitle)");
        }

        metadata_builder
            .title(title)
            .dst_rel_path(dst_path.rel_path().to_owned());

        let metadata = metadata_builder.build()?;

        Ok(Self {
            body,
            metadata,
            dst_path,
        })
    }

    fn tag_elem(&self, tag_name: &str) -> String {
        let path_to_tag = self.dst_path.path_to_dst().join("tag.html");
        let path_to_tag = path_to_tag.to_str().unwrap();
        format!(r#"<a class="tag" href="{path_to_tag}?tag={tag_name}">{tag_name}</a>"#)
    }

    fn tag_elems(&self, tag_elems: &[String]) -> String {
        let nsbp = "\u{00a0}";
        tag_elems
            .iter()
            .map(|n| self.tag_elem(n))
            .fold(String::new(), |acc, e| format!("{acc}{nsbp}{e}"))
    }

    fn crypto_html(html: &str) -> String {
        let html = html.as_bytes();
        let cypher = encode_with_password(get_password(), html);
        let encoded = BASE64_STANDARD.encode(cypher);

        formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
            <meta charset="UTF-8">
            </head>
            <script>
                const cypherBase64 = "{encoded}";

                async function decryptAes256Cbc(data, iv, key) {{
                    const aesKey = await crypto.subtle.importKey('raw', key, {{name: 'AES-CBC'}}, false, ['decrypt']);
                    return crypto.subtle.decrypt({{name: 'AES-CBC', iv: iv}}, aesKey, data);
                }}

                async function getAesKey() {{
                    const key = document.getElementById('keyInput').value;
                    const keyData = new TextEncoder().encode(key);
                    return await crypto.subtle.digest('SHA-256', keyData);
                }}

                function base64ToUint8Array(base64Str) {{
                    const raw = atob(base64Str);
                    return Uint8Array.from(Array.prototype.map.call(raw, (x) => {{
                        return x.charCodeAt(0);
                    }}));
                }}

                async function decodeCipher() {{
                    const cipherData = base64ToUint8Array(cypherBase64);
                    const iv = cipherData.slice(0, 16);
                    const encryptedData = cipherData.slice(16);
                    const keyObj = await getAesKey();

                    try {{
                        const decryptedData = await decryptAes256Cbc(encryptedData, iv, keyObj);
                        const decryptedText = new TextDecoder().decode(decryptedData);
                        document.documentElement.innerHTML = decryptedText;
                    }} catch (error) {{
                        console.error('Decryption failed:', error);
                    }}
                }}
            </script>
            <body>
                <input type="text" id="keyInput" placeholder="Enter your secret key">
                <button onclick="decodeCipher()">Decode</button>
            </body>
            </html>
        "#}
    }

    fn gen_html(&self) -> String {
        let html = formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <meta name="date" content="{data}">
            <head>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="{path_to_css}">
            </head>
            <body>
            <span>{data}</span><br>
            {tag_elems}<br>
            <span></span>
            {body}
            </body>
            </html>
        "#,
            tag_elems = self.tag_elems(&self.metadata.tags),
            data = self.metadata.date,
            path_to_css = self.dst_path.path_to_dst().join("style.css").to_str().unwrap(),
            body = self.body,
        };

        if self.metadata.crypto {
            Self::crypto_html(&html)
        } else {
            html
        }
    }

    pub fn metadata(self) -> PageMetadata {
        self.metadata
    }

    pub fn save(&self) -> Result<()> {
        let html = self.gen_html();
        write_file(self.dst_path.get_ref(), html).map_err(Into::into)
    }
}

fn get_password() -> &'static String {
    static PASSWORD: OnceLock<String> = OnceLock::new();
    PASSWORD.get_or_init(|| {
        print!("Input password for hidden pages:\n> ");
        let mut password = String::new();
        while std::io::stdin().read_line(&mut password).is_err() {
            print!("Input password for hidden pages:\n> ");
        }
        password.trim_end().to_owned()
    })
}
