use std::path::Path;

use crate::file_cache::FileCache;

pub struct App {
    pub download_cache: FileCache
}

impl App {
    pub fn new() -> App {
        App {
            download_cache: FileCache::new(Path::new("/tmp"))
        }
    }
}
