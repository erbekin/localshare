use serde::{Deserialize, Serialize};

pub const DB_NAME: &str = "localshare.db";
pub const UPLOAD_DIR: &str = "uploads";
pub const DEFAULT_PORT: &str = "8080";
pub const STATIC_DIR: &str = "static";
pub const CONFIG_FNAME: &str = "LocalShare.toml";
pub const QR_ACCESS_FNAME : &str = "qr_access.png";
pub const SESSION_COOKIE_NAME : &str = "session_id";


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    pub app: AppConfig,
    pub path: PathConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    // In your TOML, port is "8080" (string).
    // If you remove quotes in TOML, change this to u16.
    pub port: String,
    pub debug: bool,
    pub auth : bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathConfig {
    pub db: String,
    pub uploads: String,
    pub r#static : String,
}

// 2. Default implementation for your 'new' command
impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            app: AppConfig {
                port: DEFAULT_PORT.to_string(),
                debug: true,
                auth: false,
            },
            path: PathConfig {
                db: DB_NAME.to_string(),
                uploads: "uploads".to_string(),
                r#static: STATIC_DIR.to_string(),
            },
        }
    }
}


pub mod utils {
    use std::path::Path;
    use anyhow::Context;
    use tokio::{fs, io};

    use crate::config::CONFIG_FNAME;

    impl super::Config {
        pub async fn write_path(self, root: &Path) -> io::Result<()> {
            let toml_string = toml::to_string_pretty(&self).map_err(|e| io::Error::other(e))?;
            let file_path = root.join(CONFIG_FNAME);
            fs::write(file_path, toml_string).await?;
            Ok(())
        }
        pub async fn read_path(root: &Path) -> anyhow::Result<Self> {
            let file_path = root.join(CONFIG_FNAME);

            // 1. Read the file asynchronously
            let content = fs::read_to_string(&file_path)
                .await
                .context(format!("Could not read config file at {:?}", file_path))?;

            // 2. Deserialize from string
            let config: Self =
                toml::from_str(&content).context("Failed to parse Localshare.toml syntax")?;



            Ok(config)
        }
    }
}
