use std::sync::{Arc, Mutex};

const NUM_TASKS: usize = 4;
const NUM_LOOP: usize = 100_000;

#[tokio::main]
async fn main() {
    let val = Arc::new(Mutex::new(0));
    let v = (0..NUM_TASKS)
        .map(|_| {
            let n = val.clone();
            tokio::spawn(async move {
                (0..NUM_LOOP).for_each(|_| {
                    let mut n0 = n.lock().unwrap();
                    *n0 += 1;
                });
            })
        })
        .collect::<Vec<_>>();
    for i in v {
        i.await.unwrap();
    }

    println!(
        "COUNT = {} (expected = {})",
        *val.lock().unwrap(),
        NUM_TASKS * NUM_LOOP
    );
}
