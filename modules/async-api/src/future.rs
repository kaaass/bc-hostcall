use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use low_level::host::LowLevelCtx;
use crate::ctx::AsyncCtx;

/// 处理接受队列信息、进行转发及调用的异步任务
struct HandleTxFuture {}

impl Future for HandleTxFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 更新 waker
        // 检查 tx 是否空，处理消息
        // 检查模块是否存活，视情况返回
        todo!()
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc};
    use low_level::host::LowLevelCtx;

    use crate::tests::*;

    use super::*;

    /// 测试 `HandleRxFuture`
    #[tokio::test]
    async fn test_rx_future() {
        let crate::tests::Context { mut store, module, mut linker } = guest_prepare();

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
        ctx.clone().spawn(ll_ctx.clone());

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
}
