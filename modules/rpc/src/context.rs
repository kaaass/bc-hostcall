//! RPC 调用中的临时上下文管理
extern crate serde_json;
use serde::{Serialize, Deserialize};

use serialize::{Args, SerializeCtx};

use crate::{abi, adapter, Result, RpcSeqNo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
    Result,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestMessage {
    func: abi::FunctionIdent,
    args: Vec<u8>,
}

impl RequestMessage {
    pub fn new(func: abi::FunctionIdent, args: Vec<u8>) -> Self {
        RequestMessage { func, args}
    }

    pub fn func(&self) -> abi::FunctionIdent {
        self.func.clone()
    }

    pub fn args(&self) -> Vec<u8> {
        self.args.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseMessage {
    result: Vec<u8>,
}

impl ResponseMessage {
    pub fn new(result: Vec<u8>) -> Self {
        ResponseMessage { result }
    }

    pub fn result(&self) -> Vec<u8> {
        self.result.clone()
    }

}

// 请求消息 便于序列化
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RPCMessage {
    seq_no: RpcSeqNo,
    msg_type: MessageType,
    message: String,
}

impl RPCMessage {
    pub fn new(seq_no: RpcSeqNo, msg_type: MessageType, message: String) -> Self {
        RPCMessage { seq_no, msg_type, message}
    }

    pub fn seq_no(&self) -> RpcSeqNo {
        self.seq_no
    }

    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

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

    pub fn make_request(&self, func: abi::FunctionIdent, args: &[u8]) -> Option<String> {
        // TODO: 发送一个 RPC 调用请求报文。报文别忘了要带 seq_no。 type = 1
        let requestmsg = RequestMessage::new(func, args.to_vec());
        let message = RPCMessage::new(
            self.seq_no, 
            MessageType::Request, 
            serde_json::to_string(&requestmsg).unwrap());
        // 序列化
        let serial = serde_json::to_string(&message).unwrap();
        // println!("serial = {}", serial);
        Some(serial)
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

    pub fn make_response(&self, result: &[u8]) -> Option<String> {
        // TODO: 发送一个 RPC 调用结果报文 type = 2
        let responsemsg = ResponseMessage::new(result.to_vec());
        let message = RPCMessage::new(
            self.seq_no, 
            MessageType::Response, 
            serde_json::to_string(&responsemsg).unwrap());
        // 序列化
        let serial = serde_json::to_string(&message).unwrap();
        // println!("serial = {}", serial);
        Some(serial)
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
