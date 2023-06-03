// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![feature(decl_macro, let_chains)]

use database::Database;
use once_cell::sync::OnceCell;

mod database;
mod route;
mod structures;

#[tokio::main]
async fn main() {
    route::init().await
}

static DATABASE: OnceCell<Database> = OnceCell::new();

pub async fn database() -> &'static Database {
    if let Some(database) = DATABASE.get() {
        return database;
    }
    let database = Database::create().await.unwrap();
    let _ = DATABASE.set(database);
    DATABASE.get().unwrap()
}
