use core::sync::atomic::spin_loop_hint;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

///
///
/// RwLock that blocks until yielding
pub struct ERwLock<T: ?Sized>(RwLock<T>);

impl<T> ERwLock<T> {
    pub fn new(t: T) -> ERwLock<T> {
        ERwLock(RwLock::new(t))
    }
}

impl<T: ?Sized> ERwLock<T> {
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<T> {
        loop {
            match self.0.try_read() {
                Ok(guard) => break guard,
                _ => spin_loop_hint(),
            }
        }
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<T> {
        loop {
            match self.0.try_write() {
                Ok(guard) => break guard,
                _ => spin_loop_hint(),
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn inner(&self) -> &RwLock<T> {
        &self.0
    }
}
