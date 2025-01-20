use std::{
    future::Future,
    io::{BufRead, BufReader},
    net::TcpStream,
    os::fd::{AsRawFd, RawFd},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::io_selector::IOSelector;

pub struct AsyncReader {
    fd: RawFd,
    reader: BufReader<TcpStream>,
    selector: Arc<IOSelector>,
}

impl AsyncReader {
    pub fn new(stream: TcpStream, selector: Arc<IOSelector>) -> AsyncReader {
        stream.set_nonblocking(true).unwrap();
        AsyncReader {
            fd: stream.as_raw_fd(),
            reader: BufReader::new(stream),
            selector,
        }
    }

    pub fn read_line(&mut self) -> ReadLine {
        ReadLine { reader: self }
    }
}

impl Drop for AsyncReader {
    fn drop(&mut self) {
        self.selector.unregister(self.fd);
    }
}

pub struct ReadLine<'a> {
    reader: &'a mut AsyncReader,
}

impl Future for ReadLine<'_> {
    type Output = Option<String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut line = String::new();
        match self.reader.reader.read_line(&mut line) {
            Ok(0) => Poll::Ready(None),
            Ok(_) => Poll::Ready(Some(line)),
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                self.reader
                    .selector
                    .register(libc::EPOLLIN, self.reader.fd, cx.waker().clone());
                Poll::Pending
            }
            Err(_) => Poll::Ready(None),
        }
    }
}
