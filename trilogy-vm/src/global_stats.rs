use std::fmt::{self, Display};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct GlobalStats {
    pub records_allocated: AtomicUsize,
    pub records_freed: AtomicUsize,

    pub sets_allocated: AtomicUsize,
    pub sets_freed: AtomicUsize,

    pub arrays_allocated: AtomicUsize,
    pub arrays_freed: AtomicUsize,

    pub closures_allocated: AtomicUsize,
    pub closures_freed: AtomicUsize,

    pub continuations_allocated: AtomicUsize,
    pub continuations_freed: AtomicUsize,
}

impl GlobalStats {
    pub const fn new() -> Self {
        Self {
            records_allocated: AtomicUsize::new(0),
            records_freed: AtomicUsize::new(0),
            sets_allocated: AtomicUsize::new(0),
            sets_freed: AtomicUsize::new(0),
            arrays_allocated: AtomicUsize::new(0),
            arrays_freed: AtomicUsize::new(0),
            closures_allocated: AtomicUsize::new(0),
            closures_freed: AtomicUsize::new(0),
            continuations_allocated: AtomicUsize::new(0),
            continuations_freed: AtomicUsize::new(0),
        }
    }
}

impl Display for GlobalStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "--- Global Stats ---")?;
        writeln!(
            f,
            "      records: {:>8} free/{:>8} alloc",
            self.records_freed.load(Ordering::Relaxed),
            self.records_allocated.load(Ordering::Relaxed),
        )?;
        writeln!(
            f,
            "         sets: {:>8} free/{:>8} alloc",
            self.sets_freed.load(Ordering::Relaxed),
            self.sets_allocated.load(Ordering::Relaxed),
        )?;
        writeln!(
            f,
            "       arrays: {:>8} free/{:>8} alloc",
            self.arrays_freed.load(Ordering::Relaxed),
            self.arrays_allocated.load(Ordering::Relaxed),
        )?;
        writeln!(
            f,
            "     closures: {:>8} free/{:>8} alloc",
            self.closures_freed.load(Ordering::Relaxed),
            self.closures_allocated.load(Ordering::Relaxed),
        )?;
        write!(
            f,
            "continuations: {:>8} free/{:>8} alloc",
            self.continuations_freed.load(Ordering::Relaxed),
            self.continuations_allocated.load(Ordering::Relaxed),
        )
    }
}
