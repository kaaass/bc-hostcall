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
    use low_level::host::LowLevelCtx;

    use super::*;

    pub struct HostSendMessageAdapter {
        ctx: LowLevelCtx,
    }

    impl<'a> HostSendMessageAdapter {
        pub fn new(ctx: LowLevelCtx) -> Self {
            HostSendMessageAdapter { ctx }
        }
    }

    impl<'a> SendMessageAdapter for HostSendMessageAdapter {
        fn send_message(&self, message: &[u8]) -> Result<()> {
            self.ctx.send_message_to_wasm(message)
        }
    }
}
