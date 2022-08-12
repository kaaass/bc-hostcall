extern crate core;

use std::future::Future;

mod queue;
mod task;
pub mod rt;

// FIXME: 此处的错误类型仅仅是最简单，可用于容纳任何错误的类型。而实际上好的错误类型
//        应该囊括更加细节的错误信息。此处仅为适应短时间的开发需求而临时设计。
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

/// 将异步任务送入本地队列执行
#[inline]
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    task::Task::spawn(Box::pin(future));
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use crate::queue::QUEUE;
    use crate::spawn_local;

    #[test]
    fn test_future() {

        let cnt = Rc::new(Cell::new(0));

        let ccnt = cnt.clone();
        spawn_local(async move {
            ccnt.set(ccnt.get() + 1);
        });

        assert_eq!(cnt.get(), 0);

        QUEUE.with(|queue| {
            queue.run_all();
        });

        assert_eq!(cnt.get(), 1);
    }
}
