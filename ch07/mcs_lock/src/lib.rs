use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{self, AtomicBool, AtomicPtr, Ordering},
};

pub struct MCSLock<T> {
    // 큐의 맨 마지막
    last: AtomicPtr<MCSNode<T>>,
    data: UnsafeCell<T>,
}

pub struct MCSNode<T> {
    next: AtomicPtr<MCSNode<T>>,
    locked: AtomicBool,
}

pub struct MCSLockGuard<'a, T> {
    node: &'a mut MCSNode<T>,
    mcs_lock: &'a MCSLock<T>,
}

unsafe impl<T> Sync for MCSLock<T> {}
unsafe impl<T> Send for MCSLock<T> {}

impl<T> MCSNode<T> {
    pub fn new() -> MCSNode<T> {
        MCSNode {
            next: AtomicPtr::new(ptr::null_mut()),
            locked: AtomicBool::new(false),
        }
    }
}

impl<T> Default for MCSNode<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MCSLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mcs_lock.data.get() }
    }
}

impl<T> DerefMut for MCSLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mcs_lock.data.get() }
    }
}

impl<T> MCSLock<T> {
    pub fn new(v: T) -> MCSLock<T> {
        MCSLock {
            last: AtomicPtr::new(ptr::null_mut()),
            data: UnsafeCell::new(v),
        }
    }

    pub fn lock<'a>(&'a self, node: &'a mut MCSNode<T>) -> MCSLockGuard<'a, T> {
        node.next = AtomicPtr::new(ptr::null_mut());
        node.locked = AtomicBool::new(false);

        let guard = MCSLockGuard {
            node,
            mcs_lock: self,
        };

        // 자신을 큐의 맨 마지막으로 한다.
        let ptr = guard.node as *mut MCSNode<T>;
        let prev = self.last.swap(ptr, Ordering::Acquire);

        if !prev.is_null() {
            guard.node.locked.store(true, Ordering::Relaxed);

            let prev = unsafe { &*prev };
            prev.next.store(ptr, Ordering::Relaxed);

            while guard.node.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }

        atomic::fence(Ordering::Acquire);

        guard
    }
}

impl<T> Drop for MCSLockGuard<'_, T> {
    fn drop(&mut self) {
        // 자신이 맨끝 노드일 때
        if self.node.next.load(Ordering::Relaxed).is_null() {
            let ptr = self.node as *mut MCSNode<T>;
            if self
                .mcs_lock
                .last
                .compare_exchange(ptr, ptr::null_mut(), Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
        }

        // 자신의 다음 스레드가 lock 함수를 실행중일 때때
        while self.node.next.load(Ordering::Relaxed).is_null() {
            std::hint::spin_loop();
        }

        // 자신의 다음 스레드를 실행 가능하게 설정한다.
        let next = unsafe { &mut *self.node.next.load(Ordering::Relaxed) };
        next.locked.store(false, Ordering::Release);
    }
}
