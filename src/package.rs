use anyhow::Result;
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs::File;
use std::vec::Vec;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub sha256: String
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub releases: Vec<Release>,
}

impl Package {
    pub fn from_file(path: &OsStr) -> Result<Package> {
        let file = File::open(path)?;
        let package = serde_yaml::from_reader(file)?;
        Ok(package)
    }
}
