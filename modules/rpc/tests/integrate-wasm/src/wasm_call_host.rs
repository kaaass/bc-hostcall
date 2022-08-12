//! Wasm 调用函数、Host 导出函数的函数调用部分

use rpc::{abi, Result, RpcEndCtx};
use rpc::adapter::SendMessageAdapter;
use serialize::ArgsBuilder;

use crate::__bc::CTX;

/// Host 内声明的函数在 Wasm 处调用的示例。本函数应该是由如下的方法签名所自动生成的：
///
/// ```ignore
/// #[bc_import]
/// fn host_export_to_wasm(param: String) -> String;
/// ```
fn host_export_to_wasm(param: String) {
    let ctx = CTX.get().unwrap();
    let req = ctx.rpc_ctx.request();
    // 函数标识符
    let mut func = abi::FunctionIdent::new("host_export_to_wasm");
    func.set_hint(abi::LinkHint::Host);
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

pub fn host_export_to_wasm_return<T>(ret: &RpcEndCtx<T>, data: Vec<u8>) -> Result<()> {
    // 解析参数
    let result = ret.serialize_ctx().deserialize::<String>(&data).unwrap();
    // 返回结果
    println!("收到 Wasm 的返回值：{}", result);
    Ok(())
}

#[no_mangle]
pub extern "C" fn trigger_call() {
    // 调用函数
    host_export_to_wasm("wasm".to_string());
}
