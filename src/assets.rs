
use std::{io::Write, path::{Path, PathBuf}};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

pub enum StaticFile {
    Index,
    NotFound,
    Upload,
}



impl From<StaticFile> for PathBuf {
    fn from(value: StaticFile) -> Self {
        match value {
            StaticFile::Index => PathBuf::from("indexv2.html"),
            StaticFile::NotFound => PathBuf::from("not_found.html"),
            StaticFile::Upload => PathBuf::from("uploadv2.html")
        }
    }
}

pub struct Assets(Vec<PathBuf>);

impl Assets {
    pub fn new() -> Self {
        let mut assets : Vec<PathBuf> = Vec::new();
        assets.push(StaticFile::Index.into());
        assets.push(StaticFile::NotFound.into());
        assets.push(StaticFile::Upload.into());
        Self(assets)
    }
    pub fn extract_to_dir<P : AsRef<Path>>(&self ,dir : P) -> std::io::Result<()>{
        for asset in &self.0 {
            let embedded_file = StaticAssets::get(asset.to_str().unwrap()).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "embedded file not found"))?;
            
            let mut file = std::fs::File::create(dir.as_ref().join(&asset))?;
            file.write_all(&embedded_file.data)?;
        }
        Ok(())
    }
}
