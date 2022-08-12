#![cfg_attr(target_feature = "atomics", feature(stdsimd))]
#![deny(missing_docs)]

use std::future::Future;

mod queue;
mod task;

/// Runs a Rust `Future` on the current thread.
///
/// The `future` must be `'static` because it will be scheduled
/// to run in the background and cannot contain any stack references.
///
/// The `future` will always be run on the next microtask tick even if it
/// immediately returns `Poll::Ready`.
///
/// # Panics
///
/// This function has the same panic behavior as `future_to_promise`.
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
        spawn_local(async {
            cnt.set(cnt.get() + 1);
        });

        assert_eq!(cnt.get(), 0);

        QUEUE.with(|queue| {
            queue.run_all();
        });

        assert_eq!(cnt.get(), 1);
    }
}
