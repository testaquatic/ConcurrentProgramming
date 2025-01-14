use std::{
    sync::{Arc, RwLock},
    thread,
};

fn main() {
    let val = Arc::new(RwLock::new(true));

    let t = thread::spawn(move || {
        {
            let _unused = val.read().unwrap();
        }
        *val.write().unwrap() = false;
        println!("not deadlock");
    });

    t.join().unwrap();
}
