use actix_web::{http::header::ContentDisposition, middleware::Logger, web, App, HttpResponse, HttpServer};
use clap::Parser;
use localshare::{AppConfig, FileManager, FileRecord};
use log::{info, error, warn};
use futures_util::StreamExt;
use serde::Deserialize;
use std::{str::FromStr, sync::Mutex};
use uuid::Uuid;
use tokio::io::AsyncWriteExt;

use crate::config::{get_404_page, get_upload_page, get_index_page};

mod config;
mod cli;

#[derive(Deserialize)]
struct DownloadRequest {
    id: Uuid,
}

struct AppState {
    file_manager : Mutex<FileManager>
}

async fn handle_download(
    download_request: web::Path<DownloadRequest>,
    app_state : web::Data<AppState>,
) -> actix_web::Result<actix_files::NamedFile> {
    info!("Download request for id : '{}'", download_request.id);

    let fm = app_state.file_manager.lock().unwrap();
    match fm.get_record(&download_request.id) {
        Some(record) => {

            let mut file = actix_files::NamedFile::open_async(&fm.get_file_path(record)).await.map_err(|e| {
                warn!("file with id {} should exists but io error occured: {}", record.id, e);
                actix_web::error::ErrorInternalServerError("failed to open file")
            })?.set_content_disposition(ContentDisposition::attachment(&record.name));
            
            match &record.content_type {
                Some(ct) => {
                    if let Ok(mime) = actix_web::mime::Mime::from_str(ct) {
                        file = file.set_content_type(mime);
                    } else {
                        warn!("content type of record {} is invalid!", record.id);
                    }
                }
                None => {}
            }
            Ok(file)
        }
        None => {
            Err(actix_web::error::ErrorNotFound("No such file"))
        }
    }
}


async fn handle_upload(
    mut payload: actix_multipart::Multipart,
    app_state : web::Data<AppState>
) -> actix_web::Result<web::Json<FileRecord>> {
    let mut file_data = None;
    let mut user_name = None;
    let mut description = None;

    // Parse multipart form data
    while let Some(field_result) = payload.next().await {
        let mut field = field_result.map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("Multipart error: {}", e))
        })?;

        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                let content_type = field.content_type().map(|f| {
                    f.to_string()
                });

                let original_filename = match field.content_disposition() {
                    None => String::from("unnamed"),
                    Some(cd) => cd.get_filename().unwrap_or("unnamed").to_string()
                };

                let mut file_bytes = Vec::new();

                while let Some(chunk_result) = field.next().await {
                    let chunk = chunk_result.map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!("Error reading file chunk: {}", e))
                    })?;
                    file_bytes.extend_from_slice(&chunk);

                }

                file_data = Some((file_bytes, original_filename, content_type));
            }
            "name" => {
                let mut name_bytes = Vec::new();
                while let Some(chunk_result) = field.next().await {
                    let chunk = chunk_result.map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!("Error reading name field: {}", e))
                    })?;
                    name_bytes.extend_from_slice(&chunk);
                }
                user_name = Some(String::from_utf8(name_bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid UTF-8 in name field: {}", e))
                })?);
            }
            "description" => {
                // Handle description field
                let mut desc_bytes = Vec::new();
                while let Some(chunk_result) = field.next().await {
                    let chunk = chunk_result.map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!("Error reading description field: {}", e))
                    })?;
                    desc_bytes.extend_from_slice(&chunk);
                }
                description = Some(String::from_utf8(desc_bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid UTF-8 in description field: {}", e))
                })?);
            }
            _ => {
                // Skip unknown fields
                while let Some(_) = field.next().await {}
            }
        }
    }

    // Validate required fields
    let (file_bytes, original_filename, content_type) = file_data
        .ok_or_else(|| actix_web::error::ErrorBadRequest("File field is required"))?;

    let description = description
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Description field is required"))?;

    if file_bytes.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("File cannot be empty"));
    }

    let file_id = Uuid::new_v4();

    let upload_path = {
        let file_manager = app_state.file_manager.lock().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to lock file manager: {}", e))
        })?;
        file_manager.upload_dir().join(file_id.to_string())
    };

    let mut file = tokio::fs::File::create(&upload_path).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create file: {}", e))
    })?;

    file.write_all(&file_bytes).await.map_err(|e| {
         actix_web::error::ErrorInternalServerError(format!("Failed to write file: {}", e))
    })?;

    file.flush().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to flush file: {}", e))
    })?;

     // Create file record
    let file_record = FileRecord {
        id: file_id,
        name: original_filename,
        by: user_name,
        uploaded_at: chrono::Utc::now(),
        description,
        content_type,
    };

    // Lock mutex and add record to file manager
    {
        let mut file_manager = app_state.file_manager.lock().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to lock file manager: {}", e))
        })?;
        file_manager.add_record(file_record.clone());
        file_manager.save_records().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to save file records: {}", e))
        })?;
    }

    // Return success response
    Ok(web::Json(file_record))


}




async fn handle_list(
    app_state : web::Data<AppState>
) -> actix_web::Result<web::Json<Vec<FileRecord>>> {
    let list = {
        let fm = app_state.file_manager.lock().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("unable to acquire lock: {}", e))
        })?;
        fm.get_records().clone()
    };
    Ok(web::Json(list))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let cli = cli::Cli::parse();
    info!("parsed args: {:#?}", cli);
    config::configure_logger(log::LevelFilter::Info).expect("Failed to init logger");
    info!(target : "app", "Logger initialized");

    let config = AppConfig::new().create_dir(cli.create_parent_dirs).set_dir(cli.dir);
    let mut fm = match FileManager::from_config(config) {
        Ok(f) => f,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    if let std::io::Result::Err(e) = fm.init_or_read().await {
        error!("Initializing file manager failed: {}", e);
        std::process::exit(1);
    }

    let file_manager = web::Data::new(AppState {
        file_manager : Mutex::new(fm)
    });



    HttpServer::new(move || {
        App::new()
            .app_data(file_manager.clone())
            .wrap(Logger::default())
            .route(
                "/",
                web::get().to(async || {
                    HttpResponse::Found().append_header(("Location", "/index.html")).finish()
                }),
            )
            .route("/index.html", get_index_page())
            .route("/upload.html", get_upload_page())
            .route("/api/list", web::get().to(handle_list))
            .route("api/upload", web::post().to(handle_upload))
            .route("api/download/{id}", web::get().to(handle_download))
            .route("/404", get_404_page())
            .default_service(config::get_404_page())
    })
    .workers(3)
    .bind(("0.0.0.0", cli.port))
    .expect("Failed to bind")
    .run()
    .await
}
