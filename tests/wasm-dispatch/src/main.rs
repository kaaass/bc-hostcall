//! 同步场景下 Hostcall 的集成测试，本文件为 Wasm 端

#![cfg(target_arch = "wasm32")]

use std::fmt::Debug;

use bc_hostcall::rpc::RpcNode;
use bc_hostcall::serialize::SerializeCtx;
use bc_hostcall::rpc::adapter::WasmSendMessageAdapter;

use exports::*;

mod exports;

pub const MODULE_NAME: &str = "dispatch";

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
    use bc_hostcall::rpc::RpcNode;
    use once_cell::sync::OnceCell;
    use std::hash::Hash;
    use std::hash::Hasher;
    use std::collections::hash_map::DefaultHasher;
    use bc_hostcall::rpc::adapter::SendMessageAdapter;

    use super::*;

    pub static CTX: OnceCell<MockWasmContext> = OnceCell::new();

    use bc_hostcall::set_message_callback;

    fn __bc_message_callback(msg: &[u8]) {
        println!("[Wasm {}]: 接收到 Host 消息：{:?}", MODULE_NAME, msg);
        let ctx = CTX.get().unwrap();
        ctx.rpc_ctx.handle_message(msg).unwrap();
    }

    set_message_callback!(__bc_message_callback);

    #[no_mangle]
    pub extern "C" fn __bc_main() {
        // 初始化内部上下文
        let mut hasher = DefaultHasher::new();
        MODULE_NAME.hash(&mut hasher);
        let mut ctx = MockWasmContext {
            rpc_ctx: RpcNode::new(SerializeCtx::new(),
                                  hasher.finish() as u32,
                                  WasmSendMessageAdapter::new()),
        };
        // 注册导出模块
        let exports = __bc_module_export();
        ctx.rpc_ctx.set_exports(exports);
        // 发送模块名称
        let msg = ctx.rpc_ctx.make_peer_info(MODULE_NAME.to_string());
        let adapter = WasmSendMessageAdapter::new();
        adapter.send_message(&msg).unwrap();
        // 设置上下文
        CTX.set(ctx).unwrap();
        // 把控制权交回 Host。
        println!("[Wasm {}]: 完成初始化", MODULE_NAME);
    }

    #[no_mangle]
    pub extern "C" fn __bc_low_level_wasm_poll() {
        println!("[Wasm {}]: __bc_low_level_wasm_poll()", MODULE_NAME);
    }
}

fn main() {
    __bc::__bc_main();
}
