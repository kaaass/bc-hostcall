//! 对消息解析与发送的适配接口

#[cfg(not(target_arch = "wasm32"))]
pub use host::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

use crate::Result;

/// 对 `low-level` 模块消息发送 API 的适配接口
pub trait SendMessageAdapter {
    fn send_message(&self, message: &[u8]) -> Result<()>;
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use low_level::wasm::send_message_to_host;

    use super::*;

    pub struct WasmSendMessageAdapter;

    impl WasmSendMessageAdapter {
        pub fn new() -> Self {
            WasmSendMessageAdapter
        }
    }

    impl SendMessageAdapter for WasmSendMessageAdapter {
        fn send_message(&self, message: &[u8]) -> Result<()> {
            send_message_to_host(message)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod host {
    use std::sync::Arc;
    use low_level::host::LowLevelCtx;

    use super::*;

    pub struct HostSendMessageAdapter<T>
        where T: Send + Sync + 'static,
    {
        ctx: Arc<LowLevelCtx<T>>,
    }

    impl<'a, T> HostSendMessageAdapter<T>
        where T: Send + Sync + 'static,
    {
        pub fn new(ctx: Arc<LowLevelCtx<T>>) -> Self {
            HostSendMessageAdapter { ctx }
        }
    }

    impl<'a, T> SendMessageAdapter for HostSendMessageAdapter<T>
        where T: Send + Sync + 'static,
    {
        fn send_message(&self, message: &[u8]) -> Result<()> {
            self.ctx.send_message_to_wasm(message)
        }
    }
}
