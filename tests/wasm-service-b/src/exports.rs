use bc_hostcall::async_rt::spawn_local;
use bc_hostcall::rpc::{Result, RpcExports, RpcResponseCtx};
use bc_hostcall::rpc::abi;
use bc_hostcall::rpc::adapter::{SendMessageAdapter, WasmSendMessageAdapter};
use bc_hostcall::serialize::{Args, SerializeCtx};

use crate::imports::http_get;
use crate::MODULE_NAME;

/// Wasm 内导出的函数
async fn do_service() -> String {

    println!("[WASM service-b]: do_service()");

    // 请求 HTTP
    let url = "http://www.kaaass.net";
    let result = http_get(url.to_string()).await;

    if let Ok(resp) = result {
        resp
    } else {
        "<Failed>".to_string()
    }
}

fn __bc_wrapper_do_service(resp: &RpcResponseCtx<WasmSendMessageAdapter>, args: &[u8]) -> Result<()> {
    // 函数标识符
    let mut func = abi::FunctionIdent::new("app");
    func.set_hint(abi::LinkHint::BcModule(MODULE_NAME.to_string()));
    // 参数解析
    let _args = Args::from_bytes(resp.serialize_ctx(), args)?;
    // 开启异步任务
    let seq_no = resp.seq_no();
    spawn_local(async move {
        // 异步调用函数。
        let result: String = do_service().await;
        // 序列化结果
        let ser_ctx = SerializeCtx::new();
        let serialized_result = ser_ctx.serialize(&result).unwrap();
        // 结果回送
        let msg = RpcResponseCtx::new(seq_no, &ser_ctx, &())
            .make_response(func, serialized_result).unwrap();
        WasmSendMessageAdapter::new().send_message(&msg).unwrap();
    });
    Ok(())
}

pub fn __bc_module_export() -> RpcExports<WasmSendMessageAdapter> {
    let mut exports = RpcExports::new(abi::LinkHint::BcModule(MODULE_NAME.to_string()));
    // 添加导出函数的回调
    let func = abi::FunctionIdent::new("do_service");
    exports.add_exports(func, __bc_wrapper_do_service);
    exports
}
