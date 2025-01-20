use std::{
    collections::{HashMap, VecDeque},
    ffi::{c_int, c_void},
    io,
    os::fd::RawFd,
    process,
    sync::{Arc, Mutex},
    task::Waker,
};

use libc::{epoll_ctl, epoll_event};

fn write_eventfd(fd: RawFd, n: usize) {
    let ptr = &n as *const usize as *const u8;
    unsafe {
        let val = std::slice::from_raw_parts(ptr, std::mem::size_of_val(&n));
        if libc::write(fd, val.as_ptr() as *const c_void, val.len()) == -1 {
            libc::perror(c"libc::write".as_ptr());
        };
    };
}

type EpollFlags = c_int;

enum IOOps {
    // epoll에 추가
    Add(EpollFlags, RawFd, Waker),
    // epoll에서 삭제
    Remove(RawFd),
}

pub struct IOSelector {
    // fd에서 waker
    wakers: Mutex<HashMap<RawFd, Waker>>,
    // IO 큐
    queue: Mutex<VecDeque<IOOps>>,
    // epoll의 fd
    epfd: RawFd,
    // eventfd의 fd
    event: RawFd,
}

impl IOSelector {
    pub fn new() -> Arc<Self> {
        let epfd = unsafe {
            let epfd = libc::epoll_create1(0);
            if epfd == -1 {
                libc::perror(c"libc::epoll_create1".as_ptr());
                process::exit(-1);
            }
            epfd
        };
        let event = unsafe {
            let event = libc::eventfd(0, 0);
            if event == -1 {
                libc::perror(c"libc::eventfd".as_ptr());
                process::exit(-1);
            }
            event
        };

        let s = IOSelector {
            wakers: Mutex::new(HashMap::new()),
            queue: Mutex::new(VecDeque::new()),
            epfd,
            event,
        };
        let result = Arc::new(s);

        let result_clone = result.clone();
        std::thread::spawn(move || result_clone.select());

        result
    }

    fn add_event(
        &self,
        flag: EpollFlags,
        fd: RawFd,
        waker: Waker,
        wakers: &mut HashMap<RawFd, Waker>,
    ) {
        let epoll_add = libc::EPOLL_CTL_ADD;
        let epoll_mod = libc::EPOLL_CTL_MOD;
        let epoll_one = libc::EPOLLONESHOT;

        let mut ev = epoll_event {
            events: flag as u32 | epoll_one as u32,
            u64: fd as u64,
        };

        unsafe {
            if epoll_ctl(self.epfd, epoll_add, fd, &mut ev) == -1 {
                match io::Error::last_os_error().kind() {
                    io::ErrorKind::AlreadyExists => {
                        // 이미 추가되어 있는 경우에는 재설정
                        if epoll_ctl(self.epfd, epoll_mod, fd, &mut ev) == -1 {
                            libc::perror(c"libc::epoll_ctl".as_ptr());
                            process::exit(-1);
                        }
                    }
                    err => panic!("epoll_ctl: {}", err),
                }
            }
        }

        assert!(!wakers.contains_key(&fd));
        wakers.insert(fd, waker);
    }

    fn rm_event(&self, fd: RawFd, wakers: &mut HashMap<RawFd, Waker>) {
        let epoll_del = libc::EPOLL_CTL_DEL;

        let mut ev = epoll_event {
            events: 0,
            u64: fd as u64,
        };

        unsafe {
            epoll_ctl(self.epfd, epoll_del, fd, &mut ev);
        }
        wakers.remove(&fd);
    }

    fn select(&self) {
        let epoll_in = libc::EPOLLIN;
        let epoll_add = libc::EPOLL_CTL_ADD;

        let mut ev = libc::epoll_event {
            events: epoll_in as u32,
            u64: self.event as u64,
        };
        unsafe {
            if libc::epoll_ctl(self.epfd, epoll_add, self.event, &mut ev) == -1 {
                libc::perror(c"libc::epoll_ctl".as_ptr());
                process::exit(-1);
            }

            let mut events = vec![epoll_event { events: 0, u64: 0 }; 1024];
            loop {
                let nfds = libc::epoll_wait(self.epfd, events.as_mut_ptr(), 1024, -1);
                if nfds == -1 {
                    break;
                }
                let mut t = self.wakers.lock().unwrap();
                events[..nfds as usize].iter().for_each(|event| {
                    if event.u64 == self.event as u64 {
                        let mut q = self.queue.lock().unwrap();
                        while let Some(op) = q.pop_front() {
                            match op {
                                IOOps::Add(flag, fd, waker) => {
                                    self.add_event(flag, fd, waker, &mut t);
                                }
                                IOOps::Remove(fd) => {
                                    self.rm_event(fd, &mut t);
                                }
                            }
                        }
                    } else {
                        let data = event.u64 as i32;
                        let waker = t.remove(&data).unwrap();
                        waker.wake_by_ref();
                    }
                });
            }
        }
    }

    // 파일 디스크립터 등록용 함수
    pub fn register(&self, flags: EpollFlags, fd: RawFd, waker: Waker) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(IOOps::Add(flags, fd, waker));
        write_eventfd(self.event, 1);
    }

    pub fn unregister(&self, fd: RawFd) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(IOOps::Remove(fd));
        write_eventfd(self.event, 1);
    }
}
