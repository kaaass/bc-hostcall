use bc_hostcall::rpc::abi;
use bc_hostcall::serialize::{ArgsBuilder, SerializeCtx};

use crate::Result;

pub async fn http_get(url: String) -> Result<String> {
    let ser_ctx = SerializeCtx::new();
    // 函数标识符
    let mut func = abi::FunctionIdent::new("http_get");
    func.set_hint(abi::LinkHint::Host);
    // 参数拼接
    let args = ArgsBuilder::new(&ser_ctx)
        .push(&url).unwrap()
        .build().unwrap();
    // 调用函数
    let ret = bc_hostcall::async_rt::rt::request_api(func, args).await?;
    // 解析返回值
    let result = ser_ctx.deserialize::<String>(&ret)?;
    Ok(result)
}
