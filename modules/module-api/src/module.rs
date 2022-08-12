use std::sync::Arc;

use wasmtime::{Engine, Linker, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use async_api::ctx::AsyncCtx;
use low_level::host::LowLevelCtx;
use rpc::{abi, RpcExports, RpcNode};
use serialize::SerializeCtx;

use crate::manager::ModuleManager;
use crate::Result;

/// WASM 模块在本运行时中的封装
pub struct WasmModule {
    name: Option<String>,
    async_ctx: Arc<AsyncCtx>,
    ll_ctx: Option<Arc<LowLevelCtx<WasiCtx>>>,
}

impl WasmModule {
    pub fn new() -> Self {
        WasmModule {
            name: None,
            async_ctx: Arc::new(AsyncCtx::new()),
            ll_ctx: None,
        }
    }

    // 加载模块并进行初始化
    // FIXME: 其实是一个很差劲的封装，大部分操作都是硬编码的，几乎没有可拓展性。但也没有时间去做到
    //        更好了，done is better than nothing.
    pub fn init(&mut self,
                filename: &str,
                host_exports: RpcExports<Arc<AsyncCtx>>,
    ) -> Result<()> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // 链接 WASI 函数
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        // 创建 WASI 上下文
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        let mut store = Store::new(&engine, wasi);

        // 创建 Module 并进行实例化
        let module = wasmtime::Module::from_file(store.engine(), filename)?;

        // 初始化 Lowlevel
        let async_ctx = self.async_ctx.clone();
        let mut ll_ctx = LowLevelCtx::new();
        async_ctx.clone().bind_low_level(&mut ll_ctx);
        let ll_ctx = Arc::new(ll_ctx);
        ll_ctx.clone().add_to_linker(&mut linker)?;

        // 创建 RpcNode
        let mut rpc_node = RpcNode::new(
            SerializeCtx::new(),
            0,
            async_ctx.clone(),
        );

        // 增加 Host 端的导入导出需求
        rpc_node.set_exports(host_exports);

        // 绑定 RpcNode
        async_ctx.bind_rpc(rpc_node);

        // 实例化 WASM
        let instance = linker.instantiate(&mut store, &module)?;
        ll_ctx.attach(&mut store, &instance)?;
        // XX 我们在这里直接移入 Store，因为几乎没有什么情况是需要在外侧操作 Store 的。
        //    就算如果有，`ll_ctx` 也允许我们移出。
        ll_ctx.move_store(store);

        // 调用主函数进行初始化
        // XX 这里可以修改为异步，但是实际上这个函数只是做内部数据结构的初始化，不会有什么
        //    很长的 Block 或者甚至 Polling。所以实际上没有必要异步。
        ll_ctx.wasm_main()?;

        // 获得模块名称（对端模块）
        {
            let mut rpc_ctx = async_ctx.rpc_ctx.lock().unwrap();
            let rpc_ctx = rpc_ctx.get_mut().as_ref().unwrap();
            let peer_name = rpc_ctx.get_peer_name()
                .ok_or("Peer Info not presents!")?;
            self.name = Some(peer_name);
        }

        self.ll_ctx = Some(ll_ctx);

        Ok(())
    }

    /// 启动模块及其异步任务
    ///
    /// 返回的异步任务会在启动完毕后结束。
    pub async fn start(&self) {
        self.async_ctx.clone().start(self.ll_ctx.as_ref().unwrap().clone()).await
    }

    /// 异步请求 API 并返回其结果。相关参数序列化应该在包装函数中进行。
    pub async fn request_api(self: Arc<Self>, func: abi::FunctionIdent, args: Vec<u8>) -> Result<Vec<u8>> {
        self.async_ctx.clone().request_api(func, args).await
    }

    /// 结束模块异步任务。异步任务将在完成相关收尾工作之后在下一次 poll 结束。
    pub fn kill(&self) {
        // 是否需要提供一个 async 的方式允许等待异步任务完成？
        self.async_ctx.kill();
    }

    pub fn attach_to_manager(self: Arc<Self>, manager: Arc<ModuleManager>) {
        // 注册模块解析回调
        let my_hint = self.get_hint();
        self.async_ctx.set_resolve_cb(move |hint| {
            let result = manager.resolve(&hint)
                .ok_or(format!("Failed to resolve module with hint: {:?}", hint))?;

            // 检查是否是本身，避免循环调用
            if result.get_hint() == my_hint {
                return Err(format!("Circular import: {:?}", hint).into());
            }

            Ok(result.async_ctx.clone())
        });
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref().unwrap()
    }

    pub fn get_hint(&self) -> abi::LinkHint {
        abi::LinkHint::BcModule(self.get_name().to_string())
    }

    /// 仅供生成的函数使用，无需直接调用
    #[doc(hidden)]
    pub fn async_ctx(&self) -> Arc<AsyncCtx> {
        self.async_ctx.clone()
    }
}

