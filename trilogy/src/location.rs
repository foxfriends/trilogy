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

    pub(crate) fn absolute(url: Url) -> Self {
        Self::from(url)
    }

    pub(crate) fn entrypoint(root_dir: PathBuf, file: impl AsRef<Path>) -> Self {
        Location::local_absolute(root_dir.join(file))
    }

    pub(crate) fn local_absolute(path: impl AsRef<Path>) -> Self {
        Self::from(Url::from_file_path(path).unwrap())
    }

    // TODO: this should probably not be unwrapping so liberally
    pub(crate) fn relative(&self, path: &str) -> Self {
        let url = match path.parse::<Url>() {
            Ok(mut url) if url.scheme() == "file" => {
                url.set_scheme(self.0.scheme()).unwrap();
                url
            }
            Ok(url) => url,
            Err(..) => self.0.join(path.as_ref()).unwrap(),
        };
        Self::from(url)
    }
}

impl From<Url> for Location {
    fn from(mut url: Url) -> Self {
        let mut skip = 0;
        if let Some(path_segments) = url.path_segments() {
            let mut segments = vec![];
            for segment in path_segments.rev() {
                if segment == ".." {
                    skip += 1;
                    continue;
                }
                if segment == "." {
                    continue;
                }
                if skip > 0 {
                    skip -= 1;
                    continue;
                }
                segments.push(segment);
            }
            segments.reverse();
            url.set_path(&segments.join("/"));
        }
        Self(url)
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

#[cfg(test)]
mod tests {
    use super::Location;
    use std::path::PathBuf;

    #[test]
    fn location_entrypoint() {
        let location = Location::entrypoint(PathBuf::from("/a/b/c/"), "./d/e/f");
        assert_eq!(
            location,
            Location::absolute("file:///a/b/c/d/e/f".parse().unwrap())
        );
    }

    #[test]
    fn location_entrypoint_resolve() {
        let location = Location::entrypoint(PathBuf::from("/a/b/c/"), "../../d/e/f");
        assert_eq!(
            location,
            Location::absolute("file:///a/d/e/f".parse().unwrap())
        );
    }

    #[test]
    fn location_absolute_resolve() {
        let location = Location::absolute("file:///a/b/c/d/../e".parse().unwrap());
        assert_eq!(
            location,
            Location::absolute("file:///a/b/c/e".parse().unwrap())
        );
    }

    #[test]
    fn location_relative() {
        let location = Location::absolute("file:///a/b/c".parse().unwrap());
        assert_eq!(
            location.relative("./d"),
            Location::absolute("file:///a/b/d".parse().unwrap())
        );
    }

    #[test]
    fn location_relative_resolve() {
        let location = Location::absolute("file:///a/b/c".parse().unwrap());
        assert_eq!(
            location.relative("../d"),
            Location::absolute("file:///a/d".parse().unwrap())
        );
    }

    #[test]
    fn library() {
        let location = Location::library("std").unwrap();
        assert_eq!(location, Location::absolute("trilogy:std".parse().unwrap()));
    }
}
