
use rust_embed::RustEmbed;
use tokio::io::{self, AsyncWriteExt};
use std::path::{Path, PathBuf};
#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;


pub enum StaticFile {
    Index,
    Upload,
}



impl From<StaticFile> for PathBuf {
    fn from(value: StaticFile) -> Self {
        match value {
            StaticFile::Index => PathBuf::from("index.html"),
            // StaticFile::NotFound => PathBuf::from("not_found.html"),
            StaticFile::Upload => PathBuf::from("upload.html")
        }
    }
}

/// Used for extracting embedded static files.
pub struct Assets(Vec<PathBuf>);

impl Assets {
    pub fn new() -> Self {
        let mut assets : Vec<PathBuf> = Vec::new();
        assets.push(StaticFile::Index.into());
        // assets.push(StaticFile::NotFound.into());
        assets.push(StaticFile::Upload.into());
        Self(assets)
    }

    /// Checks the consistency between embedded files and registered files.
    /// If they dont match or some missing returns error.
    pub fn check_consistency(&self) -> anyhow::Result<()> {
        use std::collections::HashSet;

        // Registered files
        let registered: HashSet<String> = self
            .0
            .iter()
            .map(|p| {
                p.to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 path: {:?}", p))
                    .map(String::from)
            })
            .collect::<Result<_, _>>()?;

        // Embedded files
        let embedded: HashSet<String> = StaticAssets::iter()
            .map(|p| p.to_string())
            .collect();

        // Missing embedded files
        let missing: Vec<_> = registered
            .difference(&embedded)
            .cloned()
            .collect();

        if !missing.is_empty() {
            anyhow::bail!("Missing embedded files: {:?}", missing);
        }

        // Unexpected embedded files
        let extra: Vec<_> = embedded
            .difference(&registered)
            .cloned()
            .collect();

        if !extra.is_empty() {
            anyhow::bail!("Unregistered embedded files found: {:?}", extra);
        }

        Ok(())
    }

    pub async fn extract_to_dir<P : AsRef<Path>>(&self ,dir : P) -> io::Result<()>{
        for asset in &self.0 {
            let embedded_file = StaticAssets::get(asset.to_str().unwrap()).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "embedded file not found"))?;
            let mut file = tokio::fs::File::create(dir.as_ref().join(&asset)).await?;
            file.write_all(&embedded_file.data).await?;
        }
        Ok(())
    }
}