#[cfg(test)]
mod tests {
    use rpc::abi;
    use serialize::ArgsBuilder;

    use super::*;

    fn init_exports() -> RpcExports<Arc<AsyncCtx>> {
        RpcExports::new(abi::LinkHint::BcModule("integrate-wasm".to_string()))
    }

    async fn wasm_export_to_host(ctx: &WasmModule, param: String) -> Result<String> {
        let ser_ctx = SerializeCtx::new();
        // 函数标识符
        let mut func = abi::FunctionIdent::new("wasm_export_to_host");
        func.set_hint(ctx.get_hint());
        // 参数拼接
        let args = ArgsBuilder::new(&ser_ctx)
            .push(&param).unwrap()
            .build().unwrap();
        // 调用函数
        let ret = ctx.async_ctx().request_api(func, args).await?;
        // 解析返回值
        let result = ser_ctx.deserialize::<String>(&ret)?;
        Ok(result)
    }

    #[tokio::test]
    #[allow(unused_must_use)]
    async fn test_multiple_module() {
        let wasm = "../async-api/tests/integrate-wasm/integrate-wasm.wasm";

        // 加载两遍同一个模块
        let mut mod_a = WasmModule::new();
        mod_a.init(wasm, init_exports()).unwrap();
        println!("mod_a 名称：{}", mod_a.get_name());

        let mut mod_b = WasmModule::new();
        mod_b.init(wasm, init_exports()).unwrap();
        println!("mod_b 名称：{}", mod_b.get_name());

        // 启动
        mod_a.start().await;
        mod_b.start().await;

        // 分别异步调用两个模块的函数
        let mod_a = Arc::new(mod_a);
        let mod_b = Arc::new(mod_b);

        let cmod_a = mod_a.clone();
        let cmod_b = mod_b.clone();
        let task_a = tokio::spawn(async move {
            // 第一次调用 A
            let ret = wasm_export_to_host(cmod_a.as_ref(), "host mod a".to_string()).await.unwrap();
            assert_eq!(ret, "Hello host mod a, I'm a wasm module!".to_string());

            // 第二次调用 B
            let ret = wasm_export_to_host(cmod_b.as_ref(), "host mod b".to_string()).await.unwrap();
            assert_eq!(ret, "Hello host mod b, I'm a wasm module!".to_string());
        });

        let cmod_a = mod_a.clone();
        let cmod_b = mod_b.clone();
        let task_b = tokio::spawn(async move {
            // 第一次调用 B
            let ret = wasm_export_to_host(cmod_b.as_ref(), "host mod b".to_string()).await.unwrap();
            assert_eq!(ret, "Hello host mod b, I'm a wasm module!".to_string());

            // 第二次调用 A
            let ret = wasm_export_to_host(cmod_a.as_ref(), "host mod a".to_string()).await.unwrap();
            assert_eq!(ret, "Hello host mod a, I'm a wasm module!".to_string());
        });

        tokio::join!(task_a, task_b);
    }
}
