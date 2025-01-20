use std::{
    future::Future,
    io::BufWriter,
    net::{SocketAddr, TcpListener, TcpStream},
    os::fd::AsRawFd,
    sync::Arc,
    task::Poll,
};

use crate::{async_reader::AsyncReader, io_selector::IOSelector};

pub struct AsyncListener {
    listener: TcpListener,
    selector: Arc<IOSelector>,
}

impl AsyncListener {
    pub fn listen(addr: &str, selector: Arc<IOSelector>) -> AsyncListener {
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();

        AsyncListener { listener, selector }
    }

    pub fn accept(&self) -> Accept {
        Accept { listener: self }
    }
}

impl Drop for AsyncListener {
    fn drop(&mut self) {
        self.selector.unregister(self.listener.as_raw_fd());
    }
}

pub struct Accept<'a> {
    listener: &'a AsyncListener,
}

impl Future for Accept<'_> {
    type Output = (AsyncReader, BufWriter<TcpStream>, SocketAddr);

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.listener.listener.accept() {
            Ok((stream, addr)) => {
                let stream0 = stream.try_clone().unwrap();
                Poll::Ready((
                    AsyncReader::new(stream0, self.listener.selector.clone()),
                    BufWriter::new(stream),
                    addr,
                ))
            }
            // 받아들일 커넥션이 없는 경우에는 epoll에 등록
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                self.listener.selector.register(
                    libc::EPOLLIN,
                    self.listener.listener.as_raw_fd(),
                    cx.waker().clone(),
                );
                Poll::Pending
            }
            Err(e) => panic!("accept error: {}", e),
        }
    }
}
