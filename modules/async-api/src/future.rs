use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use low_level::host::LowLevelCtx;
use rpc::RpcSeqNo;
use crate::ctx::{AsyncCtx, ResultAction};

/// 处理接受队列信息、进行转发及调用的异步任务
pub struct HandleTxFuture {
    /// 模块的异步上下文
    ctx: Arc<AsyncCtx>,
}

impl HandleTxFuture {
    pub fn new(ctx: Arc<AsyncCtx>) -> Self {
        HandleTxFuture { ctx }
    }
}

impl Future for HandleTxFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 更新 waker
        {
            let waker = self.ctx.tx_waker.lock().unwrap();
            waker.set(Some(cx.waker().clone()));
        }
        self.ctx.init_notify.notify_one();

        // 检查存活
        let alive = self.ctx.alive();

        if alive {
            // 存活，正常操作

            // 检查 tx 是否空，处理消息
            {
                let mut tx_queue = self.ctx.tx_queue.lock().unwrap();
                let mut rpc_ctx = self.ctx.rpc_ctx.lock().unwrap();
                for msg in tx_queue.get_mut().iter() {
                    rpc_ctx.get_mut().as_ref().unwrap().handle_message(msg).unwrap();
                }
                tx_queue.get_mut().clear();
            }

            Poll::Pending
        } else {
            // 死亡，直接返回
            Poll::Ready(())
        }
    }
}

/// 处理发送队列信息，发送至 WASM 并且运行模块的异步任务
pub struct HandleRxFuture<T>
    where T: Send + Sync + 'static,
{
    /// 模块的异步上下文
    ctx: Arc<AsyncCtx>,

    /// 低层接口
    ll_ctx: Arc<LowLevelCtx<T>>,
}

impl<T> HandleRxFuture<T>
    where T: Send + Sync + 'static,
{
    pub fn new(ctx: Arc<AsyncCtx>, ll_ctx: Arc<LowLevelCtx<T>>) -> Self {
        HandleRxFuture {
            ctx,
            ll_ctx,
        }
    }
}

impl<T> Future for HandleRxFuture<T>
    where T: Send + Sync + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 更新 waker
        {
            let waker = self.ctx.rx_waker.lock().unwrap();
            waker.set(Some(cx.waker().clone()));
        }
        self.ctx.init_notify.notify_one();

        // 检查存活
        let alive = self.ctx.alive();

        if alive {
            // 存活，正常操作

            // 检查 rx_queue 是否空，并将其消息发送至 WASM
            {
                let mut rx_queue = self.ctx.rx_queue.lock().unwrap();
                for msg in rx_queue.get_mut().iter() {
                    self.ll_ctx.send_message_to_wasm(msg).unwrap();
                }
                rx_queue.get_mut().clear();
            }

            // 运行 WASM 模块
            self.ll_ctx.wasm_poll().unwrap();

            Poll::Pending
        } else {
            // 死亡，直接返回
            Poll::Ready(())
        }
    }
}

// 异步请求 API 的包装
pub struct AsyncRequestFuture {
    ctx: Arc<AsyncCtx>,
    seq_no: RpcSeqNo,
    msg: Mutex<Cell<Option<Vec<u8>>>>,
    triggered: Mutex<Cell<bool>>,
}

impl AsyncRequestFuture {
    pub fn new(ctx: Arc<AsyncCtx>, seq_no: RpcSeqNo, msg: Vec<u8>) -> Self {
        AsyncRequestFuture {
            ctx,
            seq_no,
            msg: Mutex::new(Cell::new(Some(msg))),
            triggered: Mutex::new(Cell::new(false)),
        }
    }
}

impl Future for AsyncRequestFuture {
    type Output = crate::Result<Vec<u8>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 检查是否有结果
        let action = self.ctx.take_action(self.seq_no);

