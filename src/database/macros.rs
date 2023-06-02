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
