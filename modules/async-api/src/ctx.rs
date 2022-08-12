use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::task::Waker;
use tokio;
use tokio::sync::Notify;
use low_level::host::LowLevelCtx;
use rpc::{abi, RpcNode, RpcEndCtx, RpcSeqNo, RpcResponseCtx};
use serialize::SerializeCtx;
use crate::future::{AsyncRequestFuture, HandleRxFuture, HandleTxFuture};
use crate::Result;

/// 接受消息后的动作
pub enum ResultAction {
    /// 唤醒某个 Future
    Wake(Waker),
    /// 返回结果
    Response(Vec<u8>),
    /// 转发结果
    ForwardResult(abi::LinkHint, abi::FunctionIdent),
}

pub type CtxResolveCallback =
    dyn Fn(abi::LinkHint) -> Result<Arc<AsyncCtx>> + Send + Sync;

/// 模块的异步上下文，主要维护围绕两个队列驱动的异步任务
pub struct AsyncCtx {

    // 初始化提醒
    pub init_notify: Notify,

    /// 收到待处理的队列
    pub tx_queue: Mutex<Cell<VecDeque<Vec<u8>>>>,
    pub tx_waker: Mutex<Cell<Option<Waker>>>,

    /// 要发送给 WASM 的队列
    pub rx_queue: Mutex<Cell<VecDeque<Vec<u8>>>>,
    pub rx_waker: Mutex<Cell<Option<Waker>>>,

    /// 是否仍然存活
    alive: Mutex<Cell<bool>>,

    pub rpc_ctx: Mutex<Cell<Option<RpcNode<Arc<Self>>>>>,

    pub tx_action: Mutex<Cell<HashMap<RpcSeqNo, ResultAction>>>,

    /// 解析其他模块异步上下文的回调
    resolve_cb: Mutex<Cell<Option<Box<CtxResolveCallback>>>>,
}

impl AsyncCtx {

    pub fn new() -> Self {
        AsyncCtx {
            init_notify: Notify::new(),
            tx_queue: Mutex::new(Cell::new(VecDeque::new())),
            tx_waker: Mutex::new(Cell::new(None)),
            rx_queue: Mutex::new(Cell::new(VecDeque::new())),
            rx_waker: Mutex::new(Cell::new(None)),
            alive: Mutex::new(Cell::new(true)),
            rpc_ctx: Mutex::new(Cell::new(None)),
            tx_action: Mutex::new(Cell::new(HashMap::new())),
            resolve_cb: Mutex::new(Cell::new(None)),
        }
    }

    pub fn set_resolve_cb<CB>(&self, cb: CB)
        where CB: Fn(abi::LinkHint) -> Result<Arc<AsyncCtx>> + Send + Sync + 'static,
    {
        let mut resolve_cb = self.resolve_cb.lock().unwrap();
        *resolve_cb.get_mut() = Some(Box::new(cb));
    }

