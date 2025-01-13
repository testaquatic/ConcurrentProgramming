use std::thread;

use libsemaphore::channel::channel;

const NUM_LOOP: usize = 100_000;
const NUM_THREAD: usize = 8;

fn main() {
    let (tx, rx) = channel(4);
    let mut v = Vec::new();

    let t = thread::spawn(move || {
        let mut cnt = 0;
        while cnt < NUM_LOOP * NUM_THREAD {
            let n = rx.recv();
            println!("recv: n = {:?}", n);
            cnt += 1;
        }
    });

    v.push(t);

    let recv_h = (0..NUM_THREAD).map(|i| {
        let tx0 = tx.clone();
        thread::spawn(move || {
            (0..NUM_LOOP).for_each(|j| tx0.send((i, j)));
        })
    });

    v.extend(recv_h);

    v.into_iter().for_each(|t| t.join().unwrap());
}
