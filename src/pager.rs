// SPDX-FileCopyrightText: 2024 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use which::which;

const DEFAULT_PAGERS: [&str; 3] = ["bat", "less", "more"];

pub fn find_pager() -> Option<String> {
    if let Ok(pager) = env::var("CLYDE_PAGER") {
        return Some(pager);
    }
    if let Ok(pager) = env::var("PAGER") {
        return Some(pager);
    }
    for pager in &DEFAULT_PAGERS {
        if which(pager).is_ok() {
            return Some(pager.to_string());
        }
    }
    None
}
