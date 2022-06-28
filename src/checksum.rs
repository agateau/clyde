use std::fs::File;
use std::io;
use std::path::Path;

use anyhow::{anyhow, Result};
use hex;
use sha2::{digest::DynDigest, Sha256};

pub fn compute_checksum(path: &Path) -> Result<Box<[u8]>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::default();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize_reset())
}

pub fn verify_checksum(path: &Path, expected: &str) -> Result<()> {
    let result = compute_checksum(path)?;

    let actual = hex::encode(result);

    if actual != expected {
        return Err(anyhow!(
            "Checksums do not match.\nExpected: {}\nReceived: {}",
            expected,
            actual
        ));
    }
    Ok(())
}
