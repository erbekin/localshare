use std::path::{Path, PathBuf};

use chrono::Utc;
use rocket::{FromForm};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::http::{ContentType, Status, hyper::header};
use rocket::response::Redirect;
use rocket::{
    Data, Response, Rocket, State,
    data::ToByteUnit,
    fs::NamedFile,
    response::{
        Responder,
        status::{self, Custom},
        stream::{One, ReaderStream},
    },
    routes,
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io, sync::Mutex};
use uuid::Uuid;

use crate::session::{SessionId, SessionStorage};
use crate::{
    assets::StaticFile,
    config::{self, Config},
    fm::{FileManager, record::Record},
};

#[allow(dead_code)]
pub struct Server {
    wd: PathBuf,
    config: Config,
    fm: FileManager,
    admin_password: Option<String>,
}

impl Server {
    pub fn new(workdir: &Path, config: Config) -> anyhow::Result<Self> {
        let fm = FileManager::new(workdir, config.clone())?;
        let admin_password = if config.app.auth {
            let password = std::env::var("LOCALSHARE_PASSWORD").map_err(|_| {
                anyhow::anyhow!(
                    "LOCALSHARE_PASSWORD environment variable is not set. \
                     Set it before starting the server in auth mode."
                )
            })?;
            if password.is_empty() {
                anyhow::bail!(
                    "LOCALSHARE_PASSWORD environment variable is set but empty. \
                     Provide a non-empty password."
                );
            }
            Some(password)
        } else {
            None
        };
        Ok(Self {
            wd: workdir.to_path_buf(),
            config: config,
            fm,
            admin_password,
        })
    }

