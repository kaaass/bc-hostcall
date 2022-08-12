//! Wasm 导出函数、Host 调用函数的函数导出部分

use bc_hostcall::rpc::{Result, RpcExports, RpcResponseCtx};
use bc_hostcall::rpc::abi;
use bc_hostcall::serialize::Args;
use bc_hostcall::rpc::adapter::{WasmSendMessageAdapter, SendMessageAdapter};

use crate::MODULE_NAME;

/// Wasm 内导出的函数
fn app(param: String) -> String {
    format!("Hello {}, I'm a wasm module!", param)
}

/// Wasm 内声明的函数在 Host 处调用的示例。本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_export]
/// fn app(param: String) -> String {
///     // ...
/// }
/// ```
///
/// 实际生成的函数应该是异步的并且使用异步模块的 Context。
fn __bc_wrapper_app(resp: &RpcResponseCtx<WasmSendMessageAdapter>, args: &[u8]) -> Result<()> {
    // 函数标识符
    let mut func = abi::FunctionIdent::new("app");
    func.set_hint(abi::LinkHint::BcModule(MODULE_NAME.to_string()));
    // 参数解析
    let args = Args::from_bytes(resp.serialize_ctx(), args)?;
    let arg0_param: String = args.get::<String>(0).unwrap().clone();
    // 调用函数。实际应该是异步调用的。
    let result: String = app(arg0_param);
    // 序列化结果
    let serialized_result = resp.serialize_ctx().serialize(&result)?;
    // 结果回送
    let msg = resp.make_response(func, serialized_result)?;
    resp.data().send_message(&msg)?;
    Ok(())
}

/// 本函数应该由以下代码自动生成：
///
/// ```ignore
/// bc_export_module!("dispatch",
///     app
/// );
/// ```
pub fn __bc_module_export() -> RpcExports<WasmSendMessageAdapter> {
    let mut exports = RpcExports::new(abi::LinkHint::BcModule(MODULE_NAME.to_string()));
    // 添加导出函数的回调
    let func = abi::FunctionIdent::new("app");
    exports.add_exports(func, __bc_wrapper_app);
    exports
}
