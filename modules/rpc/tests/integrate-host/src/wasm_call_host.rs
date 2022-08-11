//! Host 导出函数、Wasm 调用函数的函数导出部分

use once_cell::sync::OnceCell;

use rpc::{Result, RpcExports, RpcNode, RpcResponseCtx};
use rpc::abi;
use serialize::{Args, SerializeCtx};

use crate::MockHostContext;

/// Host 内导出的函数
fn host_export_to_wasm(param: String) -> String {
    println!("返回：Hello {}, I'm a host!", param);
    format!("Hello {}, I'm a host!", param)
}

/// Host 内声明的函数在 Wasm 处调用的示例。本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_export(host)]
/// fn host_export_to_wasm(param: String) -> String {
///     // ...
/// }
/// ```
///
/// 实际生成的函数应该是异步的并且使用异步模块的 Context。
fn __bc_wrapper_host_export_to_wasm(resp: &RpcResponseCtx, args: &[u8]) -> Result<()> {
    // 函数标识符
    let mut func = abi::FunctionIdent::new("host_export_to_wasm");
    func.set_hint(abi::LinkHint::Host);
    // 参数解析
    let args = Args::from_bytes(resp.serialize_ctx(), args)?;
    let arg0_param: String = args.get::<String>(0).unwrap().clone();
    // 调用函数。实际应该是异步调用的。
    let result: String = host_export_to_wasm(arg0_param);
    // 序列化结果
    let serialized_result = resp.serialize_ctx().serialize(&result)?;
    // 结果回送
    resp.send_response(func, serialized_result)?;
    // 完成调用后，此返回信息应该已被送入 Host 处的消息队列 rx_queue。但在本示例中应该会
    // 直接被分发至 `wasm_export_to_host_return` 函数。
    Ok(())
}

static CTX: OnceCell<MockHostContext> = OnceCell::new();

fn lowlevel_callback(data: &[u8]) {
    let ctx = CTX.get().unwrap();
    // 实际上是先进入队列，之后才调用 `handle_message`。此时 data 可以被转为 'static，
    // 因为 data 肯定是指向 Wasm 的线性内存的某处的，而由于异步任务的原因，这处的内存在单线
    // 程情况下是不会被修改的。这里为了测试则直接进行调用。
    ctx.rpc_ctx.handle_message(data).unwrap();
}

fn init_exports() -> RpcExports {
    let mut exports = RpcExports::new(abi::LinkHint::Host);
    // 添加导出函数的回调
    let func = abi::FunctionIdent::new("host_export_to_wasm");
    exports.add_exports(func, __bc_wrapper_host_export_to_wasm);
    exports
}

mod tests {
    use std::sync::{Arc};
    use low_level::host::LowLevelCtx;
    use rpc::adapter::HostSendMessageAdapter;

    use crate::utils::*;

    use super::*;

    #[test]
    fn test_host_export_to_wasm() {
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
        // 注册导出模块
        let exports = init_exports();
        ctx.rpc_ctx.set_exports(exports);
        // 设置上下文
        CTX.set(ctx).unwrap();

        // 实例化 WASM
        let instance = linker.instantiate(&mut store, &module).unwrap();
        ll_ctx.attach(&mut store, &instance).unwrap();

        // 运行 main
        let main_func = instance.get_typed_func::<(), (), _>(&mut store, "__bc_main").unwrap();
        main_func.call(&mut store, ()).unwrap();

        // 调用
        let trigger_call = instance.get_typed_func::<(), (), _>(&mut store, "trigger_call").unwrap();
        trigger_call.call(&mut store, ()).unwrap();
    }
}