        match action {
            None => {
                let triggered = self.triggered.lock().unwrap();
                if !triggered.get() {
                    // 第一次调用

                    // 保存 Waker
                    let waker = cx.waker().clone();
                    self.ctx.push_action(self.seq_no, ResultAction::Wake(waker));

                    // 发送请求
                    {
                        let msg = self.msg.lock().unwrap();
                        self.ctx.push_rx(msg.take().unwrap());
                    }

                    // 设置触发标志
                    triggered.set(true);
                }
                Poll::Pending
            },
            Some(ResultAction::Response(msg)) => {
                // 获取结果
                Poll::Ready(Ok(msg))
            },
            Some(action) => {
                // 不支持的结果类型，放回
                self.ctx.push_action(self.seq_no, action);
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc};
    use low_level::host::LowLevelCtx;
    use rpc::{abi, RpcNode};
    use serialize::{ArgsBuilder, SerializeCtx};

    use crate::tests::*;

    use super::*;

    /// 测试 `HandleRxFuture`
    #[tokio::test]
    async fn test_rx_future() {
        let crate::tests::Context { mut store, module, mut linker }
            = guest_prepare("./tests/unittest-future/unittest-future.wasm");

        // 初始化 Lowlevel
        let ll_ctx = LowLevelCtx::new();
        let ll_ctx = Arc::new(ll_ctx);
        ll_ctx.clone().add_to_linker(&mut linker).unwrap();

        // 实例化 WASM
        let instance = linker.instantiate(&mut store, &module).unwrap();
        ll_ctx.attach(&mut store, &instance).unwrap();
        ll_ctx.move_store(store);

        // 初始化异步上下文
        let ctx = Arc::new(AsyncCtx::new());

        // 启用异步任务
        ctx.clone().start(ll_ctx.clone()).await;

        // 向队列里塞消息
        let cctx = ctx.clone();
        let a = tokio::spawn(async move {
            println!("发送消息 1");
            cctx.push_rx("hello".as_bytes().to_vec());

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            println!("发送消息 2");
            cctx.push_rx("world".as_bytes().to_vec());

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            println!("发送消息 3");
            cctx.push_rx("kaaass".as_bytes().to_vec());
        });

        // 关闭异步任务
        let cctx = ctx.clone();
        let b = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            println!("停止任务");
            cctx.kill();
        });

        tokio::join!(a, b);

        // 检查结果
        let mut store = ll_ctx.take_store().unwrap();
        let get_cnt = instance.get_typed_func::<(), i32, _>(&mut store, "get_cnt").unwrap();
        let cnt = get_cnt.call(&mut store, ()).unwrap();
        assert_eq!(cnt, 2);
    }

    async fn wasm_export_to_host(ctx: Arc<AsyncCtx>, param: String) -> crate::Result<String> {
        let ser_ctx = SerializeCtx::new();
        // 函数标识符
        let mut func = abi::FunctionIdent::new("wasm_export_to_host");
        func.set_hint(abi::LinkHint::BcModule("integrate-wasm".to_string()));
        // 参数拼接
        let args = ArgsBuilder::new(&ser_ctx)
            .push(&param).unwrap()
            .build().unwrap();
        // 调用函数
        let ret = ctx.request_api(func, args).await?;
        // 解析返回值
        let result = ser_ctx.deserialize::<String>(&ret)?;
        Ok(result)
    }

    /// 测试 `HandleTxFuture`
    #[tokio::test]
    async fn test_async_call() {
        let crate::tests::Context { mut store, module, mut linker }
            = guest_prepare("./tests/integrate-wasm/integrate-wasm.wasm");

        // 初始化异步上下文
        let ctx = Arc::new(AsyncCtx::new());

        // 初始化 Lowlevel
        let mut ll_ctx = LowLevelCtx::new();
        ctx.clone().bind_low_level(&mut ll_ctx);
        let ll_ctx = Arc::new(ll_ctx);
        ll_ctx.clone().add_to_linker(&mut linker).unwrap();

        // 创建 RpcNode
        let rpc_node = RpcNode::new(
            SerializeCtx::new(),
            0,
            ctx.clone()
        );

        // 绑定 RpcNode
        ctx.bind_rpc(rpc_node);

        // 实例化 WASM
        let instance = linker.instantiate(&mut store, &module).unwrap();
        ll_ctx.attach(&mut store, &instance).unwrap();
        ll_ctx.move_store(store);

        // 调用主函数进行初始化
        ll_ctx.wasm_main().unwrap();

        // 启用异步任务
        ctx.clone().start(ll_ctx.clone()).await;

        // 调用函数
        let ret = wasm_export_to_host(ctx.clone(), "async host".to_string()).await.unwrap();
        println!("wasm_export_to_host(): {}", ret);
    }
}
