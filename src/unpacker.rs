use std::boxed::Box;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

pub trait Unpacker {
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<()>;
}

struct TarUnpacker {
    pub archive: PathBuf,
}

impl TarUnpacker {
    fn new(archive: &Path) -> TarUnpacker {
        TarUnpacker {
            archive: archive.to_path_buf(),
        }
    }
}

impl Unpacker for TarUnpacker {
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<()> {
        fs::create_dir_all(&dst_dir)?;

        let mut cmd = Command::new("tar");
        cmd.arg("-C");
        cmd.arg(dst_dir);
        if strip > 0 {
            cmd.arg(format!("--strip-components={}", strip));
        }
        cmd.arg("-xf");
        cmd.arg(self.archive.canonicalize()?);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Error unpacking {}", self.archive.display()));
        }

        Ok(())
    }
}

pub fn get_unpacker(archive: &Path) -> Result<Box<dyn Unpacker>> {
    let name = archive
        .file_name()
        .ok_or_else(|| anyhow!("Can't find file name in {}", archive.display()))?
        .to_str()
        .ok_or_else(|| anyhow!("Invalid file name in {}", archive.display()))?;
    if name.ends_with(".tar.gz") || name.ends_with(".tar.bz2") || name.ends_with(".tar.xz") {
        return Ok(Box::new(TarUnpacker::new(archive)));
    }
    Err(anyhow!("Unsupported format {}", archive.display()))
}
