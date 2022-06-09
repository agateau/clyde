use std::boxed::Box;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use tempfile::TempDir;

pub trait Unpacker {
    fn unpack(&self, binaries: &HashMap<String, String>, bin_dir: &Path) -> Result<()>;
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
    fn unpack(&self, binaries: &HashMap<String, String>, bin_dir: &Path) -> Result<()> {
        fs::create_dir_all(&bin_dir)?;

        let temp_dir = TempDir::new_in(bin_dir)?;

        let mut cmd = Command::new("tar");
        cmd.arg("-C");
        cmd.arg(temp_dir.path());
        cmd.arg("-xvf");
        cmd.arg(self.archive.canonicalize()?);
        for src in binaries.keys() {
            cmd.arg(src);
        }
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Error unpacking {}", self.archive.display()));
        }

        for (src, dst) in binaries.iter() {
            let src_path = temp_dir.path().join(src);
            let dst_path = bin_dir.join(dst);
            fs::rename(src_path, dst_path)?;
        }

        Ok(())
    }
}

pub fn get_unpacker(archive: &Path) -> Result<Box<dyn Unpacker>> {
    let name = archive
        .file_name()
        .ok_or(anyhow!("Can't find file name in {}", archive.display()))?
        .to_str()
        .ok_or(anyhow!("Invalid file name in {}", archive.display()))?;
    if name.ends_with(".tar.gz") || name.ends_with(".tar.bz2") || name.ends_with(".tar.xz") {
        return Ok(Box::new(TarUnpacker::new(&archive)));
    }
    Err(anyhow!("Unsupported format {}", archive.display()))
}
