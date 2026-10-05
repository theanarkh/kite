use std::future::Future;
use std::rc::Rc;
use tokio::task::JoinHandle;

pub type Task = Box<dyn for<'s, 'i> FnOnce(&mut v8::PinScope<'s, 'i>)>;

pub struct EventLoop {
    refs: i64,
    pub sender: Rc<tokio::sync::mpsc::UnboundedSender<Task>>,
    pub receiver: tokio::sync::mpsc::UnboundedReceiver<Task>,
}

impl EventLoop {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Task>();
        Self {
            refs: 0,
            sender: Rc::new(sender),
            receiver,
        }
    }

    pub fn add_ref(&mut self, value: i64) {
        self.refs += value;
        if value < 0 {
            let sender = self.sender.clone();
            sender
            .send(Box::new(move |_| {}))
            .unwrap();
        }
    }

    pub fn refs(&self) -> i64 {
        self.refs
    }

    pub fn submit<F, R>(
        &mut self,
        work: F,
        done: impl FnOnce(&mut v8::PinScope<'_, '_>, R) + 'static,
    ) -> JoinHandle<()>
    where
        F: Future<Output = R> + 'static,
        R: 'static,
    {
        self.add_ref(1);
        let sender = self.sender.clone();
        tokio::task::spawn_local(async move {
            let result = work.await;
            sender
                .send(Box::new(move |scope| done(scope, result)))
                .unwrap();
        })
    }

    pub async fn run(scope: &mut v8::PinScope<'_, '_>) {
        if scope.get_slot::<EventLoop>().unwrap().refs() == 0 {
            let inner_scope = std::pin::pin!(v8::HandleScope::new(scope));
            let mut inner_scope = inner_scope.init();
            let context = inner_scope.get_current_context();
            context
                .get_microtask_queue()
                .perform_checkpoint(&mut **inner_scope);
            return;
        }
        loop {
            let task = match scope.get_slot_mut::<EventLoop>().unwrap()
                .receiver
                .recv()
                .await
            {
                Some(task) => task,
                None => break,
            };
            {
                // 每个任务使用独立的嵌套 HandleScope，任务中创建的 Local 句柄
                // 在本轮结束（storage drop）时统一释放，避免堆积到顶层 scope。
                let inner_scope = std::pin::pin!(v8::HandleScope::new(scope));
                let mut inner_scope = inner_scope.init();
                task(&mut inner_scope);
                let context = inner_scope.get_current_context();
                context
                    .get_microtask_queue()
                    .perform_checkpoint(&mut **inner_scope);
            }
            if scope.get_slot::<EventLoop>().unwrap().refs() == 0 {
                break;
            }
        }
    }
}
