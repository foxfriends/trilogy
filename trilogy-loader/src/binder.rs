use crate::location::Location;
use crate::{Cache, Error, ErrorKind, Module};
use reqwest::{blocking::Client, Url};
use std::collections::{HashMap, VecDeque};
use std::fs;

#[derive(Clone, Default)]
pub struct Binder {
    modules: HashMap<Url, Module>,
}

impl Binder {
    pub fn modules(&self) -> &HashMap<Url, Module> {
        &self.modules
    }
}

#[derive(Clone)]
pub(crate) struct WipBinder<'a, E> {
    client: Client,
    cache: &'a dyn Cache<Error = E>,
    module_queue: VecDeque<Location>,
    modules: HashMap<Url, Module>,
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
            modules: HashMap::default(),
        }
    }

    fn download(&self, url: &Url) -> crate::Result<String> {
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

    fn load_source(&mut self, location: &Location) -> Result<String, crate::Error> {
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
    pub fn load(mut self, location: Location) -> crate::Result<Binder> {
        self.request(location);
        while let Some(location) = self.module_queue.pop_front() {
            let url = location.as_ref();
            if self.modules.contains_key(url) {
                continue;
            };
            let source = self.load_source(&location)?;
            let module = Module::new(location.clone(), &source);
            for import in module.imported_modules() {
                self.request(import);
            }
            self.modules.insert(location.into(), module);
        }
        Ok(Binder {
            modules: self.modules,
        })
    }
}
