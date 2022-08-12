use bc_hostcall::module_api::module::WasmModule;
use bc_hostcall::rpc::{abi, RpcImports};
use bc_hostcall::serialize::{ArgsBuilder, SerializeCtx};

use crate::Result;

/// Wasm 内声明的函数在 Host 处调用的示例。本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_import(host, module = "dispatch")]
/// fn app(param: String) -> anyhow::Result<String>;
/// ```
pub async fn app(ctx: &WasmModule, param: String) -> Result<String> {
    let ser_ctx = SerializeCtx::new();
    // 函数标识符
    let mut func = abi::FunctionIdent::new("app");
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
