use crate::cache::Cache;
use crate::location::Location;
use crate::LoadError;
use reqwest::blocking::Client;
use std::collections::{HashMap, VecDeque};
use std::fmt::{self, Display};
use std::fs;
use trilogy_parser::syntax::{DefinitionItem, Document, ModuleDefinition};
use trilogy_parser::{Parse, Parser};
use trilogy_scanner::Scanner;
use url::Url;

#[derive(Clone, Debug)]
pub struct Module {
    location: Location,
    contents: Parse<Document>,
}

#[derive(Debug)]
pub enum ResolverError<E: std::error::Error> {
    InvalidScheme(String),
    Cache(E),
    External(Box<dyn std::error::Error>),
}

impl<E: std::error::Error> ResolverError<E> {
    pub(super) fn external(e: impl std::error::Error + 'static) -> Self {
        Self::External(Box::new(e))
    }
}

impl<E: std::error::Error> Display for ResolverError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cache(error) => write!(f, "{error}"),
            Self::InvalidScheme(scheme) => {
                write!(f, "invalid scheme in module location `{}`", scheme)
            }
            Self::External(error) => write!(f, "{error}"),
        }
    }
}

impl<E: std::error::Error> std::error::Error for ResolverError<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Cache(e) => Some(e),
            Self::External(e) => Some(&**e),
            _ => None,
        }
    }
}

impl Module {
    pub fn new(location: Location, source: &str) -> Self {
        let scanner = Scanner::new(source);
        let parser = Parser::new(scanner);
        let contents = parser.parse();
        Self { location, contents }
    }

    pub fn imported_modules(&self) -> impl Iterator<Item = Location> + '_ {
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
struct Loader<'a, E> {
    client: Client, // TODO: generic resolver
    cache: &'a dyn Cache<Error = E>,
}

impl<'a, E> Loader<'a, E>
where
    E: std::error::Error + 'static,
{
    fn new(cache: &'a dyn Cache<Error = E>) -> Self {
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
            .map_err(ResolverError::external)?
            .text()
            .map_err(ResolverError::external)
    }

    fn load_source(&mut self, location: &Location) -> Result<Option<String>, ResolverError<E>> {
        if self.cache.has(location) {
            return Ok(Some(
                self.cache.load(location).map_err(ResolverError::Cache)?,
            ));
        }
        let url = location.as_ref();
        match url.scheme() {
            "file" => Ok(Some(
                fs::read_to_string(url.path()).map_err(ResolverError::external)?,
            )),
            "http" | "https" => {
                let source = self.download(url).map_err(ResolverError::external)?;
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

pub fn load<E: std::error::Error + 'static>(
    cache: &dyn Cache<Error = E>,
    entrypoint: &Location,
) -> Result<Vec<(Location, Document)>, LoadError<E>> {
    let mut modules = HashMap::new();
    let mut loader = Loader::new(cache);
    let mut module_queue = VecDeque::with_capacity(8);
    module_queue.push_back(entrypoint.clone());

    while let Some(location) = module_queue.pop_front() {
        let url = location.as_ref();
        if modules.contains_key(url) {
            continue;
        };
        let Some(source) = loader
            .load_source(&location)
            .map_err(|er| LoadError::Resolver(vec![er]))?
        else {
            continue;
        };
        let module = Module::new(location.clone(), &source);
        for import in module.imported_modules() {
            module_queue.push_back(import);
        }
        modules.insert(location, module);
    }

    // TODO: warnings are lost here...

    if modules.values().any(|module| module.contents.has_errors()) {
        Err(LoadError::Syntax(
            modules
                .values()
                .flat_map(|module| module.contents.errors())
                .cloned()
                .collect(),
        ))
    } else {
        Ok(modules
            .into_iter()
            .map(|(k, item)| (k, item.contents.into_ast()))
            .collect())
    }
}
