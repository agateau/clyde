// SPDX-FileCopyrightText: 2024 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod doc;
mod install;
mod list;
mod search;
mod setup;
mod show;
mod uninstall;
mod update;
mod upgrade;

pub use doc::doc_cmd;

pub use install::{install_cmd, install_package, install_packages, InstallRequest};

pub use list::list_cmd;

pub use search::search_cmd;

pub use setup::setup_cmd;

pub use show::show_cmd;

pub use uninstall::{uninstall_cmd, uninstall_package};

pub use update::update_cmd;

pub use upgrade::upgrade_cmd;
