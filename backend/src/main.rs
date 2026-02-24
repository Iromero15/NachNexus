mod db;
mod auth;

use axum::{routing::post, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Creamos el pool de conexión una sola vez
    let pool = db::crear_pool().await;

    // Pasamos el pool como "State" a las rutas que lo necesiten
    let app = Router::new()
        .route("/login", post(auth::login_handler))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Backend modular corriendo en {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}