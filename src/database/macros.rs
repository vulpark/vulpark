pub(super) macro eq($a: expr, $b: expr) {
    mongodb::bson::doc!{$a: $b}
}

pub(super) macro id($id: expr) {
    eq!("_id", $id)
}
