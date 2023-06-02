#![feature(decl_macro)]

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
