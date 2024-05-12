// SPDX-FileCopyrightText: 2024 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

/// This module provides a hack to work-around a bug: after running `clyde doc <some_package>`, if
/// the user presses Ctrl+C, the terminal cursor is not restored.
///
/// For more info, see https://github.com/console-rs/dialoguer/issues/294
use std::io;

use console::Term;

pub struct CursorRestorer;

impl CursorRestorer {
    // Allow new_without_default because if we don't and implement Default then clippy suggests not
    // using it for unit structs, which does not work because this class implements Drop.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        CursorRestorer {}
    }
}

impl Drop for CursorRestorer {
    fn drop(&mut self) {
        let _ = Term::stdout().show_cursor();
    }
}

pub fn disable_ctrlc_handler() {
    ctrlc::set_handler(move || {}).expect("Error setting Ctrl-C handler");
}

pub fn is_ctrlc(err: &anyhow::Error) -> bool {
    match err.root_cause().downcast_ref::<io::Error>() {
        Some(io_err) => io_err.kind() == io::ErrorKind::Interrupted,
        None => false,
    }
}
