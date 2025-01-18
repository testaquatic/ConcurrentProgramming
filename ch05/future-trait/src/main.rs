use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Wake, Waker},
};

struct Hello {
    state: StateHello,
}

enum StateHello {
    Hello,
    World,
    End,
}

impl Hello {
    fn new() -> Self {
        Self {
            state: StateHello::Hello,
        }
    }
}

impl Future for Hello {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.state {
            StateHello::Hello => {
                println!("Hello, ");
                self.get_mut().state = StateHello::World;
                std::task::Poll::Pending
            }
            StateHello::World => {
                println!("World");
                self.get_mut().state = StateHello::End;
                std::task::Poll::Pending
            }
            StateHello::End => std::task::Poll::Ready(()),
        }
    }
}

struct Task {
    hello: Mutex<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
}

impl Task {
    fn new() -> Self {
        Self {
            hello: Mutex::new(Box::pin(Hello::new())),
        }
    }
}

impl Wake for Task {
    fn wake(self: std::sync::Arc<Self>) {}
    fn wake_by_ref(self: &std::sync::Arc<Self>) {}
}

fn main() {
    let task = Arc::new(Task::new());
    let waker = Waker::from(task.clone());
    let mut ctx = Context::from_waker(&waker);
    let mut hello = task.hello.lock().unwrap();
    let _ = hello.as_mut().poll(&mut ctx);
    let _ = hello.as_mut().poll(&mut ctx);
    let _ = hello.as_mut().poll(&mut ctx);
}
