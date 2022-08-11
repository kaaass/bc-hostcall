//! 同步场景下 Hostcall 的集成测试，本文件为 Host 端

#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]

use std::fmt::Debug;
use wasmtime_wasi::WasiCtx;

use rpc::{RpcNode, adapter::HostSendMessageAdapter};

mod host_call_wasm;
mod wasm_call_host;

pub struct MockHostContext {
    rpc_ctx: RpcNode<HostSendMessageAdapter<WasiCtx>>,
}

impl Debug for MockHostContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockHostContext")
    }
}

mod utils {
    use wasmtime::*;
    use wasmtime_wasi::sync::WasiCtxBuilder;
    use wasmtime_wasi::WasiCtx;

    pub struct Context<T> {
        pub store: Store<T>,
        pub module: Module,
        pub linker: Linker<T>,
    }

    pub fn guest_prepare() -> Context<WasiCtx> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // 链接 WASI 函数
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

        // 创建 WASI 上下文
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        let store = Store::new(&engine, wasi);

        // 创建 Module 并进行实例化
        let module = Module::from_file(store.engine(),
                                       "../integrate-wasm/integrate-wasm.wasm").unwrap();

        Context { store, module, linker }
    }
}
