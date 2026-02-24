use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use std::env;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use crate::db::DbPool;

// Lo que nos manda el usuario para loguearse
#[derive(Deserialize)]
pub struct LoginRequest {
    pub usuario: String,
    pub clave: String,
}

// Lo que guardamos adentro de la "pulserita VIP" (el JWT)
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,       // El ID del usuario en tu base Postgres
    pub role: String,   // 'admin' o 'commonUser'
    pub exp: usize,     // Cuándo vence el token
}

pub async fn login_handler(
    axum::extract::State(pool): axum::extract::State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<Value>) {
    let jellyfin_url = env::var("JELLYFIN_URL").expect("Falta JELLYFIN_URL en el .env");
    let jwt_secret = env::var("JWT_SECRET").expect("Falta JWT_SECRET en el .env");

    // 1. Validar contra Jellyfin primero
    match autenticar_en_jellyfin(&payload.usuario, &payload.clave, &jellyfin_url).await {
        Ok(true) => {
            // 2. Upsert en Postgres (si no existe lo crea) y traemos su ID y ROL
            let user_record = sqlx::query!(
                "INSERT INTO users (username) VALUES ($1) 
                 ON CONFLICT (username) DO UPDATE SET username = EXCLUDED.username
                 RETURNING id, role::text as role",
                payload.usuario
            )
            .fetch_one(&pool)
            .await;

            match user_record {
                Ok(record) => {
                    // 3. Armamos el Token JWT válido por 24 horas
                    let expiracion = Utc::now()
                        .checked_add_signed(Duration::hours(24))
                        .expect("Error calculando fecha")
                        .timestamp() as usize;

                    let claims = Claims {
                        sub: record.id,
                        // Acá Postgres nos devuelve el enum como texto gracias al "::text" en la query
                        role: record.role.unwrap_or_else(|| "commonUser".to_string()), 
                        exp: expiracion,
                    };

                    // Firmamos el token con la clave secreta
                    let token = encode(
                        &Header::default(),
                        &claims,
                        &EncodingKey::from_secret(jwt_secret.as_ref())
                    ).unwrap();

                    // Le devolvemos el token al frontend
                    (StatusCode::OK, Json(json!({
                        "mensaje": "Login exitoso",
                        "status": "ok",
                        "token": token
                    })))
                },
                Err(e) => {
                    println!("Error en DB: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"mensaje": "Error en DB local", "status": "error"})))
                },
            }
        }
        Ok(false) => (StatusCode::UNAUTHORIZED, Json(json!({"mensaje": "Credenciales inválidas", "status": "error"}))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"mensaje": "Error de conexión con la bóveda", "status": "error"}))),
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