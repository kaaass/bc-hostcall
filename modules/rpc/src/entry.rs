//! `entry` 模块主要的用途是维护某一个 `RpcNode` 具有的导入、导出函数表。

use std::collections::HashMap;

use crate::{abi, Result, RpcResponseCtx, RpcResultCtx};

/// 导出函数的回调。第一个参数为发送返回结果的 RPC 上下文，第二个参数为反序列化前的参数。
pub type RpcExportCallback<T> = dyn Fn(&RpcResponseCtx<T>, &[u8]) -> Result<()> + Sync + Send + 'static;

/// 导出函数表
///
/// 由于模块的导出函数都具有相同的关于此模块的链接提示，因此可以统一进行设置。
pub struct RpcExports<T> {
    hint: abi::LinkHint,
    exports_map: HashMap<String, Box<RpcExportCallback<T>>>,
}

impl<T> RpcExports<T> {
    pub fn new(hint: abi::LinkHint) -> Self {
        Self {
            hint,
            exports_map: HashMap::new(),
        }
    }

    /// 添加一个导出函数到导出表
    pub fn add_exports<CB>(&mut self, mut func: abi::FunctionIdent, cb: CB)
        where CB: Fn(&RpcResponseCtx<T>, &[u8]) -> Result<()> + Sync + Send + 'static
    {
        // 添加链接提示
        func.set_hint(self.hint.clone());
        // 添加导出函数
        self.exports_map.insert(func.name.clone(), Box::new(cb));
    }

    /// 根据链接提示在当前导出表中查找回调函数
    pub fn get_callback(&self, func: &abi::FunctionIdent) -> Option<&RpcExportCallback<T>> {
        if func.hint == self.hint {
            // 目标为当前模块，尝试返回对应导出函数
            self.exports_map.get(&func.name).map(|cb| &**cb)
        } else {
            None
        }
    }
}

/// 导出函数的回调。第一个参数为发送返回结果的 RPC 上下文，第二个参数为反序列化前的参数。
pub type RpcImportCallback = dyn Fn(&RpcResultCtx, &[u8]) -> Result<()> + Sync + Send + 'static;

/// 导入函数表
pub struct RpcImports {
    imports_map: HashMap<abi::FunctionIdent, Box<RpcImportCallback>>,
}

impl RpcImports {
    pub fn new() -> Self {
        Self {
            imports_map: HashMap::new(),
        }
    }

    /// 添加一个导入函数到导入表
    pub fn add_imports<CB>(&mut self, func: abi::FunctionIdent, cb: CB)
        where CB: Fn(&RpcResultCtx, &[u8]) -> Result<()> + Sync + Send + 'static
    {
        self.imports_map.insert(func, Box::new(cb));
    }

    /// 根据链接提示在当前导入表中查找回调函数
    pub fn get_callback(&self, func: &abi::FunctionIdent) -> Option<&RpcImportCallback> {
        self.imports_map.get(func).map(|cb| &**cb)
    }
}
