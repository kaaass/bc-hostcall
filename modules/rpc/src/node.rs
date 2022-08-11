//! Rpc 节点的接口实现

use std::cell::RefCell;

use crate::{abi, adapter, Message, Result, RpcExports, RpcImports, RpcMessage, RpcRequestCtx, RpcResponseCtx, RpcResultCtx};
use serialize::SerializeCtx;

pub type RpcSeqNo = u64;

pub type RpcForwardCallback =
    dyn Fn(RpcSeqNo, abi::FunctionIdent, &[u8]) -> Result<()> + Sync + Send + 'static;

pub struct RpcNode<T>
where
    T: adapter::SendMessageAdapter,
{
    serialize_ctx: SerializeCtx,
    imports: Option<RpcImports>,
    exports: Option<RpcExports>,
    nonce: u32,
    request_num: RefCell<u32>,
    forward_cb: Option<Box<RpcForwardCallback>>,
    sender: T,
}

impl<T> RpcNode<T>
where
    T: adapter::SendMessageAdapter,
{
    /// 创建一个新的 RPC 节点
    ///
    /// `nonce` 为该节点的唯一标识，用于标识该节点的调用请求和调用结果
    pub fn new(serialize_ctx: SerializeCtx, nonce: u32, sender: T) -> Self {
        RpcNode {
            serialize_ctx,
            imports: None,
            exports: None,
            nonce,
            request_num: RefCell::new(0),
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
    where
        CB: Fn(RpcSeqNo, abi::FunctionIdent, &[u8]) -> Result<()> + Sync + Send + 'static,
    {
        self.forward_cb = Some(Box::new(forward_cb));
    }

    pub fn request(&self) -> RpcRequestCtx {
        // 基于 `nonce` 生成一个唯一的 RPC 调用请求序列号 `seq_no`
        let seq_no =
            (self.nonce as u64).checked_shl(32).unwrap() + *self.request_num.borrow() as u64;
        *self.request_num.borrow_mut() += 1;

        // 创建调用请求上下文
        RpcRequestCtx::new(seq_no, &SerializeCtx, &self.sender)
    }

    fn handle_request(&self, seq_no: RpcSeqNo, func: &abi::FunctionIdent, args: &[u8]) -> Result<()> {
        let exports = self.exports.as_ref().ok_or(format!("no exports"))?;
        let cb = exports.get_callback(func).ok_or(format!("no callback for {:?}", func))?;

        // 创建返回上下文
        let ctx = RpcResponseCtx::new(seq_no, &self.serialize_ctx, &self.sender);

        // 调用回调
        cb(&ctx, args)
    }

    pub fn handle_message(&self, raw_msg: &[u8]) -> Result<()> {
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
        let msg: RpcMessage = self.serialize_ctx.deserialize(raw_msg)?;
        let inner = msg.message();
        let func = msg.func();

        match inner {
            Message::Request(args) => {
                // 调用请求
                let result = self.handle_request(msg.seq_no(), &func, &*args);

                match result {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        // 如果没有对应的回调，则调用 `forward_cb` 把这个报文转发给其他模块
                        let forward_cb =
                            self.forward_cb.as_ref().ok_or(format!("no forward_cb"))?;
                        forward_cb(msg.seq_no(), func.clone(), raw_msg)
                    }
                }
            }
            Message::Response(res) => {
                // 返回结果
                let imports = self.imports.as_ref().ok_or(format!("no imports"))?;
                let cb = imports.get_callback(func).ok_or(format!("no callback for {:?}", func))?;

                // 创建返回上下文
                let ctx = RpcResultCtx::new(msg.seq_no(), &self.serialize_ctx);

                // 调用回调
                cb(&ctx, &*res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::LinkHint::Host;

    use crate::RpcResponseCtx;
    use std::cell::Cell;
    use std::sync::Arc;
    use std::sync::Mutex;

    static mut MSG: Option<Vec<u8>> = None;

    struct MockAdapter;

    impl adapter::SendMessageAdapter for MockAdapter {
        fn send_message(&self, msg: &[u8]) -> Result<()> {
            // 保存发送的消息
            unsafe {
                MSG = Some(msg.to_vec());
            }
            Ok(())
        }
    }

    #[test]
    fn test_call_request() {
        let serialize_ctx = SerializeCtx::new();
        let mut node = RpcNode::new(serialize_ctx, 0, MockAdapter);

        // 模块导出了 test 函数
        let mut func = abi::FunctionIdent::new("test");
        func.set_hint(Host);
        let mut exports = RpcExports::new(func.hint.clone());
        let count: Arc<Mutex<Cell<i32>>> = Arc::new(Mutex::new(Cell::new(0)));
        let inner_count = count.clone();
        exports.add_exports(func.clone(), Box::new(move |_: &'_ RpcResponseCtx<'_>, _: &'_ [u8]| {
            let count = inner_count.lock().unwrap();
            count.set(count.get() + 1);
            println!("test 被调用，count = {}", count.get());
            Ok(())
        }));
        node.set_exports(exports);

        // 发送一个调用请求
        node.request().send_request(func.clone(), vec![]).unwrap();

        // 处理调用请求
        node.handle_message(unsafe { MSG.as_ref().unwrap() }).unwrap();

        // 检验结果
        assert_eq!(1, count.lock().unwrap().get());
    }
}
