// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::{anyhow, Result};

pub fn get_file_name(path: &Path) -> Result<&str> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("{path:?} has no filename"))?
        .to_str()
        .ok_or_else(|| anyhow!("{path:?} has a weird filename"))?;
    Ok(name)
}

#[cfg(unix)]
pub fn set_file_executable(path: &Path) -> Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::metadata(path).unwrap().permissions();
    let mode = permissions.mode() | 0o111;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}
