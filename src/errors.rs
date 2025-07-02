
use thiserror::{Error, self};


#[derive(Debug, Error)]
pub enum AppError {
    
    #[error("Configuring app failed: {0}")]
    ConfigError(String),

    #[error("File couldn't removed: {0}")]
    RemoveError(String),

}



