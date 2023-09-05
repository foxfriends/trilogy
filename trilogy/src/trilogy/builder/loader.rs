use crate::cache::Cache;
use crate::location::Location;
use crate::LoadError;
use reqwest::blocking::Client;
use std::collections::{HashMap, VecDeque};
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
    module_queue: VecDeque<Location>,
}

impl<'a, E> Loader<'a, E>
where
    E: std::error::Error + 'static,
{
    fn new(cache: &'a dyn Cache<Error = E>) -> Self {
        Self {
            client: Client::default(),
            cache,
            module_queue: VecDeque::default(),
        }
    }

    fn download(&self, url: &Url) -> Result<String, LoadError<E>> {
        self.client
            .get(url.clone())
            .header("Accept", "text/x-trilogy")
            .send()
            .map_err(LoadError::external)?
            .text()
            .map_err(LoadError::external)
    }

    fn request(&mut self, location: Location) {
        self.module_queue.push_back(location);
    }

    fn load_source(&mut self, location: &Location) -> Result<Option<String>, LoadError<E>> {
        if self.cache.has(location) {
            return Ok(Some(self.cache.load(location).map_err(LoadError::Cache)?));
        }
        let url = location.as_ref();
        match url.scheme() {
            "file" => Ok(Some(
                fs::read_to_string(url.path()).map_err(LoadError::external)?,
            )),
            "http" | "https" => {
                let source = self.download(url).map_err(LoadError::external)?;
                self.cache
                    .save(location, &source)
                    .map_err(LoadError::Cache)?;
                Ok(Some(source))
            }
            "trilogy" => Ok(None),
            scheme => Err(LoadError::InvalidScheme(scheme.to_owned())),
        }
    }
}

pub fn load<E: std::error::Error + 'static>(
    cache: &dyn Cache<Error = E>,
    entrypoint: &Location,
) -> Result<HashMap<Location, Document>, LoadError<E>> {
    let mut modules = HashMap::new();
    let mut loader = Loader::new(cache);
    loader.request(entrypoint.clone());

    while let Some(location) = loader.module_queue.pop_front() {
        let url = location.as_ref();
        if modules.contains_key(url) {
            continue;
        };
        let Some(source) = loader.load_source(&location)? else {
            continue;
        };
        let module = Module::new(location.clone(), &source);
        for import in module.imported_modules() {
            loader.request(import);
        }
        modules.insert(location, module);
    }

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
