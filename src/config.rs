
use actix_files::NamedFile;
use actix_web::{http::header::ContentType, http::StatusCode, web, Either, HttpResponse};
use actix_web::Responder;
use localshare::assets::StaticFile;
use log::{info, debug};
use std::path::PathBuf;

use crate::AppState;


// To prevent further env var changes
pub const LOCALSHARE_RMPASS: &str = "LOCALSHARE_RMPASS";
pub const LOCALSHARE_RMPASS_HEADER_STR: &str = "X-LocalShare-RMPASS";
pub const STATIC_DIRNAME : &str = "static";


pub fn configure_logger(level : log::LevelFilter) -> Result<(), log::SetLoggerError>{
    env_logger::builder()
    .write_style(env_logger::WriteStyle::Always)
    .default_format()
    .format_level(true)
    .format_indent(Some(4))
    .format_target(true)
    .filter_level(level)
    .try_init()
}

pub fn get_404_page() -> actix_web::Route {
    web::get().to(async |app_state : web::Data<AppState>| -> Either<_,_> {
            match NamedFile::open_async(app_state.static_dir.join(PathBuf::from(StaticFile::NotFound))).await {
                Ok(f) => {
                    Either::Left(f.customize().with_status(StatusCode::NOT_FOUND))
                }
                Err(e) => {
                    info!("default route html file error: {}", e);
                    Either::Right(HttpResponse::NotFound().content_type(ContentType::html()).body(
                r#"
                    <h1>Not Found</h1>
                    <p> We don't have such page</p1>
                    <p><a href="/"> Go home. </a></p>
                "#
            ))
                }
            }
        })
}

pub fn get_upload_page() -> actix_web::Route {
    web::get().to(async |app_state : web::Data<AppState>| ->  actix_web::Result<NamedFile>{
        match NamedFile::open_async(app_state.static_dir.join(PathBuf::from(StaticFile::Upload))).await {
            Ok(f) => {
                Ok(f)
            }
            Err(e) => {
                info!("upload route html file error: {}", e);
                Err(actix_web::error::ErrorInternalServerError("failed to open html file"))
            }
        }
    })
}

pub fn get_index_page() -> actix_web::Route {
    web::get().to(async |app_state : web::Data<AppState>| -> actix_web::Result<NamedFile> {
        match NamedFile::open_async(app_state.static_dir.join(PathBuf::from(StaticFile::Index))).await {
            Ok(f)=> Ok(f),
            Err(e) => {
                info!("index route html file error: {}", e);
                Err(actix_web::error::ErrorInternalServerError("failed to open html file"))
            }
        }
    })
}

pub fn load_rmpass_from_env() -> Option<String> {
    // Load .env
    match dotenvy::dotenv() {
        Ok(_) => {
            info!(".env loaded.");
        }
        Err(e) => {
            debug!(".env error: {}", e);
        }
    }
    if let Ok(v) = std::env::var(LOCALSHARE_RMPASS) {
        Some(v)
    } else {
        None
    }

}