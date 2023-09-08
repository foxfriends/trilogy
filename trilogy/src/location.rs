use std::{
    borrow::Borrow,
    fmt::Display,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Location(Url);

impl Location {
    pub fn library(name: impl Display) -> Result<Self, url::ParseError> {
        let url = format!("trilogy:{name}").parse()?;
        Ok(Self(url))
    }

    pub(crate) fn entrypoint(root_dir: PathBuf, file: impl AsRef<Path>) -> Self {
        Location::local_absolute(root_dir.join(file))
    }

    pub(crate) fn local_absolute(path: impl AsRef<Path>) -> Self {
        Self(Url::from_file_path(path).unwrap())
    }

    // TODO: this should probably not be unwrapping so liberally
    pub(crate) fn relative(&self, path: &str) -> Self {
        match path.parse::<Url>() {
            Ok(mut url) if url.scheme() == "file" => {
                url.set_scheme(self.0.scheme()).unwrap();
                Self(url)
            }
            Ok(url) => Self(url),
            Err(..) => Self(self.0.join(path.as_ref()).unwrap()),
        }
    }
}

impl AsRef<Url> for Location {
    fn as_ref(&self) -> &Url {
        &self.0
    }
}

impl Borrow<Url> for Location {
    fn borrow(&self) -> &Url {
        &self.0
    }
}

impl From<Location> for Url {
    fn from(location: Location) -> Url {
        location.0
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
