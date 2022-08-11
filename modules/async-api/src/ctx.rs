use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::Mutex;

/// 模块的异步上下文，主要维护围绕两个队列驱动的异步任务
struct AsyncCtx {
    /// 收到待处理的队列
    tx_queue: Mutex<VecDeque<Vec<u8>>>,
    // TODO tx_waker

    /// 要发送给 WASM 的队列
    rx_queue: Mutex<VecDeque<Vec<u8>>>,
    // TODO rx_waker

    /// 是否仍然存活
    alive: Mutex<Cell<bool>>,
}

impl AsyncCtx {

    pub fn push_tx(msg: Vec<u8>) {
        // TODO
        // 压入 tx_queue
        // wake tx
    }

    pub fn push_rx(msg: Vec<u8>) {
        // TODO
        // 压入 rx_queue
        // wake rx
    }

    pub fn start() {
        // TODO
        // 启动 tx_waker
        // 启动 rx_waker
    }

    pub fn alive(&self) -> bool {
        self.alive.lock().unwrap().get()
    }

    pub fn kill(&self) {
        self.alive.lock().unwrap().set(false);
        // TODO wake tx rx
    }
}
