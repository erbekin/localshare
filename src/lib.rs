use std::{io::{self, ErrorKind}, path::{Path, PathBuf}};
use log::{info, error};
use tokio::{ io::AsyncWriteExt};
use tokio::fs;
pub use record::FileRecord;

mod errors;
use errors::AppError;
mod record;

const RECORD_FILENAME : &str = "file_records.json";
#[derive(Debug)]
pub struct AppConfig {
    upload_dir : PathBuf,
    create_dir_if_ne : bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { upload_dir: "uploads".into(), create_dir_if_ne: true }
    }
}
impl AppConfig {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn set_dir<P : AsRef<Path>>(mut self, dir : P) -> Self {
        self.upload_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Whether create directory if not exists
    pub fn create_dir(mut self, yes : bool) -> Self {
        self.create_dir_if_ne = yes;
        self
    }

    /// Set to default
    pub fn set_defaults(mut self) -> Self {
        self.upload_dir = "uploads".into();
        self.create_dir_if_ne = true;
        self
    }

    
}

pub struct FileManager {
    config : AppConfig,
    records : Vec<FileRecord>,
}

impl FileManager {

    // 
    pub fn from_config(config : AppConfig) -> Result<Self, AppError> {

        // Config directory
        match config.upload_dir.metadata() {
            Ok(meta) => {
                if !meta.is_dir() {
                    return Err(AppError::ConfigError(format!("The path {:?} is not a directory!",config.upload_dir)));
                } else {
                    info!("upload directory already exists");
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                if config.create_dir_if_ne {
                    info!("Creating upload directory: {}", config.upload_dir.display());
                    match std::fs::create_dir(&config.upload_dir) {
                        Ok(_) => {
                            info!("Upload directory created: {}", config.upload_dir.display());
                        }
                        Err(e) => {
                            error!("Failed to create upload directory: {:?}: {}", config.upload_dir, e);
                            return Err(AppError::ConfigError(format!("Failed to create upload directory: {:?}: {}", config.upload_dir, e)))
                        }
                    }
                }
                else {
                    return Err(AppError::ConfigError(format!("The directory {:?} does not exists, use '-p' option to create non-existent parent directories.", config.upload_dir)));
                }
            } 
            Err(e) => {
                return Err(AppError::ConfigError(format!("ConfigError: Unable to query path metadata: {}", e)));
            }
        }
        Ok(Self {
            config,
            records : Vec::new(),
        })

    }

    // init record file or load if exists some
    // if file is invalid format,
    // file will be removed.
    pub async fn init_or_read(&mut self)  -> io::Result<()> {
        let file_path = self.config.upload_dir.join(RECORD_FILENAME);
        let file_exists = match fs::metadata(&file_path).await {
            Ok(meta) if meta.is_file() => true,
            Err(_) | Ok(_)=> false,
        };

        if file_exists {
            match fs::read_to_string(&file_path).await {
                Ok(content) => {
                    match serde_json::from_str::<Vec<FileRecord>>(&content) {
                        Ok(parsed) => {
                            self.records = parsed;
                            info!("{}: read {} records",RECORD_FILENAME, self.records.len());
                            Ok(())
                        }
                        Err(e) => {
                            error!("{}: failed to deserialize, started empty. cause: {}",RECORD_FILENAME, e);
                            Err(io::Error::new(ErrorKind::InvalidInput, e))
                        }
                    }
                }
                Err(e) => {
                    error!("failed to read from {}: {}", e, RECORD_FILENAME);
                    Err(e)
                }
            }
        } else{
            info!("{} not found, started empty", RECORD_FILENAME);
            Ok(())
        }

        
    

    }

    pub fn upload_dir(&self) -> &Path {
        self.config.upload_dir.as_path()
    }
    // dump to file
    pub async fn save_records(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.records)?;
        let mut file = tokio::fs::File::create(self.join_path(RECORD_FILENAME)).await?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }


    pub fn add_record(&mut self, record: FileRecord) {
        self.records.push(record);
    }

    pub fn get_record(&self, id : &uuid::Uuid) -> Option<&FileRecord> {
        self.records.iter().find(|e| {
            e.id.eq(id)
        })
    }

    pub fn get_records(&self) -> &Vec<FileRecord> {
        &self.records
    }

    pub fn get_file_path(&self,record : &FileRecord) -> PathBuf {
        self.join_path(record.id.to_string())
    }

    fn join_path<P : AsRef<Path>>(&self, path : P) -> PathBuf {
        self.config.upload_dir.join(path)
    }
}


#[cfg(test)]
mod test {
   
}
