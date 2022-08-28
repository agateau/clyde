// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::iter::zip;

const COLUMN_CHAR: char = '│';
const SEPARATOR_COLUMN_CHAR: char = '┼';
const SEPARATOR_CHAR: char = '─';

fn putc(ch: char) {
    print!("{ch}");
}

/// Simple class to print tabular data
pub struct Table {
    columns: Vec<usize>,
}

impl Table {
    pub fn new(columns: &[usize]) -> Self {
        Table {
            columns: columns.into(),
        }
    }

    pub fn add_row(&self, cells: &[&str]) {
        let mut first = true;
        for (width, cell) in zip(&self.columns, cells) {
            if first {
                first = false;
            } else {
                putc(COLUMN_CHAR);
            }
            print!("{cell:width$}");
        }
        println!();
    }

    pub fn add_separator(&self) {
        let mut first = true;
        for width in &self.columns {
            if first {
                first = false;
            } else {
                putc(SEPARATOR_COLUMN_CHAR);
            }
            for _ in 0..*width {
                putc(SEPARATOR_CHAR);
            }
        }
        println!();
    }
}
