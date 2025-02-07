use std::{sync::Arc, thread};

const NUM_LOOP: usize = 100_000;
const NUM_THREAD: usize = 4;

fn main() {
    let n = Arc::new(mcs_lock::MCSLock::new(0));
    (0..NUM_THREAD)
        .map(|_| {
            let n0 = n.clone();
            thread::spawn(move || {
                let mut node = mcs_lock::MCSNode::new();
                (0..NUM_LOOP).for_each(|_| {
                    let mut r = n0.lock(&mut node);
                    *r += 1;
                });
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|t| t.join().unwrap());

    let mut node = mcs_lock::MCSNode::new();
    let r = n.lock(&mut node);
    println!("COUNT = {} (expected = {})", *r, NUM_LOOP * NUM_THREAD);
}
