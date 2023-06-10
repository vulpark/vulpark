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

pub(super) macro eq($val: expr) {
    mongodb::bson::doc! {stringify!($val): $val}
}

pub(super) macro id($id: expr) {
    mongodb::bson::doc! { "_id": $id }
}

pub(super) macro before($time: expr, $id: expr) {
    mongodb::bson::doc! {"created": {"$lt": $time}, stringify!($id): $id}
}

pub(super) macro after($time: expr, $id: expr) {
    mongodb::bson::doc! {"created": {"$gt": $time}, stringify!($id): $id}
}
