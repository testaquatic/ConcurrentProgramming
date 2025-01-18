use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpListener,
    os::fd::{AsRawFd, RawFd},
    process,
};


fn main() {
    let epoll_in = libc::EPOLLIN;
    let epoll_add = libc::EPOLL_CTL_ADD;
    let epoll_del = libc::EPOLL_CTL_DEL;

    let listener = TcpListener::bind("127.0.0.1:10000").unwrap();

    unsafe {
        let epfd = libc::epoll_create1(0);
        if epfd < 0 {
            libc::perror(c"epoll_create1 error:".as_ptr());
            process::exit(-1);
        }

        let listen_fd = listener.as_raw_fd();
        let mut ev = libc::epoll_event {
            events: epoll_in as u32,
            u64: listen_fd as u64,
        };
        if libc::epoll_ctl(epfd, epoll_add, listen_fd, &mut ev) != 0 {
            libc::perror(c"epoll_ctl error:".as_ptr());
            process::exit(-1);
        }

        let mut fd2buf = HashMap::new();
        let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; 1024];

        loop {
            let nfds = libc::epoll_wait(epfd, events.as_mut_ptr(), events.len() as i32, -1);
            if nfds < 0 {
                libc::perror(c"epoll_wait error:".as_ptr());
                break;
            }
            events.iter().take(nfds as usize).for_each(|event| {
                if event.u64 == listen_fd as u64 {
                    // 리슨 소켓에 이벤트
                    if let Ok((stream, _)) = listener.accept() {
                        let fd = stream.as_raw_fd();
                        let stream0 = stream.try_clone().unwrap();
                        let reader = BufReader::new(stream0);
                        let writer = BufWriter::new(stream);

                        fd2buf.insert(fd, (reader, writer));

                        println!("accept: fd = {}", fd);

                        let mut ev = libc::epoll_event {
                            events: epoll_in as u32,
                            u64: fd as u64,
                        };
                        if libc::epoll_ctl(epfd, epoll_add, fd, &mut ev) != 0 {
                            libc::perror(c"epoll_ctl error:".as_ptr());
                            process::exit(-1);
                        }
                    }
                } else {
                    // 클라이언트에서 데이터 도착
                    let fd = event.u64 as RawFd;
                    let (reader, writer) = fd2buf.get_mut(&fd).unwrap();

                    let mut buf = String::new();
                    let n = reader.read_line(&mut buf).unwrap();
                    // 커넥션을 클로즈한 경우 epoll 감시 대상에서 제외하낟.
                    if n == 0 {
                        let mut ev = libc::epoll_event {
                            events: epoll_in as u32,
                            u64: fd as u64,
                        };
                        if libc::epoll_ctl(epfd, epoll_del, fd, &mut ev) != 0 {
                            libc::perror(c"epoll_ctl error:".as_ptr());
                            process::exit(-1);
                        }
                        fd2buf.remove(&fd);
                        println!("close: fd = {}", fd);
                        return;
                    }
                    print!("read: fd = {}, buf = {}", fd, buf);

                    writer.write_all(buf.as_bytes()).unwrap();
                    writer.flush().unwrap();
                }
            });
        }
    }
}
