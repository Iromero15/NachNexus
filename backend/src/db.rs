use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn crear_pool() -> DbPool {
    let db_url = env::var("DATABASE_URL").expect("Falta DATABASE_URL en el .env");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("No se pudo conectar a la base de datos")
}