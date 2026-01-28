use std::path::{Path, PathBuf};

use chrono::Utc;
use rocket::{
    data::ToByteUnit, fs::NamedFile, response::{status::{self, Custom}, stream::{One, ReaderStream}, Responder}, routes, serde::json::Json, Data, Response, Rocket, State
};
use rocket::http::{hyper::header, ContentType, Status};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io, sync::Mutex};
use uuid::Uuid;

use crate::{
    assets::StaticFile,
    config::Config,
    fm::{FileManager, record::Record},
};

#[allow(dead_code)]
pub struct Server {
    wd: PathBuf,
    config: Config,
    fm: FileManager,
}

impl Server {
    pub fn new(workdir: &Path, config: Config) -> anyhow::Result<Self> {
        let fm = FileManager::new(workdir, config.clone())?;
        Ok(Self {
            wd: workdir.to_path_buf(),
            config: config,
            fm,
        })
    }

    pub async fn launch(self) -> anyhow::Result<()> {
        let _ = Rocket::build()
            .manage(Mutex::new(self))
            .mount(
                "/",
                routes![index, upload, route_api_list, route_api_upload, route_api_download],
            )
            .launch()
            .await?;
        Ok(())
    }
}

#[rocket::get("/")]
async fn index(server: &State<Mutex<Server>>) -> io::Result<NamedFile> {
    let static_dir = {
        let server = server.lock().await;
        server.wd.join(server.config.path.r#static.clone())
    };
    rocket::fs::NamedFile::open(static_dir.join(PathBuf::from(StaticFile::Index))).await
}
#[rocket::get("/upload")]
async fn upload(server: &State<Mutex<Server>>) -> io::Result<NamedFile> {
    let static_dir = {
        let server = server.lock().await;
        server.wd.join(server.config.path.r#static.clone())
    };
    rocket::fs::NamedFile::open(static_dir.join(PathBuf::from(StaticFile::Upload))).await
}
#[rocket::get("/api/list")]
async fn route_api_list(
    server: &State<Mutex<Server>>,
) -> std::result::Result<Json<Vec<Record>>, Status> {
    let list = {
        let mut server = server.lock().await;
        server.fm.get_all_records()
    };
    match list {
        Ok(list) => Ok(Json(list)),
        Err(e) => {
            log::error!("/api/list: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    id: Uuid,
}

#[rocket::post("/api/upload?<author>&<description>&<filename>", data = "<data>")]
async fn route_api_upload(
    server: &State<Mutex<Server>>,
    author: String,
    description: Option<String>,
    filename: String,
    data: Data<'_>,
) -> Result<Json<UploadResponse>, status::Custom<&'static str>> {
    let uuid = uuid::Uuid::new_v4();
    let record = Record {
        uuid: uuid.clone(),
        uploaded_at: Utc::now(),
        name: filename,
        description: description,
        author: author,
    };
    let uploads_dir: PathBuf = {
        let server_locked = server.lock().await;
        let p : PathBuf = server_locked.config.path.uploads.clone().into();
        server_locked.wd.join(p)
    };
    let p = uploads_dir.join(uuid.to_string());
    log::info!("/api/upload: writing file at: {}", p.display());
    let file = data
        .open(4.gibibytes())
        .into_file(&p)
        .await
        .map_err(|e| {
            log::error!("/api/upload: file write failed: {}", e);
            status::Custom(Status::InternalServerError, "io error")
        })?;
    if !file.is_complete() {
        log::error!("/api/upload: incomplete file upload, aborting.");
        return Err(Custom(Status::InsufficientStorage, "too large file"));
    }
    {
        let mut server_locked = server.lock().await;
        match server_locked.fm.insert_record(record) {
            Ok(_) => Ok(Json(UploadResponse { id: uuid })),
            Err(e) => {
                log::error!("/api/upload: db write failed: {}", e);
                Err(status::Custom(
                    Status::InternalServerError,
                    "db write failed",
                ))
            }
        }
    }
}

struct DownloadResponse {
    filename : String,
    stream : ReaderStream<One<File>>,
}
impl<'r, 'o : 'r> Responder<'r, 'o> for DownloadResponse {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Response::build()
            .header(ContentType::Binary)
            .raw_header(header::CONTENT_DISPOSITION.as_str(), format!("attachment; filename=\"{}\"", self.filename))
            .streamed_body(self.stream)
            .ok()
    }
}

#[rocket::get("/api/download/<file_uuid>")]
async fn route_api_download(
    server : &State<Mutex<Server>>,
    file_uuid : Uuid
) -> Result<DownloadResponse, Custom<&str>> {
    let record = {
        server.lock().await.fm.get_record_by_uuid(file_uuid)
            .map_err(|e| {
                log::error!("{}", e);
                Custom(Status::InternalServerError, "db query failed")
            })?
            .ok_or(Custom(Status::NotFound, "file record not found"))?
    };

    let uploads_dir : PathBuf = {
        let server = server.lock().await;
        server.wd.join(server.config.path.uploads.clone())
    };



    let file_path = uploads_dir.join(record.uuid.to_string());
    let file = File::open(&file_path)
        .await
        .map_err(|e| {
            log::error!("/api/download : {}", e);
            Custom(Status::NotFound, "could not open requested file")
        })?;
    let stream = ReaderStream::one(file);
    Ok(DownloadResponse {
        filename: record.name,
        stream
    })
}