    pub fn push_tx(&self, msg: Vec<u8>) {
        // 压入 tx_queue
        {
            let mut tx_queue = self.tx_queue.lock().unwrap();
            tx_queue.get_mut().push_back(msg);
        }
        // 唤醒 tx_wake
        {
            let mut waker = self.tx_waker.lock().unwrap();
            waker.get_mut().as_ref().unwrap().wake_by_ref();
        }
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

    pub fn push_action(&self, seq_no: RpcSeqNo, action: ResultAction) {
        let mut tx_action = self.tx_action.lock().unwrap();
        tx_action.get_mut().insert(seq_no, action);
    }

    pub fn take_action(&self, seq_no: RpcSeqNo) -> Option<ResultAction> {
        let mut tx_action = self.tx_action.lock().unwrap();
        tx_action.get_mut().remove(&seq_no)
    }

    fn forward_action_cb(ctx: &RpcEndCtx<Arc<Self>>, func: abi::FunctionIdent, raw_msg: &[u8]) -> rpc::Result<()> {
        let mut resolve_cb = ctx.data().resolve_cb.lock().unwrap();
        let resolve_cb = resolve_cb.get_mut().as_ref()
            .ok_or(format!("`resolve_cb` not set, cannot forward!"))?;

        // 解析链接的目标模块
        let link_hint = &func.hint;
        let dest_ctx: Arc<AsyncCtx> = resolve_cb(link_hint.clone())?;

        // 把消息转发到目标模块的 rx_queue
        dest_ctx.push_rx(raw_msg.to_vec());

        // 设置返回动作
        dest_ctx.push_action(ctx.seq_no(),
                             ResultAction::ForwardResult(link_hint.clone(), func.clone()));

        Ok(())
    }

    fn return_action_cb(ctx: &RpcEndCtx<Arc<Self>>, res: Vec<u8>) -> rpc::Result<()> {
        let seq_no = ctx.seq_no();
        let action = ctx.data().take_action(seq_no)
            .ok_or(format!("seq_no {} not found", seq_no))?;

        match action {
            ResultAction::Wake(waker) => {
                // 唤醒调用结果的 Future 动作
                // 保存调用结果
                ctx.data().push_action(seq_no, ResultAction::Response(res));
                // 唤醒 Future
                waker.wake();
                Ok(())
            },
            ResultAction::ForwardResult(link_hint, func) => {
                // 转发结果动作
                let mut resolve_cb = ctx.data().resolve_cb.lock().unwrap();
                let resolve_cb = resolve_cb.get_mut().as_ref()
                    .ok_or(format!("`resolve_cb` not set, cannot forward result!"))?;

                // 解析目标模块
                let dest_ctx: Arc<AsyncCtx> = resolve_cb(link_hint)?;

                // 拼接返回消息。此处因为 API 设计的考虑，因此暂时通过此种方法拼接。
                let ser_ctx = SerializeCtx::new();
                let resp = RpcResponseCtx::new(ctx.seq_no(), &ser_ctx, &());
                let resp_msg = resp.make_response(func, res)?;

                // 把消息转发到目标模块的 rx_queue
                dest_ctx.push_rx(resp_msg);

                Ok(())
            },
            _ => Err(format!("seq_no {}: action not support", seq_no).into()),
        }
    }

    pub fn bind_rpc(&self, mut rpc_node: RpcNode<Arc<Self>>) {
        // 添加返回回调
        rpc_node.set_forward_cb(Self::forward_action_cb);
        rpc_node.set_result_cb(Self::return_action_cb);
        // 记录引用
        let mut rpc_ctx = self.rpc_ctx.lock().unwrap();
        rpc_ctx.get_mut().replace(rpc_node);
    }

    pub fn bind_low_level<T>(self: Arc<Self>, ll_ctx: &mut LowLevelCtx<T>)
        where T: Send + Sync + 'static,
    {
        // 添加消息回调
        ll_ctx.set_message_callback(move |msg| {
            let that = self.as_ref();
            if that.prepared() {
                that.push_tx(msg.to_vec());
            } else {
                // 异步未就绪，同步处理
                let mut rpc_ctx = that.rpc_ctx.lock().unwrap();
                rpc_ctx.get_mut().as_ref().unwrap().handle_message(msg).unwrap();
            }
        });
    }

    pub async fn start<T>(self: Arc<Self>, ll_ctx: Arc<LowLevelCtx<T>>)
        where T: Send + Sync + 'static,
    {
        // 启动 tx_future
        let future = HandleTxFuture::new(self.clone());
        tokio::spawn(future);
        // 启动 rx_future
        let future = HandleRxFuture::new(self.clone(), ll_ctx.clone());
        tokio::spawn(future);
        // 等待初始化消息
        loop {
            self.init_notify.notified().await;
            if self.prepared() {
                // 成功初始化
                break;
            }
        }
    }

    /// 异步调用 API
    pub fn request_api(self: Arc<Self>, func: abi::FunctionIdent, args: Vec<u8>) -> AsyncRequestFuture {
        let mut rpc_ctx = self.rpc_ctx.lock().unwrap();
        let req = rpc_ctx.get_mut().as_ref().unwrap().request();

        // 序列化
        let msg = req.make_request(func, args).unwrap();

        AsyncRequestFuture::new(self.clone(), req.seq_no(), msg)
    }

    pub fn alive(&self) -> bool {
        self.alive.lock().unwrap().get()
    }

    pub fn kill(&self) {
        self.alive.lock().unwrap().set(false);
        // 唤醒 tx_wake
        {
            let mut waker = self.tx_waker.lock().unwrap();
            waker.get_mut().as_ref().unwrap().wake_by_ref();
        }
        // 唤醒 rx_wake
        {
            let mut waker = self.rx_waker.lock().unwrap();
            waker.get_mut().as_ref().unwrap().wake_by_ref();
        }
    }

    pub fn prepared(&self) -> bool {
        // 检查 tx_waker
        {
            let mut waker = self.tx_waker.lock().unwrap();
            if waker.get_mut().is_none() {
                // 还没成功
                return false;
            }
        }
        // 检查 rx_waker
        {
            let mut waker = self.rx_waker.lock().unwrap();
            if waker.get_mut().is_none() {
                // 还没成功
                return false;
            }
        }
        true
    }
}
