//! RPC 调用中的临时上下文管理

use serialize::{Args, SerializeCtx};

use crate::{abi, Result, RpcSeqNo};

/// RPC 函数调用请求的临时上下文，用于在相关函数回调中提供调用请求所需的 API
pub struct RpcRequestCtx<'a> {
    seq_no: RpcSeqNo,
    serialize_ctx: &'a SerializeCtx,
}

impl<'a> RpcRequestCtx<'a> {
    pub fn new(seq_no: RpcSeqNo, serialize_ctx: &'a SerializeCtx) -> Self {
        RpcRequestCtx {
            seq_no,
            serialize_ctx,
        }
    }

    pub fn send_request(&self, func: abi::FunctionIdent, args: Args) -> Result<()> {
        // TODO: 发送一个 RPC 调用请求报文。报文别忘了要带 seq_no。
        let arg_bytes = args.to_bytes();
        todo!()
    }

    pub fn serialize_ctx(&self) -> &SerializeCtx {
        self.serialize_ctx
    }

    pub fn seq_no(&self) -> RpcSeqNo {
        self.seq_no
    }
}

/// RPC 函数调用回应（真正进行函数调用的时刻）的临时上下文，用于在相关函数回调中提供返回调用结果所需的 API
pub struct RpcResponseCtx<'a> {
    seq_no: RpcSeqNo,
    serialize_ctx: &'a SerializeCtx,
}

impl<'a> RpcResponseCtx<'a> {
    pub fn new(seq_no: RpcSeqNo, serialize_ctx: &'a SerializeCtx) -> Self {
        RpcResponseCtx {
            seq_no,
            serialize_ctx,
        }
    }

    pub fn send_response(&self, result: &[u8]) -> Result<()> {
        // TODO: 发送一个 RPC 调用结果报文
        todo!();
    }

    pub fn serialize_ctx(&self) -> &SerializeCtx {
        self.serialize_ctx
    }

    pub fn seq_no(&self) -> RpcSeqNo {
        self.seq_no
    }
}

/// RPC 函数调用结果返回值的临时上下文，用于在相关函数回调中提供解析、处理调用结果所需的 API
pub struct RpcResultCtx<'a> {
    seq_no: RpcSeqNo,
    serialize_ctx: &'a SerializeCtx,
}

impl<'a> RpcResultCtx<'a> {
    pub fn new(seq_no: RpcSeqNo, serialize_ctx: &'a SerializeCtx) -> Self {
        RpcResultCtx {
            seq_no,
            serialize_ctx,
        }
    }

    pub fn serialize_ctx(&self) -> &SerializeCtx {
        self.serialize_ctx
    }

    pub fn seq_no(&self) -> RpcSeqNo {
        self.seq_no
    }
}
