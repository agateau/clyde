// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::Parser;

use clyde::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.exec()
}
