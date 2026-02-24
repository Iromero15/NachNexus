mod db;
mod auth;
mod upload; // Sumamos el nuevo módulo

use axum::{extract::DefaultBodyLimit, routing::post, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = db::crear_pool().await;

    let app = Router::new()
        .route("/login", post(auth::login_handler))
        // Agregamos la ruta de subida y le sacamos el límite de tamaño
        .route("/upload", post(upload::upload_handler))
        .layer(DefaultBodyLimit::disable()) 
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Backend modular corriendo en {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}