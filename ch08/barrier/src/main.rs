use std::sync::mpsc::{self, Sender};

fn main() {
    let mut v = Vec::new();
    let (tx, rx) = mpsc::channel::<Sender<()>>();
    let barrier = move || {
        let x = rx.recv().unwrap();
        let y = rx.recv().unwrap();
        let z = rx.recv().unwrap();
        println!("send!");
        x.send(()).unwrap();
        y.send(()).unwrap();
        z.send(()).unwrap();
    };
    let t = std::thread::spawn(barrier);
    v.push(t);

    (0..3).for_each(|_| {
        let tx_c = tx.clone();
        let node = move || {
            let (tx0, rx0) = mpsc::channel();
            tx_c.send(tx0).unwrap();
            rx0.recv().unwrap();
            println!("received!");
        };
        let t = std::thread::spawn(node);
        v.push(t);
    });

    v.into_iter().for_each(|t| t.join().unwrap());
}
