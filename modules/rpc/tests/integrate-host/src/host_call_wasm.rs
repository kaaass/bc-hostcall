//! Host 调用函数、Wasm 导出函数的函数调用部分

use once_cell::sync::OnceCell;

use rpc::{abi, Result, RpcEndCtx, RpcNode};
use rpc::adapter::SendMessageAdapter;
use serialize::{ArgsBuilder, SerializeCtx};

use crate::MockHostContext;

/// Wasm 内声明的函数在 Host 处调用的示例。本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_import(host, module = "integrate-wasm")]
/// fn wasm_export_to_host(param: String) -> String;
/// ```
///
/// 实际生成的函数应该是异步的并且返回 Result<T>，然而因为相关 API 依赖于异步模块的
/// 实现，因此此处将该函数拆分为两部分进行编写。
/// 此外，实际生成的函数应该使用异步模块的 Context，并在用户调用时传入。同样基于上述
/// 的原因，暂时由本模块代为实现 Context。
fn wasm_export_to_host(ctx: &MockHostContext, param: String) {
    let req = ctx.rpc_ctx.request();
    // 函数标识符
    let mut func = abi::FunctionIdent::new("wasm_export_to_host");
    func.set_hint(abi::LinkHint::BcModule("integrate-wasm".to_string()));
    // 参数拼接
    let args = ArgsBuilder::new(req.serialize_ctx())
        .push(&param).unwrap()
        .build().unwrap();
    // 发送消息
    let msg = req.make_request(func, args).unwrap();
    req.data().send_message(&msg).unwrap();
    // 完成上述操作后，应该已经调用了 `__bc_wrapper_wasm_export_to_host` 并停止
    // 在异步调用 `wasm_export_to_host` 之前。此时就等待返回报文触发
    // `wasm_export_to_host_return` 回调。如果已经支持异步的话，则此处是在 await
    // 之后。
}

fn wasm_export_to_host_return<T>(ret: &RpcEndCtx<T>, data: Vec<u8>) -> Result<()> {
    // 解析参数
    let result = ret.serialize_ctx().deserialize::<String>(&data).unwrap();
    // 返回结果
    println!("收到 Wasm 的返回值：{}", result);
    Ok(())
}

static CTX: OnceCell<MockHostContext> = OnceCell::new();

fn lowlevel_callback(data: &[u8]) {
    let ctx = CTX.get().unwrap();
    ctx.rpc_ctx.handle_message(data).unwrap();
}

mod tests {
    use std::sync::Arc;

    use low_level::host::LowLevelCtx;
    use rpc::adapter::HostSendMessageAdapter;

    use crate::utils::*;
    use crate::WasiCtx;

    use super::*;

    #[test]
    fn test_wasm_export_to_host() {
        let Context { mut store, module, mut linker } = guest_prepare();

        // 初始化 Lowlevel
        let mut ll_ctx = LowLevelCtx::new();
        ll_ctx.set_message_callback(lowlevel_callback);
        let ll_ctx = Arc::new(ll_ctx);
        ll_ctx.clone().add_to_linker(&mut linker).unwrap();

        // 初始化内部上下文
        let mut ctx = MockHostContext {
            rpc_ctx: RpcNode::new(SerializeCtx::new(), 0,
                                  HostSendMessageAdapter::new(ll_ctx.clone())),
        };

        ctx.rpc_ctx.set_result_cb(wasm_export_to_host_return::<HostSendMessageAdapter<WasiCtx>>);

        // 设置上下文
        CTX.set(ctx).unwrap();

        // 实例化 WASM
        let instance = linker.instantiate(&mut store, &module).unwrap();
        ll_ctx.attach(&mut store, &instance).unwrap();

        // 运行 main
        let main_func = instance.get_typed_func::<(), (), _>(&mut store, "__bc_main").unwrap();
        main_func.call(&mut store, ()).unwrap();

        // 调用
        let ctx = CTX.get().unwrap();
        ll_ctx.move_store(store);
        wasm_export_to_host(&ctx, "host".to_string());
        // take out store
    }
}
