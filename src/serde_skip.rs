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

pub fn is_vec_empty<T>(vec: &[T]) -> bool {
    vec.is_empty()
}

pub fn is_map_empty<K, V>(map: &BTreeMap<K, V>) -> bool {
    map.is_empty()
}
