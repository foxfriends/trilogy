use super::error::ErrorKind;
use super::execution::Step;
use super::program::ProgramReader;
use super::{Error, Execution, Stack};
use crate::atom::AtomInterner;
use crate::bytecode::ChunkError;
use crate::Value;
use crate::{Atom, Chunk, ChunkBuilder, Instruction, Program};
use std::collections::HashSet;

/// Interface to the Trilogy Virtual Machine.
///
/// This is a stack-based VM, but also with registers and heap.
/// Further documentation on the actual specifics will follow.
#[derive(Clone, Debug)]
pub struct VirtualMachine {
    atom_interner: AtomInterner,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
    /// Creates a new virtual machine with empty heap and no registers.
    pub fn new() -> Self {
        Self {
            atom_interner: AtomInterner::default(),
        }
    }

    /// Create an atom in the context of this VM.
    pub fn atom(&self, atom: &str) -> Atom {
        self.atom_interner.intern(atom)
    }

    /// Create an anonymous atom, that can never be recreated.
    pub fn atom_anon(&self, label: &str) -> Atom {
        Atom::new_unique(label.to_owned())
    }

    /// Compiles a program to bytecode, returning the compiled [`Chunk`][].
    pub fn compile(&self, program: &dyn Program) -> Result<Chunk, ChunkError> {
        let mut chunks_compiled = HashSet::new();
        let mut dependencies = vec![];

        let mut chunk_builder = ChunkBuilder::new(self.atom_interner.clone());
        program.entrypoint(&mut chunk_builder);
        let (_, mut chunk) = chunk_builder.build()?;
        loop {
            dependencies.extend(
                chunk
                    .iter()
                    .filter_map(|instruction| match instruction {
                        Instruction::Chunk(val) => Some(val),
                        _ => None,
                    })
                    .filter(|val| !chunks_compiled.contains(val)),
            );
            let Some(dep) = dependencies.pop() else {
                return Ok(chunk);
            };
            let mut chunk_builder = ChunkBuilder::new(self.atom_interner.clone());
            program.chunk(&dep, &mut chunk_builder);
            chunks_compiled.insert(dep);
            chunk_builder.build_from(&mut chunk)?;
        }
    }

    /// Run a [`Program`][] on this VM.
    ///
    /// This mutates the VM's internal state (heap and registers), so if reusing `VirtualMachine`
    /// instances, be sure this is the expected behaviour.
    pub fn run<P: Program>(&mut self, program: &P) -> Result<Value, Error> {
        self.run_with_registers(program, vec![])
    }

    /// Run a [`Program`][] on this VM.
    ///
    /// This mutates the VM's internal state (heap and registers), so if reusing `VirtualMachine`
    /// instances, be sure this is the expected behaviour.
    pub fn run_with_registers<P: Program>(
        &mut self,
        program: &P,
        registers: Vec<Value>,
    ) -> Result<Value, Error> {
        let program =
            ProgramReader::new(self.atom_interner.clone(), program).map_err(|err| Error {
                ip: 0,
                kind: ErrorKind::InvalidBytecode(err),
                stack_dump: Stack::default(),
            })?;
        let mut executions = vec![Execution::new(
            self.atom_interner.clone(),
            program,
            registers,
        )];
        // In future, multiple executions will likely be run in parallel on different
        // threads. Maybe this should be a compile time option, where the alternatives
        // are depth-first, or breadth-first multitasking.
        //
        // For now, the only option is depth-first multitasking, as it is easiest to
        // reason about and implement, as well as most likely to be performant in most
        // simple situations, which is all that we will have to deal with while this
        // VM remains a (relative) toy.
        let mut ep = 0;
        let last_ex = loop {
            let ex = &mut executions[ep];
            match ex.step()? {
                Step::Continue => {}
                Step::Suspend => {
                    ep = (ep + 1) % executions.len();
                }
                Step::End => {
                    let ex = executions.remove(ep);
                    ep %= executions.len();
                    if executions.is_empty() {
                        break ex;
                    }
                }
                Step::Spawn(ex) => executions.push(ex),
                Step::Exit(value) => return Ok(value),
            }
        };
        Err(last_ex.error(ErrorKind::ExecutionFizzledError))
    }
}
