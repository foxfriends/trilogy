use super::Cache;
use crate::location::Location;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct FileSystemCache {
    cache_dir: PathBuf,
}

impl FileSystemCache {
    pub fn new(cache_dir: impl AsRef<Path>) -> io::Result<Self> {
        if !cache_dir.as_ref().exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        Ok(Self {
            cache_dir: cache_dir.as_ref().to_owned(),
        })
    }

    fn cache_path(&self, location: &Location) -> Option<PathBuf> {
        let url = location.as_ref();
        match url.scheme() {
            "file" => Some(url.path().parse().unwrap()),
            "http" | "https" => {
                let host = url.host().expect("http(s) url should have a host");
                let dir = match url.port().filter(|&port| port != 80 && port != 443) {
                    Some(port) => format!("{host}:{port}"),
                    None => host.to_string(),
                };
                Some(self.cache_dir.join(dir).join(url.path()))
            }
            _ => None,
        }
    }
}

impl Cache for FileSystemCache {
    type Error = io::Error;

    fn has(&self, location: &Location) -> bool {
        self.cache_path(location)
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    fn load(&self, location: &Location) -> Result<String, Self::Error> {
        fs::read_to_string(self.cache_path(location).unwrap())
    }

    fn save(&self, location: &Location, source: &str) -> Result<(), Self::Error> {
        if let Some(path) = self.cache_path(location) {
            fs::write(path, source)?;
        }
        Ok(())
    }
}
