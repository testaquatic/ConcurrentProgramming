use std::thread;

use lib_bankers::Bankers;

const NUM_LOOP: usize = 100_000;

fn main() {
    let banker = Bankers::new([1, 1], [[1, 1], [1, 1]]);
    let banker0 = banker.clone();

    let philosopher0 = thread::spawn(move || {
        (0..NUM_LOOP).for_each(|_| {
            while !banker0.take(0, 0) {}
            while !banker0.take(0, 1) {}
            println!("0: eating");

            banker0.release(0, 0);
            banker0.release(0, 1);
        });
    });

    let philosopher1 = thread::spawn(move || {
        (0..NUM_LOOP).for_each(|_| {
            while !banker.take(1, 0) {}
            while !banker.take(1, 1) {}
            println!("1: eating");

            banker.release(1, 0);
            banker.release(1, 1);
        });
    });

    philosopher0.join().unwrap();
    philosopher1.join().unwrap();
}
