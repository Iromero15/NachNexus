use axum::{extract::Multipart, http::StatusCode, Json};
use serde_json::{json, Value};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

pub async fn upload_handler(mut multipart: Multipart) -> (StatusCode, Json<Value>) {
    // 1. Extraemos el archivo que viene en la petición
    while let Some(mut field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            continue;
        };

        // Le encajamos un ID único adelante al nombre original
        let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
        // Esta es la ruta donde Rust va a guardar el archivo (tu compu/Raspberry)
        let filepath = format!("./temp_uploads/{}", unique_name);
        let _ = tokio::fs::create_dir_all("./temp_uploads").await;

        // 2. Guardamos el archivo en el disco (chunk por chunk)
        let mut file = File::create(&filepath).await.expect("Fallo al crear el archivo físico");
        while let Some(chunk) = field.chunk().await.unwrap_or(None) {
            file.write_all(&chunk).await.unwrap();
        }

        println!("Archivo recibido y guardado en temporales: {}", unique_name);

        // 3. Le avisamos a ClamAV que lo escanee
        //let esta_limpio = escanear_con_clamav(&unique_name).await;

        let esta_limpio = true; //dejamos esto asi para saltear por ahora DESCOMENTAR A FUTURO

        if esta_limpio {
            // Acá a futuro haríamos el insert en la base de datos (Postgres)
            // marcándolo como "pending" para que lo apruebes vos.
            return (
                StatusCode::OK, 
                Json(json!({"mensaje": "Archivo limpio y en cuarentena esperando aprobación", "status": "ok"}))
            );
        } else {
            // Si salta el antivirus, lo borramos a la mierda para no correr riesgos
            let _ = tokio::fs::remove_file(&filepath).await;
            return (
                StatusCode::BAD_REQUEST, 
                Json(json!({"mensaje": "¡Alerta! Se detectó código malicioso. Archivo fulminado.", "status": "error"}))
            );
        }
    }

    (StatusCode::BAD_REQUEST, Json(json!({"mensaje": "No se encontró ningún archivo en la petición", "status": "error"})))
}

async fn escanear_con_clamav(filename: &str) -> bool {
    // Nos conectamos al contenedor de ClamAV por el puerto expuesto
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:3310").await {
        
        // Magia pura: Le pasamos la ruta INTERNA que ClamAV conoce (/scandir)
        let comando = format!("SCAN /scandir/{}\n", filename);
        let _ = stream.write_all(comando.as_bytes()).await;

        let mut buffer = [0; 1024];
        if let Ok(n) = stream.read(&mut buffer).await {
            let respuesta = String::from_utf8_lossy(&buffer[..n]);
            println!("Respuesta del patovica: {}", respuesta);
            
            // Si está todo bien, ClamAV responde algo como "/scandir/archivo.mp4: OK"
            return respuesta.contains("OK");
        }
    }
    
    println!("Fallo en la conexión con ClamAV o respuesta inesperada.");
    false // Si falla la conexión, asumimos que tiene bicho por las dudas
}