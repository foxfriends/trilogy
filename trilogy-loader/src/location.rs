use std::path::Path;
use url::Url;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Location(Url);

impl Location {
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

impl From<Location> for Url {
    fn from(location: Location) -> Url {
        location.0
    }
}
