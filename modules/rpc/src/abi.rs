//! 函数调用相关的 ABI 定义，用于定位函数符号、检验调用数据等

use serde::{Deserialize, Serialize};

/// 链接函数时提供给链接器的提示
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkHint {
    /// 链接目标为 Host 端实现函数
    Host,
    /// 链接目标为指定名称的 Bc Hostcall Module 实现的函数
    BcModule(String),
    /// 链接目标为指定名称的 WASM Module 实现的函数
    NativeModule(String),
}

/// 函数标识符，用于提供链接器以确定调用的目标函数
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionIdent {
    pub name: String,
    pub hint: LinkHint,
}

impl FunctionIdent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hint: LinkHint::Host,
        }
    }

    /// 设置链接提示
    pub fn set_hint(&mut self, hint: LinkHint) {
        self.hint = hint;
    }
}
