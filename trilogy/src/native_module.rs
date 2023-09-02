use std::collections::HashMap;
use trilogy_vm::{Native, NativeFunction};

#[derive(Clone)]
pub struct NativeModule {
    pub(crate) modules: HashMap<&'static str, NativeModule>,
    pub(crate) procedures: HashMap<&'static str, Native>,
}

#[derive(Clone)]
pub struct NativeModuleBuilder {
    inner: NativeModule,
}

impl Default for NativeModuleBuilder {
    fn default() -> Self {
        Self {
            inner: NativeModule {
                modules: Default::default(),
                procedures: Default::default(),
            },
        }
    }
}

impl NativeModuleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_submodule(mut self, name: &'static str, module: NativeModule) -> Self {
        self.inner.modules.insert(name, module);
        self
    }

    pub fn add_procedure<N: NativeFunction + 'static>(
        mut self,
        name: &'static str,
        proc: N,
    ) -> Self {
        self.inner.procedures.insert(name, proc.into());
        self
    }

    pub fn build(self) -> NativeModule {
        self.inner
    }
}
