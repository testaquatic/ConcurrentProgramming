use std::sync::Arc;

const NUM_LOOP: usize = 100_000;
const NUM_THREAD: usize = 4;

fn main() {
    let lock = Arc::new(fair_lock::FairLock::new(0));
    (0..NUM_THREAD)
        .map(|i| {
            let lock0 = lock.clone();
            std::thread::spawn(move || {
                (0..NUM_LOOP).for_each(|_| {
                    let mut data = lock0.lock(i);
                    *data += 1;
                });
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|t| t.join().unwrap());

    println!(
        "COUNT = {} (expected = {})",
        *lock.lock(0),
        NUM_LOOP * NUM_THREAD
    );
}
