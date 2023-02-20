use crate::cactus::Cactus;

#[derive(Clone, Debug)]
pub struct VirtualMachine {
    cactus: Cactus<()>,
}
