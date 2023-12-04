use super::chunk::Line;
use crate::Value;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Type {
    Unit,
    Boolean,
    Character,
    Number,
    Bits,
    Atom,
    String,
    Tuple,
    Array,
    Record,
    Set,
    Struct,
    Callable,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum CellValue {
    Constant(Value),
    Typed(Type),
    Any,
    Unknown,
    Empty,
}

struct Cell {
    contents: CellValue,
}

#[derive(Default)]
struct Simulator {
    stack: Vec<Cell>,
    registers: Vec<Cell>,
}

pub(super) fn optimize(
    lines: Vec<Line>,
    _entrypoint: usize,
    _force_reachable: &[String],
) -> Vec<Line> {
    let simulator = Simulator::default();
    lines
}
