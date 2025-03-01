use std::{
    cell::UnsafeCell,
    hint,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

const NUM_THREAD: usize = 4;
const NUM_LOOP: usize = 100_000;

struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

struct SpinLockGuard<'a, T> {
    spin_lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    fn new(v: T) -> Self {
        SpinLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(v),
        }
    }

    fn lock(&self) -> SpinLockGuard<T> {
        loop {
            while self.lock.load(Ordering::Relaxed) {
                hint::spin_loop();
            }

            if self
                .lock
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        SpinLockGuard { spin_lock: self }
    }
}

unsafe impl<T> Sync for SpinLock<T> {}
unsafe impl<T> Send for SpinLock<T> {}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.spin_lock.lock.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.spin_lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.spin_lock.data.get() }
    }
}

fn main() {
    let lock = Arc::new(SpinLock::new(0));
    (0..NUM_THREAD)
        .map(|_| {
            let lock0 = lock.clone();

            thread::spawn(move || {
                (0..NUM_LOOP).for_each(|_| {
                    let mut data = lock0.lock();
                    *data += 1;
                });
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|t| t.join().unwrap());

    println!(
        "COUNT = {} (expected = {})",
        *lock.lock(),
        NUM_LOOP * NUM_THREAD
    );
}
