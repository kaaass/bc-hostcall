use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use tokio;
use low_level::host::LowLevelCtx;
use crate::future::HandleRxFuture;

/// 模块的异步上下文，主要维护围绕两个队列驱动的异步任务
pub struct AsyncCtx {
    /// 收到待处理的队列
    pub tx_queue: Mutex<Cell<VecDeque<Vec<u8>>>>,
    // TODO tx_waker

    /// 要发送给 WASM 的队列
    pub rx_queue: Mutex<Cell<VecDeque<Vec<u8>>>>,
    pub rx_waker: Mutex<Cell<Option<Waker>>>,

    /// 是否仍然存活
    alive: Mutex<Cell<bool>>,
}

impl AsyncCtx {

    pub fn new() -> Self {
        AsyncCtx {
            tx_queue: Mutex::new(Cell::new(VecDeque::new())),
            rx_queue: Mutex::new(Cell::new(VecDeque::new())),
            rx_waker: Mutex::new(Cell::new(None)),
            alive: Mutex::new(Cell::new(true)),
        }
    }

    pub fn push_tx(&self, msg: Vec<u8>) {
        // TODO
        // 压入 tx_queue
        // wake tx
    }

    pub fn push_rx(&self, msg: Vec<u8>) {
        // 压入 rx_queue
        {
            let mut rx_queue = self.rx_queue.lock().unwrap();
            rx_queue.get_mut().push_back(msg);
        }
        // 唤醒 rx_wake
        {
            let mut waker = self.rx_waker.lock().unwrap();
            waker.get_mut().as_ref().unwrap().wake_by_ref();
        }
    }

    pub fn spawn<T>(self: Arc<Self>, ll_ctx: Arc<LowLevelCtx<T>>)
        where T: Send + Sync + 'static,
    {
        // TODO 启动 tx_waker
        // 启动 rx_waker
        let future = HandleRxFuture::new(self.clone(), ll_ctx.clone());
        tokio::spawn(future);
    }

    pub fn alive(&self) -> bool {
        self.alive.lock().unwrap().get()
    }

    pub fn kill(&self) {
        self.alive.lock().unwrap().set(false);
        // TODO 唤醒 tx_wake
        // 唤醒 rx_wake
        {
            let mut waker = self.rx_waker.lock().unwrap();
            waker.get_mut().as_ref().unwrap().wake_by_ref();
        }
    }
}
