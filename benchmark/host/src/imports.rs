use bc_hostcall::module_api::module::WasmModule;
use bc_hostcall::rpc::abi;
use bc_hostcall::serialize::{ArgsBuilder, SerializeCtx};

use crate::Result;

pub async fn do_service(ctx: &WasmModule, url: String) -> Result<String> {
    let ser_ctx = SerializeCtx::new();
    // 函数标识符
    let mut func = abi::FunctionIdent::new("do_service");
    func.set_hint(ctx.get_hint());
    // 参数拼接
    let args = ArgsBuilder::new(&ser_ctx)
        .push(&url)?
        .build()?;
    // 调用函数
    let ret = ctx.async_ctx().request_api(func, args).await?;
    // 解析返回值
    let result = ser_ctx.deserialize::<String>(&ret)?;
    Ok(result)
}
