//! RPC 负责确定 WASM 与 Host 之间的通信方式，并处理部分通信动作

pub use context::*;
pub use entry::*;
pub use node::*;

pub mod abi;
pub mod adapter;
mod entry;
mod node;
mod context;

// FIXME: 此处的错误类型仅仅是最简单，可用于容纳任何错误的类型。而实际上好的错误类型
//        应该囊括更加细节的错误信息。此处仅为适应短时间的开发需求而临时设计。
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
