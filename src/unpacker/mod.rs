// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::boxed::Box;
use std::path::Path;

use anyhow::{anyhow, Result};

mod exe_unpacker;
mod single_file_unpacker;
mod tar_unpacker;
mod zip_unpacker;

use exe_unpacker::ExeUnpacker;
use single_file_unpacker::SingleFileUnpacker;
use tar_unpacker::TarUnpacker;
use zip_unpacker::ZipUnpacker;

pub trait Unpacker {
    /// Unpacks the archive in `dst_dir`
    ///
    /// Returns the name of the unpacked asset if the archive was a single-file one
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<Option<String>>;
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
    if SingleFileUnpacker::supports(name) {
        return Ok(Box::new(SingleFileUnpacker::new(archive)));
    }
    Err(anyhow!("Unsupported format {}", archive.display()))
}
