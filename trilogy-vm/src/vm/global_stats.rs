use std::sync::atomic::AtomicUsize;

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
