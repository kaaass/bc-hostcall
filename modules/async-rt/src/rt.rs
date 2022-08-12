use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use low_level::set_message_callback;
use rpc::{abi, RpcEndCtx, RpcNode, RpcSeqNo};
use rpc::adapter::{SendMessageAdapter, WasmSendMessageAdapter};

use crate::queue::QUEUE;

/// WASM 内部的运行时上下文
pub struct WasmRtCtx {
    pub rpc_ctx: RefCell<Option<RpcNode<WasmSendMessageAdapter>>>,
    pub return_actions: RefCell<HashMap<RpcSeqNo, WasmReturnAction>>,
}

/// 返回动作
pub enum WasmReturnAction {
    Wake(Waker),
    Response(Vec<u8>),
}

impl WasmRtCtx {
    pub fn new() -> Self {
        WasmRtCtx {
            rpc_ctx: RefCell::new(None),
            return_actions: RefCell::new(HashMap::new()),
        }
    }
}

thread_local! {
    pub static CTX: WasmRtCtx = WasmRtCtx::new();
}

/// Host 消息处理器
#[allow(dead_code)]
fn host_message_handler(msg: &[u8]) {
    CTX.with(|rt_ctx| {
        let rpc_ctx = rt_ctx.rpc_ctx.borrow();
        if let Some(rpc_ctx) = rpc_ctx.as_ref() {
            rpc_ctx.handle_message(msg).unwrap();
        }
    });
}

set_message_callback!(host_message_handler);

/// WASM 侧的返回消息回调
pub fn result_message_cb(ctx: &RpcEndCtx<WasmSendMessageAdapter>, res: Vec<u8>) -> rpc::Result<()> {
    CTX.with(|rt_ctx| {
        // 唤醒调用结果的等待者
        let mut return_actions = rt_ctx.return_actions.borrow_mut();
        let action = return_actions.remove(&ctx.seq_no());

        match action {
            Some(WasmReturnAction::Wake(waker)) => {
                return_actions.insert(ctx.seq_no(), WasmReturnAction::Response(res));
                // 唤醒 Future
                waker.wake();
            }
            _ => {
                panic!("未知的返回动作");
            }
        }
    });
    Ok(())
}

/// 产生模块入口
#[macro_export]
macro_rules! bc_wasm_module {
    ($name:expr, $export_cb:ident) => {
        #[no_mangle]
        pub extern "C" fn __bc_main() {
            use std::hash::Hash;
            use std::hash::Hasher;
            use std::collections::hash_map::DefaultHasher;
            use bc_hostcall::rpc::RpcNode;
            use bc_hostcall::rpc::adapter::{WasmSendMessageAdapter, SendMessageAdapter};
            use bc_hostcall::serialize::SerializeCtx;

            // 初始化内部上下文
            let mut hasher = DefaultHasher::new();
            $name.hash(&mut hasher);
            let mut rpc_ctx = RpcNode::new(SerializeCtx::new(),
                                      hasher.finish() as u32,
                                      WasmSendMessageAdapter::new());
            // 注册导出模块
            let exports = $export_cb();
            rpc_ctx.set_exports(exports);
            // 设置回调
            rpc_ctx.set_forward_cb(|_, _, _| {
                panic!("模块未导出此函数！");
            });
            rpc_ctx.set_result_cb(bc_hostcall::async_rt::rt::result_message_cb);
            // 发送模块名称
            let msg = rpc_ctx.make_peer_info($name.to_string());
            let adapter = WasmSendMessageAdapter::new();
            adapter.send_message(&msg).unwrap();
            // 设置上下文
            bc_hostcall::async_rt::rt::CTX.with(|ctx| {
                ctx.rpc_ctx.replace(Some(rpc_ctx));
            });
            // 把控制权交回 Host
        }
    };
}

#[no_mangle]
pub extern "C" fn __bc_low_level_wasm_poll() {
    QUEUE.with(|queue| {
        queue.run_all();
    });
}

/// 创建异步 API 请求
pub fn request_api(func: abi::FunctionIdent, args: Vec<u8>) -> WasmAsyncRequestFuture {
    CTX.with(|rt_ctx| {
        let req = rt_ctx.rpc_ctx.borrow();
        let req = req.as_ref().unwrap().request();

        // 序列化
        let msg = req.make_request(func, args).unwrap();

        WasmAsyncRequestFuture::new(req.seq_no(), msg)
    })
}

// 异步请求 API 的包装
pub struct WasmAsyncRequestFuture {
    seq_no: RpcSeqNo,
    msg: Cell<Vec<u8>>,
}

impl WasmAsyncRequestFuture {
    pub fn new(seq_no: RpcSeqNo, msg: Vec<u8>) -> Self {
        WasmAsyncRequestFuture {
            seq_no,
            msg: Cell::new(msg),
        }
    }
}

impl Future for WasmAsyncRequestFuture {
    type Output = crate::Result<Vec<u8>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 检查是否有结果
        CTX.with(|rt_ctx| {
            let mut return_actions = rt_ctx.return_actions.borrow_mut();
            let action = return_actions.remove(&self.seq_no);

            match action {
                None => {
                    // 保存 Waker
                    let waker = cx.waker().clone();
                    return_actions.insert(self.seq_no, WasmReturnAction::Wake(waker));

                    // 发送请求
                    let rpc_ctx = rt_ctx.rpc_ctx.borrow();
                    let rpc_ctx = rpc_ctx.as_ref().unwrap();
                    let req = rpc_ctx.request();
                    let msg = self.msg.take();
                    req.data().send_message(&msg)?;
                    Poll::Pending
                }
                Some(WasmReturnAction::Response(msg)) => {
                    // 获取结果
                    Poll::Ready(Ok(msg))
                }
                Some(action) => {
                    // 不支持的结果类型，放回
                    return_actions.insert(self.seq_no, action);
                    Poll::Pending
                }
            }
        })
    }
}
