//! 本模块用于提供在 Host 中进行信息传送的低层接口
use super::*;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;

use std::sync::Mutex;
use std::rc::*;
use crate::Result;


// TODO: 完善结构体，增加需要的字段，比如回调表啊、wasmtime 所需的数据结构等等
pub struct LowLevelCtx<T> {
    engine: Engine,
    linker: Linker<T>,
    store: Store<T>,
    instance: Instance,
    memory: Memory,
    callback: Option<Box<dyn Fn(&[u8]) -> ()>>,
}

static mut LowLevelCtxObjs: Vec<LowLevelCtx<WasiCtx>> = Vec::new();

impl LowLevelCtx<WasiCtx> {

    pub fn new(wasm_file_path: &str) -> Self {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // 链接 WASI 函数
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        
        // 创建 WASI 上下文
        let wasi = WasiCtxBuilder::new().build();
        let mut store = Store::new(&engine, wasi);

        // 创建 Module 并进行实例化
        let module = Module::from_file(store.engine(), wasm_file_path).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        return LowLevelCtx {engine, linker, store, instance, memory, callback: None};
    }

    /// 将 LowLevelCtx 与 wasmtime 实例绑定
    pub fn attach(self: &mut Rc<Self>) -> Result<()> {
        // TODO: 将内部的信息处理函数 __bc_low_level receive_message_from_wasm 注册到 Module 中。
        //       可以使用 `linker.func_wrap` 注册。函数内应该获得消息的引用，并对消息回调进行调用。

        self.linker.func_wrap("__bc_low_level", "receive_message_from_wasm", |msg: u32, msg_len: u32| {
            let memory_data = self.memory.data(&self.store);
        })?;
        
        // TODO: 准备发送消息所必要的函数。可以通过 `instance.get_typed_func` 获取到 WASM 内的处理函数
        
        let host_msg_handler = self.instance.get_typed_func::<(u32, u32), (), _>(&mut self.store, "__bc_low_level_host_message_handler")?;
        let wasm_realloc = self.instance.get_typed_func::<(u32, u32, u32, u32), u32, _>(&mut self.store, "canonical_abi_realloc")?;
        let wasm_free = self.instance.get_typed_func::<(u32, u32, u32), (), _>(&mut self.store, "canonical_abi_free")?;



        return Ok(());
    }

    fn receive_message_from_wasm(msg: u32, msg_len: u32) {
        //let memory_data = self.memory.data(&self.store);
        //let msg_start = msg as usize;
        //let msg_len = msg_len as usize;
        //let real_mem_data = &memory_data[msg_start..msg_start + msg_len];
        //callback(real_mem_data.clone());
        //if let Some(func_box) = self.callback {
            
            
        //}
    }

    /// 设置接受 WASM 模块消息的回调函数。与 `low_level::wasm::send_message_to_host` 函数
    /// 相对应，共同完成消息的发送与接收。
    ///
    /// ## 使用示例
    ///
    /// ```rust
    /// use low_level::host::LowLevelCtx;
    /// fn receive_message_from_wasm(msg: &[u8]) {
    ///    println!("接收到 WASM 消息：{:?}", msg);
    /// }
    ///
    /// let mut ctx = LowLevelCtx::new();
    /// // 绑定 wasmtime 实例 ...
    /// ctx.set_message_callback(receive_message_from_wasm);
    /// ```
    ///
    pub fn set_message_callback<F>(&mut self, cb: F)
        where F: Fn(&[u8]) -> () + 'static
    {
        // 注册真正接受信息的回调函数
        self.callback = Some(Box::new(cb));
    }

    /// 将消息发送至 WASM 模块
    ///
    /// ## 使用示例
    ///
    /// ```rust
    /// use low_level::host::LowLevelCtx;
    ///
    /// let ctx = LowLevelCtx::new();
    /// // 绑定 wasmtime 实例 ...
    /// ctx.send_message_to_wasm("hello wasm".as_bytes()).unwrap();
    /// ```
    ///
    pub fn send_message_to_wasm(&self, msg: &[u8]) -> Result<()> {
        // TODO: 进行内存分配、调用 `__bc_low_level_host_message_handler`、销毁内存
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

// TODO: 增加需要的测试，如验证回调是否可以正常触发。在必要时可以使用 Mock 的方式模拟 WASM 端。
}
