//! RPC 调用中的临时上下文管理
use serde::{Serialize, Deserialize};

use serialize::SerializeCtx;

use crate::{abi, adapter, Result, RpcSeqNo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Request(Vec<u8>),
    Response(Vec<u8>),
}

// 请求消息 便于序列化
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcMessage {
    seq_no: RpcSeqNo,
    func: abi::FunctionIdent,
    message: Message,
}

impl RpcMessage {
    pub fn new(seq_no: RpcSeqNo, func: abi::FunctionIdent, message: Message) -> Self {
        RpcMessage { seq_no, func, message }
    }

    pub fn seq_no(&self) -> RpcSeqNo {
        self.seq_no
    }

    pub fn func(&self) -> &abi::FunctionIdent {
        &self.func
    }

    pub fn message(&self) -> &Message {
        &self.message
    }
}

/// RPC 函数调用请求的临时上下文，用于在相关函数回调中提供调用请求所需的 API
pub struct RpcRequestCtx<'a> {
    seq_no: RpcSeqNo,
    serialize_ctx: &'a SerializeCtx,
    sender: &'a dyn adapter::SendMessageAdapter,
}

impl<'a> RpcRequestCtx<'a> {
    pub fn new(seq_no: RpcSeqNo, serialize_ctx: &'a SerializeCtx, sender: &'a dyn adapter::SendMessageAdapter) -> Self {
        RpcRequestCtx {
            seq_no,
            serialize_ctx,
            sender,
        }
    }

    pub fn send_request(&self, func: abi::FunctionIdent, args: Vec<u8>) -> Result<()> {
        // 拼接报文
        let msg = RpcMessage {
            seq_no: self.seq_no,
            func,
            message: Message::Request(args),
        };

        // 序列化
        let msg_bytes = self.serialize_ctx.serialize(&msg)?;

        // 发送报文
        self.sender.send_message(&msg_bytes)?;

        Ok(())
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
    sender: &'a dyn adapter::SendMessageAdapter,
}

impl<'a> RpcResponseCtx<'a> {
    pub fn new(seq_no: RpcSeqNo, serialize_ctx: &'a SerializeCtx, sender: &'a dyn adapter::SendMessageAdapter) -> Self {
        RpcResponseCtx {
            seq_no,
            serialize_ctx,
            sender,
        }
    }

    pub fn send_response(&self, func: abi::FunctionIdent, result: Vec<u8>) -> Result<()> {
        // 拼接报文
        let msg = RpcMessage {
            seq_no: self.seq_no,
            func,
            message: Message::Response(result),
        };

        // 序列化
        let msg_bytes = self.serialize_ctx.serialize(&msg)?;

        // 发送报文
        self.sender.send_message(&msg_bytes)?;

        Ok(())
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
