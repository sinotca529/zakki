use std::path::Path;

pub trait PathExt {
    fn extension_is(&self, ext: &str) -> bool;
}

impl PathExt for Path {
    fn extension_is(&self, ext: &str) -> bool {
        self.extension().is_some_and(|e| e == ext)
    }
}
