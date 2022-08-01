// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use console::{style, StyledObject};

#[derive(Default)]
pub struct Ui {
    level: u32,
}

impl Ui {
    pub fn nest(&self) -> Self {
        Ui {
            level: self.level + 1,
        }
    }

    pub fn print_indent(&self) {
        for _ in 0..self.level {
            eprint!("  ");
        }
    }

    pub fn get_indent(&self) -> String {
        "  ".repeat(self.level as usize)
    }

    pub fn println(&self, msg: &str) {
        self.print_indent();
        eprintln!("{}", msg);
    }

    pub fn info(&self, msg: &str) {
        self.print_with_indicator(style("[I]").blue(), msg);
    }

    pub fn warn(&self, msg: &str) {
        self.print_with_indicator(style("[!]").yellow(), msg);
    }

    pub fn error(&self, msg: &str) {
        self.print_with_indicator(style("[X]").red(), msg);
    }

    fn print_with_indicator(&self, indicator: StyledObject<&str>, msg: &str) {
        self.print_indent();
        eprintln!("{indicator} {}", msg);
    }
}
