use std::boxed::Box;
use std::path::Path;

use anyhow::{anyhow, Result};

pub mod exe_unpacker;
pub mod tar_unpacker;
pub mod zip_unpacker;

use exe_unpacker::ExeUnpacker;
use tar_unpacker::TarUnpacker;
use zip_unpacker::ZipUnpacker;

pub trait Unpacker {
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<()>;
}

pub fn get_unpacker(archive: &Path) -> Result<Box<dyn Unpacker>> {
    let name = archive
        .file_name()
        .ok_or_else(|| anyhow!("Can't find file name in {}", archive.display()))?
        .to_str()
        .ok_or_else(|| anyhow!("Invalid file name in {}", archive.display()))?;
    if TarUnpacker::supports(name) {
        return Ok(Box::new(TarUnpacker::new(archive)));
    }
    if ZipUnpacker::supports(name) {
        return Ok(Box::new(ZipUnpacker::new(archive)));
    }
    if ExeUnpacker::supports(archive) {
        return Ok(Box::new(ExeUnpacker::new(archive)));
    }
    Err(anyhow!("Unsupported format {}", archive.display()))
}
