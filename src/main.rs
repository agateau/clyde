// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::Parser;

use clyde::app::App;
use clyde::cli::{Cli, Command};
use clyde::install::install_cmd;
use clyde::list::list_cmd;
use clyde::search::search_cmd;
use clyde::setup::setup_cmd;
use clyde::show::show_cmd;
use clyde::ui::Ui;
use clyde::uninstall::uninstall_cmd;
use clyde::update::update_cmd;
use clyde::upgrade::upgrade_cmd;

pub fn exec(cli: Cli) -> Result<()> {
    let ui = Ui::default();
    let home = App::find_home()?;

    let _instance = App::create_single_instance(&home)?;

    match cli.command {
        Command::Setup {
            update_scripts,
            store_url,
        } => setup_cmd(&ui, &home, update_scripts, store_url.as_deref()),
        Command::Update {} => {
            let app = App::new(&home)?;
            update_cmd(&app, &ui)
        }
        Command::Install {
            reinstall,
            package_names,
        } => {
            let app = App::new(&home)?;
            install_cmd(&app, &ui, reinstall, &package_names)
        }
        Command::Uninstall { package_names } => {
            let app = App::new(&home)?;
            uninstall_cmd(&app, &ui, &package_names)
        }
        Command::Show { package_name, list } => {
            let app = App::new(&home)?;
            show_cmd(&app, &package_name, list)
        }
        Command::Search { query } => {
            let app = App::new(&home)?;
            search_cmd(&app, &ui, &query)
        }
        Command::List {} => {
            let app = App::new(&home)?;
            list_cmd(&app)
        }
        Command::Upgrade {} => {
            let app = App::new(&home)?;
            upgrade_cmd(&app, &ui)
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    exec(cli)
}
