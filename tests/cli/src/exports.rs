use std::sync::Arc;
use bc_hostcall::async_api::ctx::AsyncCtx;
use bc_hostcall::rpc::{abi, RpcExports, RpcResponseCtx};
use bc_hostcall::serialize::{Args, SerializeCtx};

use crate::Result;

pub fn init_exports() -> RpcExports<Arc<AsyncCtx>> {
    let mut exports = RpcExports::new(abi::LinkHint::Host);
    // 添加导出函数的回调
    let func = abi::FunctionIdent::new("http_get");
    exports.add_exports(func, __bc_wrapper_http_get);
    exports
}

async fn http_get(url: String) -> String {
    let res = reqwest::get(&url).await;
    let res = if let Ok(res) = res {
        res
    } else {
        return "<Failed>".to_string();
    };
    let body = res.text().await;
    if let Ok(body) = body {
        body
    } else {
        "<Failed>".to_string()
    }
}

/// 本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_export(host)]
/// async fn http_get(url: String) -> String {
///     // ...
/// }
/// ```
///
fn __bc_wrapper_http_get(resp: &RpcResponseCtx<Arc<AsyncCtx>>, args: &[u8]) -> Result<()> {
    // 函数标识符
    let mut func = abi::FunctionIdent::new("http_get");
    func.set_hint(abi::LinkHint::Host);
    // 参数解析
    let args = Args::from_bytes(resp.serialize_ctx(), args)?;
    let arg0_param: String = args.get::<String>(0).unwrap().clone();
    // 开启异步任务
    let ctx = resp.data().clone();
    tokio::spawn(async move {
        // 异步调用函数
        let result: String = http_get(arg0_param).await;
        // 序列化结果
        let ser_ctx = SerializeCtx::new();
        let serialized_result = ser_ctx.serialize(&result).unwrap();
        // 结果回送
        let msg = resp.make_response(func, serialized_result).unwrap();
        ctx.push_rx(msg);
    });
    Ok(())
}
