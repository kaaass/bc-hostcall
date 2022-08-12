//! 同步场景下 Hostcall 的集成测试，本文件为 Wasm 端

#![cfg(target_arch = "wasm32")]

use bc_hostcall::bc_wasm_module;
use exports::*;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

mod exports;
pub mod imports;

pub const MODULE_NAME: &str = "dispatch";

bc_wasm_module!(MODULE_NAME, __bc_module_export);

fn main() {}
