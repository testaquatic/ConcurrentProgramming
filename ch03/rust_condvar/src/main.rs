use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};

fn child(id: u64, p: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*p;
    let started = lock.lock().unwrap();
    let _guard = cvar.wait_while(started, |started| !*started).unwrap();

    println!("Child {}", id);
}

fn parent(p: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*p;

    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_all();
}

fn main() {
    let pair0 = Arc::new((Mutex::new(false), Condvar::new()));
    let pair1 = pair0.clone();
    let pair2 = pair0.clone();

    let c0 = thread::spawn(move || child(0, pair0));
    let c1 = thread::spawn(move || child(1, pair1));
    let p = thread::spawn(move || parent(pair2));

    c0.join().unwrap();
    c1.join().unwrap();
    p.join().unwrap();
}
