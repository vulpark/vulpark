// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![feature(
    associated_type_bounds,
    async_closure,
    decl_macro,
    fn_traits,
    let_chains
)]

use database::Database;
use dotenv::dotenv;
use std::sync::OnceLock;
use ulid::Ulid;

mod database;
mod route;
mod structures;

macro map_async($v: expr, $out: expr, $f: expr) {
    for i in $v.into_iter() {
        $out.push($f.call((i,)).await);
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv();
    route::init().await
}

static DATABASE: OnceLock<Database> = OnceLock::new();

pub fn generate_ulid() -> String {
    Ulid::new().to_string()
}

pub async fn database() -> &'static Database {
    if let Some(database) = DATABASE.get() {
        return database;
    }
    let database = Database::create().await.unwrap();
    let _ = DATABASE.set(database);
    DATABASE.get().unwrap()
}
