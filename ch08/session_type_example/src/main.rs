extern crate session_types;
use std::thread;

use session_types as S;

type Client = S::Send<u64, S::Choose<S::Recv<u64, S::Eps>, S::Recv<bool, S::Eps>>>;
// 클라이언트의 엔드포인트 타입
type Server = <Client as S::HasDual>::Dual;

enum Op {
    Square,
    Even,
}

fn server(c: S::Chan<(), Server>) {
    let (c, n) = c.recv();
    match c.offer() {
        S::Branch::Left(c) => c.send(n * n).close(),
        S::Branch::Right(c) => c.send(n % 2 == 0).close(),
    }
}

fn client(c: S::Chan<(), Client>, n: u64, op: Op) {
    let c = c.send(n);
    match op {
        Op::Square => {
            let c = c.sel1();
            let (c, val) = c.recv();
            c.close();
            println!("{}^2 = {}", n, val);
        }
        Op::Even => {
            let c = c.sel2();
            let (c, val) = c.recv();
            c.close();
            if val {
                println!("{} is even", n);
            } else {
                println!("{} is odd", n);
            }
        }
    }
}

fn main() {
    let (server_chan, client_chan) = S::session_channel();
    let srv_t = thread::spawn(move || server(server_chan));
    let cli_t = thread::spawn(move || client(client_chan, 11, Op::Even));
    srv_t.join().unwrap();
    cli_t.join().unwrap();

    let (server_chan, client_chan) = S::session_channel();
    let srv_t = thread::spawn(move || server(server_chan));
    let cli_t = thread::spawn(move || client(client_chan, 11, Op::Square));
    srv_t.join().unwrap();
    cli_t.join().unwrap();
}
