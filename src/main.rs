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
use futures::Future;
use once_cell::sync::OnceCell;

mod database;
mod route;
mod structures;

async fn map_async<I, O, F>(v: Vec<I>, f: fn(I) -> F) -> Vec<O>
where
    I: Send,
    O: Send,
    F: Future<Output = O> + Send,
{
    let mut out = vec![];

    for i in v.into_iter() {
        out.push(f.call((i,)).await);
    }

    out
}

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
