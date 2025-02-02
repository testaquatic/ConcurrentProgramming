use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{self, AtomicBool, AtomicUsize, Ordering},
};

const NUM_LOCK: usize = 8;

const MASK: usize = NUM_LOCK - 1;

pub struct FairLock<T> {
    waiting: Vec<AtomicBool>,
    lock: AtomicBool,
    turn: AtomicUsize,
    data: UnsafeCell<T>,
}

pub struct FairLockGuard<'a, T> {
    fair_lock: &'a FairLock<T>,
    idx: usize,
}

impl<T> FairLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            waiting: (0..NUM_LOCK).map(|_| AtomicBool::new(false)).collect(),
            lock: AtomicBool::new(false),
            turn: AtomicUsize::new(0),
            data: UnsafeCell::new(v),
        }
    }

    pub fn lock(&self, idx: usize) -> FairLockGuard<T> {
        assert!(idx < NUM_LOCK);
        self.waiting[idx].store(true, Ordering::Relaxed);

        loop {
            // 다른 스레드가 false를 설정한 경우 락 획득
            if !self.waiting[idx].load(Ordering::Relaxed) {
                break;
            }

            if !self.lock.load(Ordering::Relaxed)
                && self
                    .lock
                    .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                break;
            }
        }
        atomic::fence(Ordering::Acquire);
        FairLockGuard {
            fair_lock: self,
            idx,
        }
    }
}

impl<T> Drop for FairLockGuard<'_, T> {
    fn drop(&mut self) {
        let fl = self.fair_lock;

        fl.waiting[self.idx].store(false, Ordering::Relaxed);

        let turn = fl.turn.load(Ordering::Relaxed);
        let next = if turn == self.idx {
            (turn + 1) & MASK
        } else {
            turn
        };

        if fl.waiting[next].load(Ordering::Relaxed) {
            fl.turn.store(next, Ordering::Release);
            fl.waiting[next].store(false, Ordering::Release);
        } else {
            fl.turn.store((next + 1) & MASK, Ordering::Relaxed);
            fl.lock.store(false, Ordering::Release);
        }
    }
}

unsafe impl<T> Sync for FairLock<T> {}
unsafe impl<T> Send for FairLock<T> {}

impl<T> Deref for FairLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fair_lock.data.get() }
    }
}

impl<T> DerefMut for FairLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.fair_lock.data.get() }
    }
}

#[cfg(test)]
mod tests {}
