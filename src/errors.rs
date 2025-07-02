
use thiserror::{Error, self};


#[derive(Debug, Error)]
pub enum AppError {
    
    #[error("Configuring app failed: {0}")]
    ConfigError(String),

}



