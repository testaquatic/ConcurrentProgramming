#![allow(static_mut_refs)]

use std::{
    ptr::{read_volatile, write_volatile},
    sync::atomic::{fence, Ordering},
    thread,
};

const NUM_THREADS: usize = 4;
const NUM_LOOP: usize = 100_000;

macro_rules! read_mem {
    ($addr: expr) => {
        unsafe { std::ptr::read_volatile($addr) }
    };
}

macro_rules! write_mem {
    ($addr: expr, $val: expr) => {
        unsafe { std::ptr::write_volatile($addr, $val) }
    };
}

struct BakeryLock {
    // i 번째 스레드가 티켓을 획득 중이면 entering[i]는 true
    entering: [bool; NUM_THREADS],
    // i번째 스레드의 티켓은 ticket[i]
    tickets: [Option<u64>; NUM_THREADS],
}

impl BakeryLock {
    fn lock(&mut self, idx: usize) -> LockGuard {
        fence(Ordering::SeqCst);
        write_mem!(&mut self.entering[idx], true);
        fence(Ordering::SeqCst);

        let max = (0..NUM_THREADS)
            .filter_map(|i| read_mem!(&self.tickets[i]))
            .max()
            .unwrap_or(0);

        let ticket = max + 1;
        write_mem!(&mut self.tickets[idx], Some(ticket));

        fence(Ordering::SeqCst);
        write_mem!(&mut self.entering[idx], false);
        fence(Ordering::SeqCst);

        // 대기처리
        (0..NUM_THREADS)
            .filter(|i| *i != idx)
            .inspect(|i| while read_mem!(&self.entering[*i]) {})
            .for_each(|i| loop {
                match read_mem!(&self.tickets[i]) {
                    Some(t) => {
                        if ticket < t || (ticket == t && idx < i) {
                            break;
                        }
                    }
                    None => break,
                }
            });

        fence(Ordering::SeqCst);
        LockGuard { idx }
    }
}

static mut LOCK: BakeryLock = BakeryLock {
    entering: [false; NUM_THREADS],
    tickets: [None; NUM_THREADS],
};
static mut COUNT: u64 = 0;

struct LockGuard {
    idx: usize,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        fence(Ordering::SeqCst);
        write_mem!(&mut LOCK.tickets[self.idx], None);
    }
}

fn main() {
    (0..NUM_THREADS)
        .map(|i| {
            thread::spawn(move || {
                (0..NUM_LOOP).for_each(|_| {
                    let _lock = unsafe { LOCK.lock(i) };
                    unsafe {
                        let c = read_volatile(&COUNT);
                        write_volatile(&mut COUNT, c + 1);
                    };
                });
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|th| th.join().unwrap());

    println!(
        "COUNT = {} (expected) = {}",
        unsafe { COUNT },
        NUM_LOOP * NUM_THREADS
    );
}
