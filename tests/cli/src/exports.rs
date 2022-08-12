use std::sync::Arc;
use bc_hostcall::async_api::ctx::AsyncCtx;
use bc_hostcall::rpc::{abi, RpcExports};

pub fn init_exports() -> RpcExports<Arc<AsyncCtx>> {
    RpcExports::new(abi::LinkHint::Host)
}
