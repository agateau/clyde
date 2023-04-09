// SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
use std::env;
use std::io::Error;
use std::path::PathBuf;

include!("./src/cli.rs");

/// Generates the shell auto-completion files in the `completions` dir
fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=src/cli.rs");

    let directory = match env::var("CARGO_MANIFEST_DIR").as_ref() {
        Ok(x) => PathBuf::from(x).join("completions"),
        Err(_) => {
            return Ok(());
        }
    };
    let command = &mut Cli::command();
    let name = &command.get_name().to_string();

    println!(
        "cargo:info=Generated {:?}",
        generate_to(Bash, command, name, &directory)?
    );

    println!(
        "cargo:info=Generated {:?}",
        generate_to(Elvish, command, name, &directory)?
    );

    println!(
        "cargo:info=Generated {:?}",
        generate_to(Fish, command, name, &directory)?
    );

    println!(
        "cargo:info=Generated {:?}",
        generate_to(PowerShell, command, name, &directory)?
    );

    println!(
        "cargo:info=Generated {:?}",
        generate_to(Zsh, command, name, &directory)?
    );
    Ok(())
}
