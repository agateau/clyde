// SPDX-FileCopyrightText: 2026 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

use crate::arch_os::{Arch, Os};
use crate::serde_skip::is_none;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Hash, Serialize)]
pub enum FetcherConfig {
    #[default]
    Auto,
    Forgejo {
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        arch: Option<Arch>,
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        os: Option<Os>,
        base_url: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        include: Option<String>,
    },
    GitHub {
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        arch: Option<Arch>,
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        os: Option<Os>,
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        include: Option<String>,
    },
    GitLab {
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        arch: Option<Arch>,
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        os: Option<Os>,
        #[serde(default)]
        #[serde(skip_serializing_if = "is_none")]
        include: Option<String>,
    },
    Script,
    Off,
}
