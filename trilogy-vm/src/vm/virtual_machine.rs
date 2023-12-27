use super::error::ErrorKind;
use super::execution::Step;
use super::program_reader::ProgramReader;
use super::stack::{StackCell, StackDump, StackTrace};
use super::{Error, Execution};
use crate::atom::AtomInterner;
use crate::bytecode::ChunkError;
use crate::cactus::Cactus;
use crate::gc::GarbageCollector;
use crate::{Atom, Chunk, ChunkBuilder, Instruction, Program, Value};
use std::collections::HashSet;

#[cfg(feature = "stats")]
use super::Stats;
#[cfg(feature = "stats")]
use crate::RefCount;

/// Interface to the Trilogy Virtual Machine.
///
/// This is a stack-based VM, but also with registers and heap.
/// Further documentation on the actual specifics will follow.
#[derive(Clone, Debug)]
pub struct VirtualMachine {
    atom_interner: AtomInterner,
    #[cfg(feature = "stats")]
    stats: RefCount<Stats>,
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
            #[cfg(feature = "stats")]
            stats: Default::default(),
        }
    }

    /// Reports statistics about the execution of the program.
    ///
    /// This method is only available with feature `stats` enabled, which has
    /// a not-insignificant performance penalty to record the stats.
    #[cfg(feature = "stats")]
    pub fn stats(&self) -> RefCount<Stats> {
        self.stats.clone()
    }

    /// Create an atom in the context of this VM.
    ///
    /// See [`Atom`][] for more details.
    pub fn atom(&self, atom: &str) -> Atom {
        self.atom_interner.intern(atom)
    }

    /// Create an anonymous atom, that can never be recreated.
    ///
    /// See [`Atom`][] for more details.
    pub fn atom_anon(&self, label: &str) -> Atom {
        Atom::new_unique(label.to_owned())
    }

    /// Compiles a program to bytecode, returning the compiled [`Chunk`][].
    ///
    /// As each chunk is loaded, its bytecode will be scanned for any further
    /// chunk references which will be loaded immediately. Those loaded pieces will
    /// be concatenated into the resulting single chunk.
    ///
    /// This chunk can be printed to a file and later run on another VM, given
    /// you can write a suitable [`Program`][] implementation to load it back in.
    pub fn compile<P: Program>(&self, program: &P) -> Result<Chunk, ChunkError> {
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

    /// Run a [`Program`][] on this VM, returning the [Value][] it exits with.
    ///
    /// When run in this way, the program will be run with 0 registers. If your program
    /// uses registers, use [`run_with_registers`][VirtualMachine::run_with_registers]
    /// instead to provide the initial values of those registers.
    pub fn run<P: Program>(&self, program: &P) -> Result<Value, Error> {
        self.run_with_registers(program, vec![])
    }

    /// Run a [`Program`][] on this VM using the provided initial register state. Returns
    /// the [Value][] the program exits with.
    pub fn run_with_registers<P: Program>(
        &self,
        program: &P,
        registers: Vec<Value>,
    ) -> Result<Value, Error> {
        let stack = Cactus::<StackCell>::with_capacity(8192);

        let program =
            ProgramReader::new(self.atom_interner.clone(), program).map_err(|err| Error {
                ip: 0,
                kind: ErrorKind::InvalidBytecode(err),
                stack_trace: StackTrace::default(),
                stack_dump: StackDump::default(),
            })?;
        let mut executions = vec![Execution::new(
            self.atom_interner.clone(),
            program,
            stack.branch(),
            registers,
            #[cfg(feature = "stats")]
            self.stats.clone(),
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
        let mut gc_threshold = stack.capacity() * 3 / 4;
        let last_ex = loop {
            let ex = &mut executions[ep];
            match ex.step()? {
                Step::Continue => {}
                Step::Suspend => {
                    log::debug!("execution suspended ({})", ep);
                    ep = (ep + 1) % executions.len();
                }
                Step::End => {
                    log::debug!("execution ended ({})", ep);
                    let ex = executions.remove(ep);
                    if executions.is_empty() {
                        break ex;
                    }
                    ep %= executions.len();
                }
                Step::Spawn(ex) => {
                    log::debug!("execution spawned");
                    executions.push(ex);
                }
                Step::Exit(value) => return Ok(value),
            }
            if stack.len() > gc_threshold {
                let gc = GarbageCollector::new(&stack);
                gc.collect_garbage(&executions);
                gc_threshold = (stack.capacity() - stack.len()) * 3 / 4 + stack.len();
            }
        };
        Err(last_ex.error(ErrorKind::ExecutionFizzledError))
    }
}
