use super::report::ReportBuilder;
use crate::cache::Cache;
use crate::location::Location;

#[cfg(feature = "async")]
use reqwest::Client;
#[cfg(not(feature = "async"))]
use reqwest::blocking::Client;

use source_span::Span;
use std::collections::{HashMap, VecDeque};
use std::fmt::{self, Display};
use std::fs;
use std::time::Instant;
use trilogy_parser::syntax::{DefinitionItem, Document, ModuleDefinition, StringLiteral};
use trilogy_parser::{Parse, Parser, Spanned};
use trilogy_scanner::Scanner;
use url::Url;

#[derive(Clone, Debug)]
struct Module {
    contents: Parse<Document>,
}

#[derive(Debug)]
pub(super) struct Error<E: std::error::Error> {
    pub(super) location: Location,
    pub(super) span: Span,
    pub(super) kind: ErrorKind<E>,
}

#[derive(Debug)]
pub(super) enum ErrorKind<E> {
    InvalidScheme(String),
    Network(reqwest::Error),
    Io(std::io::Error),
    Cache(E),
}

impl<E: std::error::Error> Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Cache(error) => write!(f, "{error}"),
            ErrorKind::Io(error) => write!(f, "{error}"),
            ErrorKind::Network(error) => write!(f, "{error}"),
            ErrorKind::InvalidScheme(scheme) => {
                write!(f, "invalid scheme in module location `{}`", scheme)
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for Error<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.kind.cause()
    }
}

impl<E: std::error::Error + 'static> ErrorKind<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            ErrorKind::Cache(e) => Some(e),
            ErrorKind::Network(e) => Some(e),
            ErrorKind::Io(e) => Some(e),
            _ => None,
        }
    }

    fn into_cause(self) -> Option<Box<dyn std::error::Error + 'static>> {
        match self {
            ErrorKind::Cache(e) => Some(Box::new(e)),
            ErrorKind::Network(e) => Some(Box::new(e)),
            ErrorKind::Io(e) => Some(Box::new(e)),
            _ => None,
        }
    }
}

impl Module {
    fn new(source: &str) -> Self {
        let time_parsing = Instant::now();
        let scanner = Scanner::new(source);
        let parser = Parser::new(scanner);
        let contents = parser.parse();
        log::trace!("module parsed: {:?}", time_parsing.elapsed());
        Self { contents }
    }

    fn imported_modules(&self) -> impl Iterator<Item = StringLiteral> + '_ {
        fn module_imported_modules(module_def: &ModuleDefinition) -> Vec<&StringLiteral> {
            module_def
                .definitions
                .iter()
                .flat_map(|def| match &def.item {
                    DefinitionItem::Module(module_def) => module_imported_modules(module_def),
                    DefinitionItem::ExternalModule(module_def) => vec![&module_def.locator],
                    _ => vec![],
                })
                .collect()
        }

        self.contents
            .ast()
            .definitions
            .iter()
            .flat_map(|def| match &def.item {
                DefinitionItem::Module(module_def) => module_imported_modules(module_def),
                DefinitionItem::ExternalModule(module_def) => vec![&module_def.locator],
                _ => vec![],
            })
            .cloned()
    }
}

#[derive(Clone)]
pub(super) struct Loader<'a, E> {
    client: Client, // TODO: generic resolver
    cache: &'a dyn Cache<Error = E>,
    libraries: &'a HashMap<Location, String>,
}

impl<'a, E> Loader<'a, E>
where
    E: std::error::Error + 'static,
{
    pub fn new(cache: &'a dyn Cache<Error = E>, libraries: &'a HashMap<Location, String>) -> Self {
        Self {
            client: Client::default(),
            libraries,
            cache,
        }
    }

    #[cfg(not(feature = "async"))]
    fn download(&self, url: &Url) -> Result<String, ErrorKind<E>> {
        self.client
            .get(url.clone())
            .header("Accept", "text/x-trilogy")
            .send()
            .map_err(ErrorKind::Network)?
            .text()
            .map_err(ErrorKind::Network)
    }

    #[cfg(feature = "async")]
    fn download(&self, url: &Url) -> Result<String, ErrorKind<E>> {
        tokio::runtime::Handle::current().block_on(async {
            self.client
                .get(url.clone())
                .header("Accept", "text/x-trilogy")
                .send()
                .await
                .map_err(ErrorKind::Network)?
                .text()
                .await
                .map_err(ErrorKind::Network)
        })
    }

    pub fn load_source(&self, location: &Location) -> Result<Option<String>, ErrorKind<E>> {
        log::debug!("locating module `{}`", location);
        if self.cache.has(location) {
            log::trace!("module cache hit");
            return Ok(Some(self.cache.load(location).map_err(ErrorKind::Cache)?));
        }
        let url = location.as_ref();
        match url.scheme() {
            "file" => Ok(Some(fs::read_to_string(url.path()).map_err(ErrorKind::Io)?)),
            "http" | "https" => {
                let source = self.download(url)?;
                self.cache
                    .save(location, &source)
                    .map_err(ErrorKind::Cache)?;
                Ok(Some(source))
            }
            "trilogy" => Ok(self.libraries.get(location).cloned()),
            scheme => Err(ErrorKind::InvalidScheme(scheme.to_owned())),
        }
    }
}

pub(super) fn load<C: Cache>(
    cache: &C,
    entrypoint: &Location,
    libraries: &HashMap<Location, String>,
    report: &mut ReportBuilder<C::Error>,
) -> Vec<(Location, Document)> {
    let mut modules = HashMap::new();
    let loader = Loader::new(cache, libraries);

    let mut module_queue = VecDeque::with_capacity(8);

    let source = match loader.load_source(entrypoint) {
        Ok(Some(source)) => source,
        Ok(None) => unreachable!(),
        Err(error) => {
            report.error(error.into_cause().unwrap().into());
            return vec![];
        }
    };
    let entrymodule = Module::new(&source);
    for import in entrymodule.imported_modules() {
        module_queue.push_back((entrypoint.clone(), import));
    }
    modules.insert(entrypoint.clone(), entrymodule);
    while let Some((from_location, locator)) = module_queue.pop_front() {
        let span = locator.span();
        let location = from_location.relative(locator.as_ref());
        let url = location.as_ref();
        if modules.contains_key(url) {
            continue;
        };
        let source = match loader.load_source(&location).map_err(|kind| {
            super::Error::resolution(
                from_location.clone(),
                Error {
                    span,
                    location: location.clone(),
                    kind,
                },
            )
        }) {
            Ok(Some(source)) => source,
            Ok(None) => continue,
            Err(error) => {
                report.error(error);
                continue;
            }
        };
        let module = Module::new(&source);
        for import in module.imported_modules() {
            module_queue.push_back((location.clone(), import));
        }
        modules.insert(location, module);
    }

    for (location, module) in &modules {
        for error in module.contents.errors() {
            report.error(super::Error::syntax(location.clone(), error.clone()))
        }
        for error in module.contents.warnings() {
            report.warning(super::Error::syntax(location.clone(), error.clone()))
        }
    }

    modules
        .into_iter()
        .map(|(k, item)| (k, item.contents.into_ast()))
        .collect()
}
