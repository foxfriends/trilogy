use super::report::ReportBuilder;
use super::Error;
use crate::cache::Cache;
use crate::location::Location;
use reqwest::blocking::Client;
use std::collections::{HashMap, VecDeque};
use std::fmt::{self, Display};
use std::fs;
use trilogy_parser::syntax::{DefinitionItem, Document, ModuleDefinition};
use trilogy_parser::{Parse, Parser};
use trilogy_scanner::Scanner;
use url::Url;

#[derive(Clone, Debug)]
struct Module {
    location: Location,
    contents: Parse<Document>,
}

#[derive(Debug)]
pub(super) enum ResolverError<E: std::error::Error> {
    InvalidScheme(String),
    Network(reqwest::Error),
    Io(std::io::Error),
    Cache(E),
}

impl<E: std::error::Error> Display for ResolverError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cache(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::Network(error) => write!(f, "{error}"),
            Self::InvalidScheme(scheme) => {
                write!(f, "invalid scheme in module location `{}`", scheme)
            }
        }
    }
}

impl<E: std::error::Error> std::error::Error for ResolverError<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Cache(e) => Some(e),
            _ => None,
        }
    }
}

impl Module {
    fn new(location: Location, source: &str) -> Self {
        let scanner = Scanner::new(source);
        let parser = Parser::new(scanner);
        let contents = parser.parse();
        Self { location, contents }
    }

    fn imported_modules(&self) -> impl Iterator<Item = Location> + '_ {
        fn module_imported_modules(module_def: &ModuleDefinition) -> Vec<&str> {
            module_def
                .definitions
                .iter()
                .flat_map(|def| match &def.item {
                    DefinitionItem::Module(module_def) => module_imported_modules(module_def),
                    DefinitionItem::ExternalModule(module_def) => vec![module_def.locator.as_ref()],
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
                DefinitionItem::ExternalModule(module_def) => vec![module_def.locator.as_ref()],
                _ => vec![],
            })
            .map(|import| self.location.relative(import))
    }
}

#[derive(Clone)]
pub(super) struct Loader<'a, E> {
    client: Client, // TODO: generic resolver
    cache: &'a dyn Cache<Error = E>,
}

impl<'a, E> Loader<'a, E>
where
    E: std::error::Error + 'static,
{
    pub fn new(cache: &'a dyn Cache<Error = E>) -> Self {
        Self {
            client: Client::default(),
            cache,
        }
    }

    fn download(&self, url: &Url) -> Result<String, ResolverError<E>> {
        self.client
            .get(url.clone())
            .header("Accept", "text/x-trilogy")
            .send()
            .map_err(ResolverError::Network)?
            .text()
            .map_err(ResolverError::Network)
    }

    pub fn load_source(&self, location: &Location) -> Result<Option<String>, ResolverError<E>> {
        if self.cache.has(location) {
            return Ok(Some(
                self.cache.load(location).map_err(ResolverError::Cache)?,
            ));
        }
        let url = location.as_ref();
        match url.scheme() {
            "file" => Ok(Some(
                fs::read_to_string(url.path()).map_err(ResolverError::Io)?,
            )),
            "http" | "https" => {
                let source = self.download(url)?;
                self.cache
                    .save(location, &source)
                    .map_err(ResolverError::Cache)?;
                Ok(Some(source))
            }
            "trilogy" => Ok(None),
            scheme => Err(ResolverError::InvalidScheme(scheme.to_owned())),
        }
    }
}

pub(super) fn load<C: Cache>(
    cache: &C,
    entrypoint: &Location,
    report: &mut ReportBuilder<C::Error>,
) -> Vec<(Location, Document)> {
    let mut modules = HashMap::new();
    let loader = Loader::new(cache);

    let mut module_queue = VecDeque::with_capacity(8);
    module_queue.push_back((entrypoint.clone(), entrypoint.clone()));
    while let Some((from_location, location)) = module_queue.pop_front() {
        let url = location.as_ref();
        if modules.contains_key(url) {
            continue;
        };
        let source = match loader
            .load_source(&location)
            .map_err(|e| Error::resolution(from_location.clone(), e))
        {
            Ok(Some(source)) => source,
            Ok(None) => continue,
            Err(error) => {
                report.error(error);
                continue;
            }
        };
        let module = Module::new(location.clone(), &source);
        for import in module.imported_modules() {
            module_queue.push_back((location.clone(), import));
        }
        modules.insert(location, module);
    }

    for (location, module) in &modules {
        for error in module.contents.errors() {
            report.error(Error::syntax(location.clone(), error.clone()))
        }
        for error in module.contents.warnings() {
            report.warning(Error::syntax(location.clone(), error.clone()))
        }
    }

    modules
        .into_iter()
        .map(|(k, item)| (k, item.contents.into_ast()))
        .collect()
}
