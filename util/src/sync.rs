use std::{sync::atomic::{Ordering, AtomicBool, AtomicU32}, marker::PhantomData};

/// An atomic flag that cannot be set to false once it has been set to true.
#[derive(Debug)]
pub struct AtomicFlag {
    flag: AtomicBool
}

impl AtomicFlag {
    pub fn new() -> Self {
        Self { flag: AtomicBool::new(false) }
    }

    pub fn set(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn get(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

impl Default for AtomicFlag {
    fn default() -> Self {
        Self::new()
    }
}