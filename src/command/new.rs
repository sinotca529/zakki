use crate::path::zakki_src_dir;
use anyhow::{Result, bail};
use std::path::PathBuf;

pub fn new(path: &str) -> Result<()> {
    let mut rel: PathBuf = path.into();
    if rel.extension().is_none() {
        rel.set_extension("md");
    }

    let dest = zakki_src_dir()?.join(&rel);

    if dest.exists() {
        bail!("{} already exists", dest.display());
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let title = dest
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let today = today();

    let content = format!(
        "---\ntitle: {title}\ncreate: {today}\nupdate: {today}\ntag: []\n---\n"
    );

    std::fs::write(&dest, content)?;
    println!("Created: {}", dest.display());
    Ok(())
}

fn today() -> String {
    // std のみで YYYY-MM-DD を得る
    // UNIX_EPOCH からの日数を計算する
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    // グレゴリオ暦変換
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}")
}

// UNIX_EPOCH (1970-01-01) からの日数 → (year, month, day)
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // アルゴリズム: http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
