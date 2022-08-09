//! Rpc 节点的接口实现

use serialize::SerializeCtx;
use crate::{RpcImports, RpcExports, RpcRequestCtx, Result, abi, adapter};

pub type RpcSeqNo = u64;

pub type RpcForwardCallback = dyn Fn(RpcSeqNo, abi::FunctionIdent, &[u8])
    -> Result<()> + Sync + Send + 'static;

pub struct RpcNode<T>
    where T: adapter::SendMessageAdapter
{
    serialize_ctx: SerializeCtx,
    imports: Option<RpcImports>,
    exports: Option<RpcExports>,
    nonce: u64,
    forward_cb: Option<Box<RpcForwardCallback>>,
    sender: T,
}

impl<T> RpcNode<T>
    where T: adapter::SendMessageAdapter
{
    /// 创建一个新的 RPC 节点
    ///
    /// `nonce` 为该节点的唯一标识，用于标识该节点的调用请求和调用结果
    pub fn new(serialize_ctx: SerializeCtx, nonce: u64, sender: T) -> Self {
        RpcNode {
            serialize_ctx,
            imports: None,
            exports: None,
            nonce,
            forward_cb: None,
            sender,
        }
    }

    pub fn set_imports(&mut self, imports: RpcImports) {
        self.imports = Some(imports);
    }

    pub fn set_exports(&mut self, exports: RpcExports) {
        self.exports = Some(exports);
    }

    pub fn set_forward_cb<CB>(&mut self, forward_cb: CB)
        where CB: Fn(RpcSeqNo, abi::FunctionIdent, &[u8]) -> Result<()> + Sync + Send + 'static
    {
        self.forward_cb = Some(Box::new(forward_cb));
    }

    pub fn request(&self) -> RpcRequestCtx {
        // TODO: 基于 `nonce` 生成一个唯一的 RPC 调用请求序列号 `seq_no`
        let seq_no = 0;
        RpcRequestCtx::new(seq_no, &SerializeCtx)
    }

    pub fn handle_message(&self, msg: &[u8]) -> Result<()> {
        // TODO: 处理接收到的报文
        // 对于收到的报文，首先要将其解码（可以反序列化为 `RpcMessage` 之类的）。
        // 因为这一次解码主要是用来判断如何处理报文的，所以不用反序列化详细的数据
        // （比如调用参数、返回值）。只需要获得报文的类型（调用请求、返回结果）和
        // `abi::FunctionIdent`（调用请求）、是否发生错误（返回结果）即可。
        // - 如果报文是调用请求，则需要调用 `exports` 中的对应的回调。如果没有
        //   对应的回调，则调用 `forward_cb` 把这个报文转发给其他模块。如果没
        //   有定义 `forward_cb` 则失败（发送一个返回结果发生错误报文）。
        // - 如果报文是返回结果，并且没有发生错误，则需要调用 `imports` 中的对
        //   应的回调。如果没有对应的回调，则返回错误。
        // - 如果报文是返回结果，并且发生错误，则返回错误。
        todo!();
    }
}
