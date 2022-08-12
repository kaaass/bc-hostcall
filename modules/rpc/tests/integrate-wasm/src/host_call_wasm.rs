//! Wasm 导出函数、Host 调用函数的函数导出部分

use rpc::{Result, RpcExports, RpcResponseCtx};
use rpc::abi;
use serialize::Args;

/// Wasm 内导出的函数
fn wasm_export_to_host(param: String) -> String {
    format!("Hello {}, I'm a wasm module!", param)
}

/// Wasm 内声明的函数在 Host 处调用的示例。本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_export]
/// fn wasm_export_to_host(param: String) -> String {
///     // ...
/// }
/// ```
///
/// 实际生成的函数应该是异步的并且使用异步模块的 Context。
fn __bc_wrapper_wasm_export_to_host(resp: &RpcResponseCtx, args: &[u8]) -> Result<()> {
    // 函数标识符
    let mut func = abi::FunctionIdent::new("wasm_export_to_host");
    func.set_hint(abi::LinkHint::BcModule("integrate-wasm".to_string()));
    // 参数解析
    let args = Args::from_bytes(resp.serialize_ctx(), args)?;
    let arg0_param: String = args.get::<String>(0).unwrap().clone();
    // 调用函数。实际应该是异步调用的。
    let result: String = wasm_export_to_host(arg0_param);
    // 序列化结果
    let serialized_result = resp.serialize_ctx().serialize(&result)?;
    // 结果回送
    resp.make_response(func, serialized_result)?;
    // 完成调用后，此返回信息应该已被送入 Host 处的消息队列 rx_queue。但在本示例中应该会
    // 直接被分发至 `wasm_export_to_host_return` 函数。
    Ok(())
}

/// 本函数应该由以下代码自动生成：
///
/// ```ignore
/// bc_export_module!("integrate-wasm",
///     wasm_export_to_host
/// );
/// ```
pub fn __bc_module_export() -> RpcExports {
    let mut exports = RpcExports::new(abi::LinkHint::BcModule("integrate-wasm".to_string()));
    // 添加导出函数的回调
    let func = abi::FunctionIdent::new("wasm_export_to_host");
    exports.add_exports(func, __bc_wrapper_wasm_export_to_host);
    exports
}
