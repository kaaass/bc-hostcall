use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
struct HandleRxFuture {}

impl Future for HandleRxFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 更新 waker
        // 检查 rx 是否空，消息发送至 WASM
        // 运行 WASM 模块
        // 检查模块是否存活，视情况返回
        todo!()
    }
}
