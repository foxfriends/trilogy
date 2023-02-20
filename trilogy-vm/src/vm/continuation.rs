use crate::cactus::Cactus;
use crate::runtime::Value;

#[derive(Clone, Debug, Default)]
pub(crate) struct Continuation {
    cactus: Cactus<Value>,
    ip: usize,
}

impl Continuation {
    pub fn new() -> Self {
        Self {
            cactus: Cactus::new(),
            ip: 0,
        }
    }
}
