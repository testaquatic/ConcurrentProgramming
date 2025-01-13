use std::{
    sync::{Arc, Barrier},
    thread,
};

fn main() {
    let barrier = Arc::new(Barrier::new(10));

    let v = (0..10)
        .map(|_| {
            let b = barrier.clone();
            thread::spawn(move || {
                b.wait();
                println!("finished barrier");
            })
        })
        .collect::<Vec<_>>();

    v.into_iter().try_for_each(|th| th.join()).unwrap();
}
