use std::env::consts;

use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArchOs {
    pub arch: String,
    pub os: String,
}

impl ArchOs {
    pub fn new(arch: &str, os: &str) -> ArchOs {
        ArchOs {
            arch: arch.into(),
            os: os.into(),
        }
    }

    pub fn parse(text: &str) -> Result<ArchOs> {
        if text == "any" {
            return Ok(ArchOs::new("any", "any"));
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
        return format!("{}-{}", self.arch, self.os);
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