    pub async fn launch(self) -> anyhow::Result<()> {
        let default_config = if self.config.app.debug {
            rocket::Config::debug_default()
        } else {
            rocket::Config::release_default()
        };
        let config = rocket::Config {
            port: self.config.app.port.parse()?,
            address: "0.0.0.0".parse().unwrap(),
            log_level: rocket::config::LogLevel::Normal,
            ..default_config
        };
        let _ = Rocket::custom(config)
            .manage(Mutex::new(self))
            .manage(Mutex::new(SessionStorage::new()))
            .mount(
                "/",
                routes![
                    index,
                    upload,
                    login_page,
                    qr,
                    route_api_list,
                    route_api_upload,
                    route_api_download,
                    route_api_delete,
                    route_api_login,
                    route_api_session,
                    route_api_auth,
                    route_api_logout
                ],
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

#[rocket::get("/login")]
async fn login_page(server: &State<Mutex<Server>>) -> io::Result<NamedFile> {
    let static_dir = {
        let server = server.lock().await;
        server.wd.join(server.config.path.r#static.clone())
    };
    rocket::fs::NamedFile::open(static_dir.join(PathBuf::from(StaticFile::Login))).await
}


#[rocket::get("/qr")]
async fn qr(server: &State<Mutex<Server>>) -> Result<NamedFile, Status> {
    let path_to_qr = {
        let server = server.lock().await;
        // $WORK_DIR/$static/$qr_filename
        server
            .wd
            .join(server.config.path.r#static.clone())
            .join(config::QR_ACCESS_FNAME)
    };
    NamedFile::open(&path_to_qr)
        .await
        .map_err(|_| Status::NotFound)
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
        let p: PathBuf = server_locked.config.path.uploads.clone().into();
        server_locked.wd.join(p)
    };
    let p = uploads_dir.join(uuid.to_string());
    log::info!("/api/upload: writing file at: {}", p.display());
    let file = data.open(4.gibibytes()).into_file(&p).await.map_err(|e| {
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
    filename: String,
    stream: ReaderStream<One<File>>,
}
impl<'r, 'o: 'r> Responder<'r, 'o> for DownloadResponse {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Response::build()
            .header(ContentType::Binary)
            .raw_header(
                header::CONTENT_DISPOSITION.as_str(),
                format!("attachment; filename=\"{}\"", self.filename),
            )
            .streamed_body(self.stream)
            .ok()
    }
}

#[rocket::get("/api/download/<file_uuid>")]
async fn route_api_download(
    server: &State<Mutex<Server>>,
    file_uuid: Uuid,
) -> Result<DownloadResponse, Custom<&str>> {
    let record = {
        server
            .lock()
            .await
            .fm
            .get_record_by_uuid(file_uuid)
            .map_err(|e| {
                log::error!("{}", e);
                Custom(Status::InternalServerError, "db query failed")
            })?
            .ok_or(Custom(Status::NotFound, "file record not found"))?
    };

    let uploads_dir: PathBuf = {
        let server = server.lock().await;
        server.wd.join(server.config.path.uploads.clone())
    };

    let file_path = uploads_dir.join(record.uuid.to_string());
    let file = File::open(&file_path).await.map_err(|e| {
        log::error!("/api/download : {}", e);
        Custom(Status::NotFound, "could not open requested file")
    })?;
    let stream = ReaderStream::one(file);
    Ok(DownloadResponse {
        filename: record.name,
        stream,
    })
}

#[rocket::delete("/api/delete/<file_uuid>")]
async fn route_api_delete(
    server: &State<Mutex<Server>>,
    _session: SessionId,
    file_uuid: Uuid,
) -> Result<Status, status::Custom<&'static str>> {
    // get requested record meta
    let record = {
        server
            .lock()
            .await
            .fm
            .get_record_by_uuid(file_uuid)
            .map_err(|e| {
                log::error!("FmError: {}", e);
                Custom(Status::InternalServerError, "database query failed")
            })?
            .ok_or(Custom(Status::NotFound, "file record not found"))?
    };
    let uploads_dir: PathBuf = {
        let server = server.lock().await;
        server.wd.join(server.config.path.uploads.clone())
    };
    tokio::fs::remove_file(uploads_dir.join(record.uuid.to_string()))
        .await
        .map_err(|e| {
            if matches!(e.kind(), std::io::ErrorKind::NotFound) {
                log::warn!(
                    "/api/delete: requested record doesnt exist in disk, but exists in database"
                );
            }
            Custom(Status::InternalServerError, "file removal failed")
        })?;
    server
        .lock()
        .await
        .fm
        .delete_record(file_uuid)
        .map_err(|_| Custom(Status::InternalServerError, "db delete failed"))?;
    Ok(Status::NoContent)
}

#[rocket::get("/api/login?<return_url>")]
async fn route_api_login(
    server: &State<Mutex<Server>>,
    session_storage: &State<Mutex<SessionStorage>>,
    cookies: &CookieJar<'_>,
    return_url: Option<String>,
) -> Redirect {
    let return_to = return_url.unwrap_or_else(|| "/".into());

    // Check if a valid session cookie already exists
    let has_valid_session = {
        if let Some(cookie) = cookies.get_private(config::SESSION_COOKIE_NAME) {
            if let Ok(session_id) = cookie.value().parse::<SessionId>() {
                session_storage.lock().await.contains(&session_id)
            } else {
                false
            }
        } else {
            false
        }
    };

    if has_valid_session {
        return Redirect::to(return_to);
    }

    let auth_enabled = {
        server.lock().await.admin_password.is_some()
    };

    if !auth_enabled {
        // Auth is off — auto-issue a session and grant access
        let session_id = SessionId::generate();
        session_storage.lock().await.insert(session_id.clone());
        cookies.add_private(Cookie::from(session_id));
        return Redirect::to(return_to);
    }

    // Auth is on — send to login page
    Redirect::to(format!("/login?return_url={}", return_to))
}

#[rocket::get("/api/session")]
async fn route_api_session(_session: SessionId) -> Status {
    Status::Ok
}


#[derive(FromForm)]
struct LoginForm {
    password: String,
    from: Option<String>,
}

#[rocket::post("/api/auth", data = "<form>")]
async fn route_api_auth(
    server: &State<Mutex<Server>>,
    session_storage: &State<Mutex<SessionStorage>>,
    cookies: &CookieJar<'_>,
    form: Form<LoginForm>,
) -> Result<Redirect, status::Custom<&'static str>> {
    let password_opt = {
        let s = server.lock().await;
        s.admin_password.clone()
    };
    let redirect_to = form.from.clone().unwrap_or_else(|| "/".into());

    match password_opt {
        Some(password) => {
            // POTENTIAL VULNERABILITY: timing attack possible, fix later
            if password == form.password {
                let session_id = SessionId::generate();
                session_storage.lock().await.insert(session_id.clone());
                cookies.add_private(Cookie::from(session_id));
                Ok(Redirect::to(redirect_to))
            } else {
                Err(Custom(Status::Unauthorized, "wrong password"))
            }
        }
        None => Ok(Redirect::to(redirect_to)),
    }
}


#[rocket::post("/api/logout?<return_url>")]
async fn route_api_logout(
    session_id : SessionId,
    session_storage: &State<Mutex<SessionStorage>>,
    cookies: &CookieJar<'_>,
    return_url: Option<String>) -> Redirect {
        session_storage.lock().await.remove(&session_id);
        cookies.remove_private(Cookie::from(session_id));
        if let Some(url) = return_url {
            Redirect::to(url)
        } else {
            Redirect::to("/")
        }
}
