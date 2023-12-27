use crate::cactus::{Cactus, Pointer, RangeMap};
use crate::callable::CallableKind;
use crate::vm::stack::StackCell;
use crate::{Execution, Value};
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone)]
pub(crate) struct GarbageCollector<'a> {
    cactus: &'a Cactus<StackCell>,
}

struct GarbageVisitor<'a> {
    cactus: &'a Cactus<StackCell>,
    reachable: RangeMap<bool>,
    visited: HashSet<usize>,
}

impl GarbageVisitor<'_> {
    fn visit_range(&mut self, range: Range<usize>) {
        let ranges = self
            .reachable
            .range(range.clone())
            .filter(|(_, val)| !val)
            .map(|(r, _)| r)
            .collect::<Vec<_>>();
        self.reachable.insert(range, true);
        for range in ranges {
            for i in range {
                if let Some(StackCell::Set(value)) = self.cactus.get(i) {
                    self.visit(&value);
                }
            }
        }
    }

    fn visit_pointer(&mut self, pointer: &Pointer<StackCell>) {
        for (range, v) in pointer.ranges().iter() {
            if v {
                self.visit_range(range);
            }
        }
    }

    fn visit(&mut self, value: &Value) {
        match value {
            Value::Array(array) if !self.visited.contains(&array.id()) => {
                self.visited.insert(array.id());
                for value in array {
                    self.visit(&value);
                }
            }
            Value::Set(set) if !self.visited.contains(&set.id()) => {
                self.visited.insert(set.id());
                for value in set {
                    self.visit(&value);
                }
            }
            Value::Record(record) if !self.visited.contains(&record.id()) => {
                self.visited.insert(record.id());
                for (key, value) in record {
                    self.visit(&key);
                    self.visit(&value);
                }
            }
            Value::Tuple(tuple) if !self.visited.contains(&tuple.id()) => {
                self.visited.insert(tuple.id());
                self.visit(tuple.first());
                self.visit(tuple.second());
            }
            Value::Struct(val) => self.visit(val.value()),
            Value::Callable(callable) => match &callable.0 {
                CallableKind::Closure(closure) if !self.visited.contains(&closure.id()) => {
                    self.visited.insert(closure.id());
                    self.visit_pointer(closure.stack_pointer());
                }
                CallableKind::Continuation(continuation)
                    if !self.visited.contains(&continuation.id()) =>
                {
                    self.visited.insert(continuation.id());
                    for frame in continuation.frames() {
                        self.visit_pointer(frame);
                    }
                    self.visit_pointer(continuation.stack_pointer());
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl<'a> GarbageCollector<'a> {
    pub fn new(cactus: &'a Cactus<StackCell>) -> Self {
        Self { cactus }
    }

    pub fn collect_garbage(&self, executions: &[Execution<'a>]) {
        let mut visitor = GarbageVisitor {
            cactus: self.cactus,
            visited: HashSet::new(),
            reachable: RangeMap::default(),
        };

        for ex in executions {
            for val in ex.registers() {
                visitor.visit(val);
            }
            let stack = ex.stack();
            for frame in stack.frames() {
                if let Some(slice) = &frame.slice {
                    visitor.visit_pointer(slice.pointer());
                }
            }
            let branch = stack.active();
            visitor.visit_pointer(branch.shared().pointer());
            for val in branch.locals() {
                if let Some(val) = val.as_set() {
                    visitor.visit(val);
                }
            }
        }

        let to_remove: usize = visitor
            .reachable
            .iter()
            .filter(|(_, v)| !v)
            .map(|(r, _)| r.len())
            .sum();
        if to_remove < self.cactus.len() / 4 {
            self.cactus.retain_ranges(visitor.reachable);
        } else {
            self.cactus.remove_ranges(visitor.reachable);
        }
    }
}
