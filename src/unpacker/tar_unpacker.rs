use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::unpacker::Unpacker;

pub struct TarUnpacker {
    pub archive: PathBuf,
}

impl TarUnpacker {
    pub fn new(archive: &Path) -> TarUnpacker {
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
