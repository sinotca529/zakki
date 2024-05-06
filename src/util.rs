use chrono::{DateTime, Utc};
use std::{io, path::Path, time::SystemTime};

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, contents)
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    let to = to.as_ref();
    std::fs::create_dir_all(to.parent().unwrap())?;
    std::fs::copy(from, to)
}

pub trait DateFormat {
    fn yyyy_mm_dd_utc(self) -> String;
}

impl DateFormat for SystemTime {
    fn yyyy_mm_dd_utc(self) -> String {
        let date: DateTime<Utc> = self.into();
        date.format("%Y-%m-%d").to_string()
    }
}

pub trait ToJs {
    fn to_js(&self) -> String;
}
