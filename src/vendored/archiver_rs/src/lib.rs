use std::path::Path;

use thiserror::Error;

#[cfg(feature = "bzip")]
pub use crate::bzip2::Bzip2;

#[cfg(feature = "gzip")]
pub use crate::gzip::Gzip;

#[cfg(feature = "tar")]
pub use crate::tar::Tar;

#[cfg(feature = "xz")]
pub use crate::xz::Xz;

#[cfg(feature = "zip")]
pub use crate::zip::Zip;

#[cfg(feature = "bzip")]
mod bzip2;

#[cfg(feature = "gzip")]
mod gzip;

#[cfg(feature = "tar")]
mod tar;

#[cfg(feature = "xz")]
mod xz;

#[cfg(feature = "zip")]
mod zip;

#[derive(Error, Debug)]
pub enum ArchiverError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("File Not Found")]
    NotFound,
}

type Result<T> = std::result::Result<T, ArchiverError>;

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

pub fn open(path: &Path) -> std::io::Result<Box<dyn Archive>> {
    use std::fs::File;
    use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

    let mut file = File::open(&path)?;
    let mut buffer = [0u8; 2];
    file.read(&mut buffer)?;
    file.seek(SeekFrom::Start(0))?;

    match buffer {
        #[cfg(all(feature = "bzip", feature = "tar"))]
        [0x42, 0x5A] => Ok(Box::new(Tar::new(Bzip2::new(file)?)?)), // .tar.gz
        #[cfg(all(feature = "gzip", feature = "tar"))]
        [0x1F, 0x8B] => Ok(Box::new(Tar::new(Gzip::new(file)?)?)), // .tar.gz
        #[cfg(all(feature = "xz", feature = "tar"))]
        [0xFD, 0x37] => Ok(Box::new(Tar::new(Xz::new(file)?)?)), // .tar.xz
        #[cfg(feature = "zip")]
        [0x50, 0x4B] => Ok(Box::new(Zip::new(file)?)), // .zip
        _ => Err(Error::from(ErrorKind::InvalidData))?,
    }
}
