use std::{
    hint::spin_loop,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

const NUM_THREADS: usize = 8;

struct ReentLock {
    // 락용 공용 변수
    lock: AtomicBool,
    // 현재 락을 획득 중인 스레드 ID, 0이 아니면 락 획득 중임
    id: AtomicUsize,
    // 재귀락 카운트
    cnt: AtomicUsize,
}

impl ReentLock {
    fn acquire(&self, id: usize) {
        if self.lock.load(Ordering::Acquire) && self.id.load(Ordering::Acquire) == id {
            self.cnt.fetch_add(1, Ordering::AcqRel);
        } else {
            while self.lock.load(Ordering::Acquire) {
                spin_loop();
            }
            self.id.store(id, Ordering::Release);
            self.cnt.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn release(&self) {
        self.cnt.fetch_sub(1, Ordering::AcqRel);
        if self.cnt.load(Ordering::Acquire) == 0 {
            self.id.store(0, Ordering::Release);
            self.lock.store(false, Ordering::Release);
        }
    }

    fn reent_lock_test(&self, id: usize, n: usize) {
        if n == 0 {
            return;
        }

        self.acquire(id);
        self.reent_lock_test(id, n - 1);
        self.release();
    }
}

unsafe impl Sync for ReentLock {}

fn main() {
    let lock = Arc::new(ReentLock {
        lock: AtomicBool::new(false),
        id: AtomicUsize::new(0),
        cnt: AtomicUsize::new(0),
    });

    (0..NUM_THREADS)
        .map(|id| {
            let id = id + 1;
            let lock = lock.clone();
            thread::spawn(move || {
                assert_ne!(id, 0);
                (0..10_000).for_each(|_| lock.clone().reent_lock_test(id, 10));
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|th| th.join().unwrap());
}
