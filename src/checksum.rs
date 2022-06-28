use std::fs::File;
use std::io;
use std::path::Path;

use anyhow::{anyhow, Result};
use hex;
use sha2::{digest::DynDigest, Sha256};

pub fn compute_checksum(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::default();
    io::copy(&mut file, &mut hasher)?;
    let checksum = hex::encode(hasher.finalize_reset());
    Ok(checksum)
}

pub fn verify_checksum(path: &Path, expected: &str) -> Result<()> {
    let actual = compute_checksum(path)?;

    if actual != expected {
        return Err(anyhow!(
            "Checksums do not match.\nExpected: {}\nReceived: {}",
            expected,
            actual
        ));
    }
    Ok(())
}
