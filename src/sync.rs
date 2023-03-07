use core::{
    cell::UnsafeCell,
    fmt::{Debug, Display},
    hint,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Mutex<T> {
    locked: AtomicBool,
    inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            if self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break MutexGuard { mutex: self };
            }
            hint::spin_loop();
        }
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T: Debug> Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: Display> Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}
