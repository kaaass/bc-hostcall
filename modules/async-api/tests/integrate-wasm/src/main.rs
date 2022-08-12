//! 同步场景下 Hostcall 的集成测试，本文件为 Wasm 端

#![cfg(target_arch = "wasm32")]

use host_call_wasm::*;
use rpc::adapter::SendMessageAdapter;
use rpc::adapter::WasmSendMessageAdapter;
use rpc::RpcNode;
use serialize::SerializeCtx;
use std::fmt::Debug;

mod host_call_wasm;

pub struct MockWasmContext {
    rpc_ctx: RpcNode<WasmSendMessageAdapter>,
}

unsafe impl Send for MockWasmContext {}

impl Debug for MockWasmContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockWasmContext")
    }
}

/// 本模块应该由以下代码自动生成：
///
/// ```ignore
/// #[bc_wasm_main]
/// // 或者
/// bc_wasm_main!(async_main)
/// ```
pub(crate) mod __bc {
    use low_level::set_message_callback;
    use once_cell::sync::OnceCell;

    use super::*;

    pub static CTX: OnceCell<MockWasmContext> = OnceCell::new();

    fn __bc_message_callback(msg: &[u8]) {
        println!("接收到 Host 消息：{:?}", msg);

        let ctx = CTX.get().unwrap();
        ctx.rpc_ctx.handle_message(msg).unwrap();
    }

    set_message_callback!(__bc_message_callback);

    #[no_mangle]
    pub extern "C" fn __bc_main() {
        // 初始化内部上下文
        let mut ctx = MockWasmContext {
            rpc_ctx: RpcNode::new(SerializeCtx::new(), 123123, WasmSendMessageAdapter::new()),
        };
        // 注册导出模块
        let exports = __bc_module_export();
        ctx.rpc_ctx.set_exports(exports);
        // 发送模块名称
        let msg = ctx.rpc_ctx.make_peer_info("integrate-wasm".to_string());
        let adapter = WasmSendMessageAdapter::new();
        adapter.send_message(&msg).unwrap();
        // 设置上下文
        CTX.set(ctx).unwrap();
        // 此时应该启动 Wasm 内的异步运行时运行用户的异步 `main`。不过此处没有实现，
        // 只是把控制权交回 Host。
        println!("完成初始化");
    }

    #[no_mangle]
    pub extern "C" fn __bc_low_level_wasm_poll() {
        println!("__bc_low_level_wasm_poll()");
    }
}

fn main() {
    __bc::__bc_main();
}
