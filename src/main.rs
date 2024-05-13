// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::Parser;

use clyde::app::App;
use clyde::cli::{Cli, Command};
use clyde::cmd::{
    doc_cmd, install_cmd, list_cmd, search_cmd, setup_cmd, show_cmd, uninstall_cmd, update_cmd,
    upgrade_cmd,
};
use clyde::ctrlcutils;
use clyde::ui::Ui;

pub fn exec(cli: Cli) -> Result<()> {
    let ui = Ui::default();
    let home = App::find_home()?;

    let _instance = App::create_single_instance(&home)?;

    ctrlcutils::disable_ctrlc_handler();

    let result = match cli.command {
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
        Command::Show {
            package_name,
            json,
            list,
        } => {
            let app = App::new(&home)?;
            show_cmd(&app, &package_name, json, list)
        }
        Command::Search { query } => {
            let app = App::new(&home)?;
            search_cmd(&app, &ui, &query)
        }
        Command::Doc { package_name } => {
            let app = App::new(&home)?;
            doc_cmd(&app, &package_name)
        }
        Command::List { json } => {
            let app = App::new(&home)?;
            list_cmd(&app, json)
        }
        Command::Upgrade {} => {
            let app = App::new(&home)?;
            upgrade_cmd(&app, &ui)
        }
    };
    if let Err(ref err) = result {
        if ctrlcutils::is_ctrlc(err) {
            println!("Interrupted");
            return Ok(());
        }
    }
    result
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    exec(cli)
}
