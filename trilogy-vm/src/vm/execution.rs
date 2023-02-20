use crate::cactus::Cactus;
use crate::runtime::Value;

#[derive(Clone, Debug, Default)]
pub(crate) struct Execution {
    pub cactus: Cactus<Value>,
    pub ip: usize,
}

impl Execution {
    pub fn new() -> Self {
        Self {
            cactus: Cactus::new(),
            ip: 0,
        }
    }

    pub fn branch(&mut self) -> Self {
        let branch = self.cactus.branch();
        Self {
            cactus: branch,
            ip: self.ip,
        }
    }
}
