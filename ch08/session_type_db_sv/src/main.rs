extern crate session_types;

use session_types::{self as S, offer};
use std::{collections::HashMap, thread};

type Put = S::Recv<u64, S::Recv<u64, S::Var<S::Z>>>;
type Get = S::Recv<u64, S::Send<Option<u64>, S::Var<S::Z>>>;

type DBServer = S::Rec<S::Offer<Put, S::Offer<Get, S::Eps>>>;

type DBClient = <DBServer as S::HasDual>::Dual;

fn db_server(c: S::Chan<(), DBServer>) {
    let mut c_enter = c.enter();
    let mut db = HashMap::new();

    loop {
        match c_enter.offer() {
            // Put
            S::Branch::Left(c) => {
                let (c, key) = c.recv();
                let (c, val) = c.recv();
                db.insert(key, val);
                c_enter = c.zero();
            }
            // Get 또는 종료
            S::Branch::Right(c) => match c.offer() {
                // Get
                S::Branch::Left(c) => {
                    let (c, key) = c.recv();
                    let c = if let Some(val) = db.get(&key) {
                        c.send(Some(*val))
                    } else {
                        c.send(None)
                    };
                    c_enter = c.zero();
                }
                // 종료
                S::Branch::Right(c) => {
                    c.close();
                    return;
                }
            },
        }
    }
}

fn db_client(c: S::Chan<(), DBClient>) {
    let c = c.enter();

    // Put
    let c = c.sel1().send(10).send(4).zero();
    let c = c.sel1().send(50).send(7).zero();

    // Get
    let (c, val) = c.sel2().sel1().send(10).recv();
    println!("val = {:?}", val);

    let c = c.zero();

    // Get
    let (c, val) = c.sel2().sel1().send(20).recv();
    println!("val = {:?}", val);

    c.zero().sel2().sel2().close();
}

fn db_server_macro(c: S::Chan<(), DBServer>) {
    let mut c_center = c.enter();
    let mut db = HashMap::new();

    loop {
        let c = c_center;
        offer! {
            c,
            Put => {
                let (c, key) = c.recv();
                let (c, val) = c.recv();
                db.insert(key, val);
                c_center = c.zero();
            },
            Get => {
                let (c, key) = c.recv();
                let c = if let Some(val) = db.get(&key) {
                    c.send(Some(*val))
                } else {
                    c.send(None)
                };
                c_center = c.zero();
            },
            Quit => {
                c.close();
                return;
            }
        }
    }
}

type SChan = S::Chan<(), S::Send<(), S::Eps>>;
type ChanRecv = S::Recv<SChan, S::Eps>;
type ChanSend = <ChanRecv as S::HasDual>::Dual;

fn chan_recv(c: S::Chan<(), ChanRecv>) {
    let (c, cr) = c.recv();
    c.close();
    let cr = cr.send(());
    cr.close();
}

fn chan_send(c: S::Chan<(), ChanSend>) {
    let (c1, c2) = S::session_channel();
    let c = c.send(c1);
    c.close();
    let (c2, _) = c2.recv();
    c2.close();
}

fn main() {
    let (server_chan, client_chan) = S::session_channel();
    let srv_t = thread::spawn(move || db_server_macro(server_chan));
    let cli_t = thread::spawn(move || db_client(client_chan));
    srv_t.join().unwrap();
    cli_t.join().unwrap();
}
