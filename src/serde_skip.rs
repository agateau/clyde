// SPDX-FileCopyrightText: 2026 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

pub fn is_none<T>(x: &Option<T>) -> bool {
    x.is_none()
}

pub fn is_zero(x: &u32) -> bool {
    *x == 0
}

pub fn is_empty<T: CanBeEmpty>(x: &T) -> bool {
    x.is_empty()
}

/// Helper trait to test emptiness on various structs
pub trait CanBeEmpty {
    fn is_empty(&self) -> bool;
}

impl CanBeEmpty for String {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V> CanBeEmpty for BTreeMap<K, V> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> CanBeEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
