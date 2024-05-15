// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env::consts;
use std::fmt;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const ANY: &str = "any";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    Any,
    X86_64,
    X86,
    Aarch64,
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_yaml::to_string(self).unwrap().trim(),)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Any,
    Linux,
    MacOs,
    Windows,
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_yaml::to_string(self).unwrap().trim(),)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ArchOs {
    pub arch: Arch,
    pub os: Os,
}

impl fmt::Display for ArchOs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ArchOs {
    fn from_strings(arch: &str, os: &str) -> Result<ArchOs> {
        let arch = serde_yaml::from_str(arch)?;
        let os = serde_yaml::from_str(os)?;
        Ok(ArchOs { arch, os })
    }

    pub fn any() -> ArchOs {
        ArchOs::new(Arch::Any, Os::Any)
    }

    pub fn new(arch: Arch, os: Os) -> ArchOs {
        ArchOs { arch, os }
    }

    pub fn with_any_arch(&self) -> ArchOs {
        ArchOs {
            arch: Arch::Any,
            os: self.os,
        }
    }

    pub fn with_any_os(&self) -> ArchOs {
        ArchOs {
            arch: self.arch,
            os: Os::Any,
        }
    }

    pub fn parse(text: &str) -> Result<ArchOs> {
        if text == ANY {
            return Ok(ArchOs::new(Arch::Any, Os::Any));
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
        ArchOs::from_strings(arch, os)
    }

    pub fn current() -> ArchOs {
        ArchOs::from_strings(consts::ARCH, consts::OS).unwrap()
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
            ArchOs::new(Arch::X86_64, Os::Linux)
        );
        assert_eq!(
            ArchOs::parse("x86_64-unknown-linux-gnu").unwrap(),
            ArchOs::new(Arch::X86_64, Os::Linux)
        );
    }

    #[test]
    fn test_to_str() {
        assert_eq!(
            ArchOs::new(Arch::X86_64, Os::Linux).to_str(),
            "x86_64-linux"
        );
    }
}
