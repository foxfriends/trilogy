use super::location::Location;
use super::{Binder, Cache, Error, ErrorKind, Module};
use reqwest::blocking::Client;
use std::collections::VecDeque;
use std::fs;
use trilogy_parser::syntax::Document;
use trilogy_parser::Parse;
use url::Url;

#[derive(Clone)]
pub(crate) struct WipBinder<'a, E> {
    client: Client,
    cache: &'a dyn Cache<Error = E>,
    module_queue: VecDeque<Location>,
}

impl<'a, E> WipBinder<'a, E>
where
    E: std::error::Error + 'static,
{
    pub fn new(cache: &'a dyn Cache<Error = E>) -> Self {
        Self {
            client: Client::default(),
            cache,
            module_queue: VecDeque::default(),
        }
    }

    fn download(&self, url: &Url) -> super::Result<String> {
        self.client
            .get(url.clone())
            .header("Accept", "text/x-trilogy")
            .send()
            .map_err(Error::inaccessible)?
            .text()
            .map_err(Error::invalid)
    }

    fn request(&mut self, location: Location) {
        self.module_queue.push_back(location);
    }

    fn load_source(&mut self, location: &Location) -> Result<String, super::Error> {
        if self.cache.has(location) {
            return self.cache.load(location).map_err(Error::cache);
        }
        let url = location.as_ref();
        match url.scheme() {
            "file" => Ok(fs::read_to_string(url.path()).map_err(Error::inaccessible)?),
            "http" | "https" => {
                let source = self.download(url)?;
                self.cache.save(location, &source).map_err(Error::cache)?;
                Ok(source)
            }
            _ => Err(Error::from(ErrorKind::InvalidLocation)),
        }
    }

    // TODO: multithreading
    pub fn load(mut self, entrypoint: Location) -> super::Result<Binder<Parse<Document>>> {
        self.request(entrypoint.clone());
        let mut binder = Binder::new(entrypoint);
        while let Some(location) = self.module_queue.pop_front() {
            let url = location.as_ref();
            if binder.modules.contains_key(url) {
                continue;
            };
            let source = self.load_source(&location)?;
            let module = Module::new(location.clone(), &source);
            for import in module.imported_modules() {
                self.request(import);
            }
            binder.modules.insert(location.into(), module);
        }
        Ok(binder)
    }
}
