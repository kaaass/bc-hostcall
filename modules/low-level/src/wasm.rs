//! 本模块用于提供在 WASM 中进行信息传送的低层接口

use std::alloc::{self, Layout};

use crate::Result;

/// 将一段缓冲区由 WASM 发送至 Host。与 `low_level::host::LowLevelCtx::set_message_callback`
/// 函数设置的回调函数相对应，共同完成消息的发送与接收。
///
/// ## 使用示例
///
/// ```rust
/// use low_level::wasm::send_message_to_host;
///
/// send_message_to_host("hello host".as_bytes()).unwrap();
/// ```
///
pub fn send_message_to_host(msg: &[u8]) -> Result<()> {
    #[link(wasm_import_module = "__bc_low_level")]
    extern "C" {
        #[link_name = "receive_message_from_wasm"]
        fn host_message_handler(msg: *const u8, msg_len: usize);
    }

    // TODO: 应该通过调用 receive_message_from_wasm 函数将信息传送至 Host
    todo!()
}

/// 设置接受 Host 模块消息的回调函数。与 `low_level::host::LowLevelCtx::send_message_to_wasm`
/// 函数相对应，共同完成消息的发送与接收。
///
/// ## 使用示例
///
/// ```rust
/// use low_level::set_message_callback;
///
/// fn receive_message_from_host(msg: &[u8]) {
///    println!("接收到 Host 消息：{:?}", msg);
/// }
///
/// set_message_callback!(receive_message_from_host);
/// ```
///
#[macro_export]
macro_rules! set_message_callback {
    ($cb:ident) => {
        #[no_mangle]
        pub extern "C" fn __bc_low_level_host_message_handler(msg: *const u8, msg_len: usize) {
            // TODO: 可以按照需要随意修改，此处只是示例
            let msg = unsafe {
                std::slice::from_raw_parts(msg, msg_len)
            };
            $cb(msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

// TODO: 增加需要的测试，如验证回调是否可以正常触发
}
