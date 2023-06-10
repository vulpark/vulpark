// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(super) macro basic_create($col: expr, $fun: expr, $value: expr) {
    $col.insert_one($fun(&$value), None).await
}

pub(super) macro basic_fetch($col: expr, $search: expr) {
    if let Some(val) = $col.find_one($search, None).await? {
        Some(val.into())
    } else {
        None
    }
}

pub(super) macro basic_update($col: expr, $search: expr, $replace: expr) {
    if let Some(val) = $col.find_one_and_update($search, $replace, None).await? {
        Some(val.into())
    } else {
        None
    }
}

pub(super) macro eq($($val: expr),*) {
    keyed!($(stringify!($val), $val),*)
}

pub(super) macro keyed($($key: expr, $value: expr),*) {
    {
        let mut doc = mongodb::bson::Document::new();
        $(
            doc.insert($key, $value);
        )*
        doc
    }
}

pub(super) macro eq_keyed($key: expr, $value: expr $(, $($val: expr),*)?) {
    keyed!($key, $value $(, $(stringify!($val), $val),*)?)
}

pub(super) macro id($id: expr $(, $($val: expr),*)?) {
    eq_keyed!("_id", $id $(, $($val,)*)?)
}

pub(super) macro before($time: expr $(, $($val: expr),*)?) {
    eq_keyed!("created", keyed!("lt", $time) $(, $($val),*)?)
}

pub(super) macro after($time: expr $(, $($val: expr),*)?) {
    eq_keyed!("created", keyed!("gt", $time) $(, $($val),*)?)
}
