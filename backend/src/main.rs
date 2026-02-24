use axum::{
    routing::post,
    Router,
    Json,
    http::StatusCode,
};
use dotenvy::dotenv;
use std::env;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

// 1. Definimos la estructura de lo que nos va a mandar el usuario (el frontend)
#[derive(Deserialize)]
struct LoginRequest {
    usuario: String,
    clave: String,
}

// 2. Tu función de Jellyfin intacta
async fn autenticar_en_jellyfin(usuario: &str, clave: &str, base_url: &str) -> Result<bool, reqwest::Error> {
    let client = Client::new();
    let auth_url = format!("{}/Users/AuthenticateByName", base_url);
    let payload = json!({ "Username": usuario, "Pw": clave });
    let auth_header = "MediaBrowser Client=\"NasRequestApp\", Device=\"RustBackend\", DeviceId=\"12345\", Version=\"1.0.0\"";

    let response = client.post(&auth_url)
        .header("X-Emby-Authorization", auth_header)
        .json(&payload)
        .send()
        .await?;

    Ok(response.status().is_success())
}

// 3. El "Handler" (El controlador que atiende la ruta /login)
async fn login_handler(Json(payload): Json<LoginRequest>) -> (StatusCode, Json<Value>) {
    let jellyfin_url = env::var("JELLYFIN_URL").expect("Falta JELLYFIN_URL en el .env");

    match autenticar_en_jellyfin(&payload.usuario, &payload.clave, &jellyfin_url).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({ "mensaje": "Login exitoso, bienvenido", "status": "ok" }))
        ),
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "mensaje": "Credenciales inválidas", "status": "error" }))
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "mensaje": "Error conectando con la bóveda de Jellyfin", "status": "error" }))
        ),
    }
}

// 4. Levantamos el servidor
#[tokio::main]
async fn main() {
    dotenv().ok();
    
    // Armamos las rutas de la API
    let app = Router::new()
        .route("/login", post(login_handler));

    println!("🚀 Backend de NAS arrancando en http://0.0.0.0:3000");

    // Le decimos que escuche en el puerto 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}