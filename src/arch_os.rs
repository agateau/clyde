// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env::consts;
use std::fmt;

use anyhow::{anyhow, Result};

pub const ANY: &str = "any";

pub const ARCH_X86_64: &str = "x86_64";
pub const ARCH_X86: &str = "x86";
pub const ARCH_AARCH64: &str = "aarch64";

pub const OS_LINUX: &str = "linux";
pub const OS_MACOS: &str = "macos";
pub const OS_WINDOWS: &str = "windows";

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArchOs {
    pub arch: String,
    pub os: String,
}

impl fmt::Display for ArchOs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.arch, self.os)
    }
}

impl ArchOs {
    pub fn new(arch: &str, os: &str) -> ArchOs {
        ArchOs {
            arch: arch.into(),
            os: os.into(),
        }
    }

    pub fn with_any_arch(&self) -> ArchOs {
        ArchOs {
            arch: ANY.into(),
            os: self.os.clone(),
        }
    }

    pub fn with_any_os(&self) -> ArchOs {
        ArchOs {
            arch: self.arch.clone(),
            os: ANY.into(),
        }
    }

    pub fn parse(text: &str) -> Result<ArchOs> {
        if text == ANY {
            return Ok(ArchOs::new(ANY, ANY));
        }

        let mut iter = text.split('-');
        let arch = iter
            .next()
            .ok_or_else(|| anyhow!("Could not find arch in {}", text))?;
        let token = iter
            .next()
            .ok_or_else(|| anyhow!("Could not find OS in {}", text))?;
        let os = match token {
            "unknown" => iter
                .next()
                .ok_or_else(|| anyhow!("Could not find OS in {}", text))?,
            x => x,
        };
        Ok(ArchOs::new(arch, os))
    }

    pub fn current() -> ArchOs {
        ArchOs::new(consts::ARCH, consts::OS)
    }

    pub fn to_str(&self) -> String {
        format!("{}-{}", self.arch, self.os)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            ArchOs::parse("x86_64-linux").unwrap(),
            ArchOs::new("x86_64", "linux")
        );
        assert_eq!(
            ArchOs::parse("x86_64-unknown-linux-gnu").unwrap(),
            ArchOs::new("x86_64", "linux")
        );
    }
}
