use crate::location::Location;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub trait Cache {
    type Error;

    fn has(&self, location: &Location) -> bool;
    fn load(&self, location: &Location) -> Result<String, Self::Error>;
    fn save(&self, location: &Location, source: &str) -> Result<(), Self::Error>;
}

pub struct FileSystemCache {
    cache_dir: PathBuf,
}

impl FileSystemCache {
    pub fn new(cache_dir: impl AsRef<Path>) -> io::Result<Self> {
        let metadata = cache_dir.as_ref().metadata()?;
        // TODO: determine if this will detect and fail if there is a file in the way
        if !metadata.is_dir() {
            fs::create_dir_all(&cache_dir)?;
        }
        Ok(Self {
            cache_dir: cache_dir.as_ref().to_owned(),
        })
    }

    fn cache_path(&self, location: &Location) -> PathBuf {
        let url = location.as_ref();
        match url.scheme() {
            "file" => url.path().parse().unwrap(),
            "http" | "https" => {
                let host = url.host().expect("http(s) url should have a host");
                let dir = match url.port().filter(|&port| port != 80 && port != 443) {
                    Some(port) => format!("{host}:{port}"),
                    None => host.to_string(),
                };
                self.cache_dir.join(dir).join(url.path())
            }
            _ => unimplemented!(
                "only file, http, and https are valid schemes for loading Trilogy modules"
            ),
        }
    }
}

impl Cache for FileSystemCache {
    type Error = io::Error;

    fn has(&self, location: &Location) -> bool {
        self.cache_path(location).exists()
    }

    fn load(&self, location: &Location) -> Result<String, Self::Error> {
        fs::read_to_string(self.cache_path(location))
    }

    fn save(&self, location: &Location, source: &str) -> Result<(), Self::Error> {
        fs::write(self.cache_path(location), source)
    }
}

pub struct NoopCache;

impl Cache for NoopCache {
    type Error = std::convert::Infallible;

    fn has(&self, _location: &Location) -> bool {
        false
    }

    fn load(&self, _location: &Location) -> Result<String, Self::Error> {
        unimplemented!()
    }

    fn save(&self, _location: &Location, _source: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
