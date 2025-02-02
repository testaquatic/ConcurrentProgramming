use std::{
    cell::UnsafeCell,
    sync::atomic::{self, AtomicUsize, Ordering},
};

pub struct TicketLock<T> {
    ticket: AtomicUsize,
    turn: AtomicUsize,
    data: UnsafeCell<T>,
}

pub struct TicketLockGuard<'a, T> {
    ticket_lock: &'a TicketLock<T>,
}

impl<T> TicketLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            ticket: AtomicUsize::new(0),
            turn: AtomicUsize::new(0),
            data: UnsafeCell::new(v),
        }
    }

    pub fn lock(&self) -> TicketLockGuard<T> {
        let ticket = self.ticket.fetch_add(1, Ordering::Relaxed);
        while self.turn.load(Ordering::Relaxed) != ticket {
            std::hint::spin_loop();
        }
        atomic::fence(Ordering::Acquire);
        TicketLockGuard { ticket_lock: self }
    }
}

impl<T> Drop for TicketLockGuard<'_, T> {
    fn drop(&mut self) {
        self.ticket_lock.turn.fetch_add(1, Ordering::Release);
    }
}
