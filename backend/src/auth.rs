use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use std::env;
use crate::db::DbPool;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub usuario: String,
    pub clave: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub mensaje: String,
    pub status: String,
}

pub async fn login_handler(
    axum::extract::State(pool): axum::extract::State<DbPool>, // Corregido aquí
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<Value>) {
    let jellyfin_url = env::var("JELLYFIN_URL").expect("Falta JELLYFIN_URL en el .env");

    // 1. Validar contra Jellyfin
    match autenticar_en_jellyfin(&payload.usuario, &payload.clave, &jellyfin_url).await {
        Ok(true) => {
            // 2. Si es válido, asegurar que existe en nuestra DB (Upsert)
            let res = sqlx::query!(
                "INSERT INTO users (username) VALUES ($1) 
                 ON CONFLICT (username) DO UPDATE SET username = EXCLUDED.username
                 RETURNING id",
                payload.usuario
            )
            .fetch_one(&pool)
            .await;

            match res {
                Ok(_) => (StatusCode::OK, Json(json!({"mensaje": "Login exitoso", "status": "ok"}))),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"mensaje": "Error en DB local", "status": "error"}))),
            }
        }
        Ok(false) => (StatusCode::UNAUTHORIZED, Json(json!({"mensaje": "Credenciales inválidas", "status": "error"}))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"mensaje": "Error de conexión", "status": "error"}))),
    }
}

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