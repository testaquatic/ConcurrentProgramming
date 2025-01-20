use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Wake, Waker},
};

struct Task {
    // 실행하는 코루틴
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
    // Executor에 스케줄링하기 위한 채널
    sender: SyncSender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let self0 = self.clone();
        self.sender.send(self0).unwrap();
    }
}

struct Executor {
    sender: SyncSender<Arc<Task>>,
    receiver: Receiver<Arc<Task>>,
}

impl Executor {
    fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel(1024);
        Self { sender, receiver }
    }

    fn get_spawner(&self) -> Spawner {
        Spawner {
            sender: self.sender.clone(),
        }
    }

    fn run(&self) {
        while let Ok(task) = self.receiver.recv() {
            let mut future = task.future.lock().unwrap();
            let waker = Waker::from(task.clone());
            let mut ctx = Context::from_waker(&waker);
            let _ = future.as_mut().poll(&mut ctx);
        }
    }
}

struct Spawner {
    sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + Send + Sync + 'static) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(future),
            sender: self.sender.clone(),
        });

        self.sender.send(task).unwrap();
    }
}

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
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.state {
            StateHello::Hello => {
                println!("Hello, ");
                self.state = StateHello::World;
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            StateHello::World => {
                println!("World");
                self.get_mut().state = StateHello::End;
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            StateHello::End => std::task::Poll::Ready(()),
        }
    }
}

fn main() {
    let executor = Executor::new();
    executor.get_spawner().spawn(async {
        let h = Hello::new();
        h.await;
    });
    executor.run();
}
