use std::{
    borrow::Borrow,
    fmt::Display,
    path::{Path, PathBuf},
};
use url::Url;

/// A Location describes where to find a Trilogy source file.
///
/// Typically locations are created from `module` statements in Trilogy programs,
/// and correspond to files on your local file system or the Internet.
///
/// Internally, locations are represented as `Url`s, and so any location created
/// must be a valid URL .
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Location(Url);

impl<'a> From<&'a Location> for Location {
    fn from(loc: &'a Location) -> Self {
        loc.clone()
    }
}

impl Location {
    /// Creates a Location that corresponds to an externally provided library.
    ///
    /// Such libraries are prefixed with `trilogy:`, reference them in your Trilogy
    /// programs accordingly. For example, the Trilogy standard library is a library
    /// with name `std`, and is declared from Trilogy programs as `module std at "trilogy:std"`.
    ///
    /// # Errors
    ///
    /// This function will return an Err if the string formed by by prefixing the name with `trilogy:`
    /// cannot be parsed as a URL (with scheme `trilogy`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy::Location;
    /// let location = Location::library("std").unwrap();
    /// assert_eq!(location.to_string(), "trilogy:std");
    /// ```
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

    /// Returns `true` if this `Location` represents a file on the local file system.
    pub fn is_local(&self) -> bool {
        self.0.scheme() == "file"
    }

    /// If this `Location` represents a file on the local file system, returns the path to that file.
    /// Returns `None` otherwise.
    pub fn to_local_path(&self) -> Option<PathBuf> {
        self.0.to_file_path().ok()
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
