// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(super) macro basic_fetch($col: expr, $id: expr) {
    if let Some(val) = $col.find_one($id, None).await? {
        Some(val.into())
    } else {
        None
    }
}

pub(super) macro eq($a: expr, $b: expr) {
    mongodb::bson::doc! {$a: $b}
}

pub(super) macro id($id: expr) {
    eq!("_id", $id)
}

pub(super) macro before($time: expr) {
    mongodb::bson::doc! {"created": {"$lt": $time}}
}

pub(super) macro after($time: expr) {
    mongodb::bson::doc! {"created": {"$gt": $time}}
}
