// This file is a copy of lib.rs, simplified to only include the features
// Clyde needs.
use std::path::Path;

use thiserror::Error;

pub use bzip2::Bzip2;

pub use gzip::Gzip;

pub use xz::Xz;

mod bzip2;

mod gzip;

mod xz;

#[derive(Error, Debug)]
pub enum ArchiverError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("File Not Found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, ArchiverError>;

pub trait Archive {
    fn contains(&mut self, file: String) -> Result<bool>;

    fn extract(&mut self, destination: &Path) -> Result<()>;

    fn extract_single(&mut self, target: &Path, file: String) -> Result<()>;

    fn files(&mut self) -> Result<Vec<String>>;

    fn walk(&mut self, f: Box<dyn Fn(String) -> Option<String>>) -> Result<()>;
}

pub trait Compressed: std::io::Read {
    fn decompress(&mut self, target: &Path) -> Result<()>;
}
