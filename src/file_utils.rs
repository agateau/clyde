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
