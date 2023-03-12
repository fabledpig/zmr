use std::sync::atomic::AtomicUsize;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeId {
    value: usize,
}

impl RuntimeId {
    pub fn generate() -> Self {
        Self {
            value: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}
