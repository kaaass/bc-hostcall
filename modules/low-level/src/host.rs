//! 本模块用于提供在 Host 中进行信息传送的低层接口

use crate::Result;

use wasmtime::{TypedFunc, Linker, Caller, Instance, Memory, Trap, Store, AsContextMut};
use std::sync::{Arc, Mutex};
use std::cell::Cell;
use std::mem;

type OptionWrapper<T> = Mutex<Cell<Option<T>>>;

struct InstanceCtx {
    canonical_abi_free: TypedFunc<(i32, i32, i32), ()>,
    canonical_abi_realloc: TypedFunc<(i32, i32, i32, i32), i32>,
    host_message_handler: TypedFunc<(i32, i32), ()>,
    memory: Memory,
}

pub struct LowLevelCtx<T>
    where T: Send + Sync + 'static,
{
    instance_ctx: OptionWrapper<InstanceCtx>,
    message_cb: Option<Box<dyn Fn(&[u8]) -> () + Send + Sync + 'static>>,
    /// 临时 caller，用来处理嵌套 call
    oneshot_caller: OptionWrapper<Caller<'static, T>>,
    /// 临时 store
    temp_store: OptionWrapper<Store<T>>
}

impl<T> LowLevelCtx<T>
    where T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            instance_ctx: Mutex::new(Cell::new(None)),
            message_cb: None,
            oneshot_caller: Mutex::new(Cell::new(None)),
            temp_store: Mutex::new(Cell::new(None)),
        }
    }

    /// 将 LowLevelCtx 所需的信息处理函数注册到 WASM Linker
    pub fn add_to_linker(self: Arc<Self>, linker: &mut Linker<T>) -> Result<()> {
        // 将内部的信息处理函数 __bc_low_level receive_message_from_wasm 注册到 Module 中
        // 用于对从 WASM 模块发送过来的消息进行处理
        let cb = move |mut caller: Caller<'_, T>, msg: i32, msg_len: i32| {

            // 从 WASM 的线性内存中获得消息引用
            let memory = &get_memory(&mut caller, "memory")
                .unwrap(); // FIXME: unwrap
            let (mem, _) = memory.data_and_store_mut(&mut caller);
            let mem_base = mem.as_ptr();
            let msg = unsafe {
                std::slice::from_raw_parts(mem_base.offset(msg as isize),
                                           msg_len as usize)
            };

            // 保存 Caller
            unsafe {
                let temp_caller = self.oneshot_caller.lock().unwrap();
                let caller: Caller<'static, T> = std::mem::transmute(caller);
                temp_caller.replace(Some(caller));
            }

            // 调用回调
            (self.message_cb.as_ref().unwrap())(msg);

            // 丢弃 Caller，无论是否已经被消耗
            {
                let temp_caller = self.oneshot_caller.lock().unwrap();
                temp_caller.replace(None);
            }
        };
        linker.func_wrap("__bc_low_level", "receive_message_from_wasm", cb)?;

        Ok(())
    }

    /// 将 LowLevelCtx 与 wasmtime 实例绑定
    pub fn attach(&self, mut store: impl AsContextMut, instance: &Instance) -> Result<()> {
        // 准备发送消息所必要的函数。可以通过 `instance.get_typed_func` 获取到 WASM 内的处理函数
        let mut wasm_funcs = self.instance_ctx.lock().unwrap();
        let wasm_funcs = wasm_funcs.get_mut();

        let canonical_abi_realloc = instance.get_typed_func(
            store.as_context_mut(), "canonical_abi_realloc")?;
        let canonical_abi_free = instance.get_typed_func(
            store.as_context_mut(), "canonical_abi_free")?;
        let host_message_handler = instance.get_typed_func(
            store.as_context_mut(), "__bc_low_level_host_message_handler")?;

        *wasm_funcs = Some(InstanceCtx {
            canonical_abi_realloc,
            canonical_abi_free,
            host_message_handler,
            memory: instance.get_memory(store.as_context_mut(), "memory").unwrap(), // FIXME
        });

        Ok(())
    }

    /// 把 Store 移入
    pub fn move_store(&self, store: Store<T>) {
        let temp_store = self.temp_store.lock().unwrap();
        temp_store.replace(Some(store));
    }

    /// 取回移入的 Store
    pub fn take_store(&self) -> Option<Store<T>> {
        let temp_store = self.temp_store.lock().unwrap();
        temp_store.take()
    }

    /// 设置接受 WASM 模块消息的回调函数。与 `low_level::wasm::send_message_to_host` 函数
    /// 相对应，共同完成消息的发送与接收。
    ///
    /// ## 使用示例
    ///
    /// ```ignore
    /// use low_level::host::LowLevelCtx;
    /// fn receive_message_from_wasm(msg: &[u8]) {
    ///    println!("接收到 WASM 消息：{:?}", msg);
    /// }
    ///
    /// let mut ctx = LowLevelCtx::new(...);
    /// // 绑定 wasmtime 实例 ...
    /// ctx.set_message_callback(receive_message_from_wasm);
    /// ```
    ///
    pub fn set_message_callback<F>(&mut self, cb: F)
        where F: Fn(&[u8]) -> () + Send + Sync + 'static
    {
        self.message_cb = Some(Box::new(cb));
    }

    /// 将消息发送至 WASM 模块
    ///
    /// ## 使用示例
    ///
    /// ```ignore
    /// use low_level::host::LowLevelCtx;
    ///
    /// let ctx = LowLevelCtx::new(...);
    /// // 绑定 wasmtime 实例 ...
    /// ctx.send_message_to_wasm("hello wasm".as_bytes()).unwrap();
    /// ```
    ///
    pub fn send_message_to_wasm(&self, msg: &[u8]) -> Result<()> {
        // 如果有 oneshot_caller，说明是嵌套调用，则直接使用
        let oneshot_caller = self.oneshot_caller.lock().unwrap();
        let caller = oneshot_caller.replace(None);
        drop(oneshot_caller);
        if let Some(caller) = caller {
            return self.send_message_to_wasm_with_store(caller, msg);
        }

        // 没有的话看看是否持有 Store
        let temp_store = self.temp_store.lock().unwrap();
        let mut store = temp_store.replace(None).ok_or("No available store!")?;

        let result = self.send_message_to_wasm_with_store(store.as_context_mut(), msg);

        // 放回 store
        temp_store.replace(Some(store));

        return result;
    }

    pub fn send_message_to_wasm_with_store(&self, mut store: impl AsContextMut, msg: &[u8]) -> Result<()> {
        let mut instance_ctx = self.instance_ctx.lock().unwrap();
        let instance_ctx = instance_ctx.get_mut().as_ref().unwrap();
        let func_canonical_abi_free = &instance_ctx.canonical_abi_free;
        let func_canonical_abi_realloc = &instance_ctx.canonical_abi_realloc;
        let func_host_message_handler = &instance_ctx.host_message_handler;

        // 在 WASM 内分配消息内存
        let msg_len = msg.len() as i32;
        let msg_ptr = func_canonical_abi_realloc
            .call(&mut store, (0, 0, 1, msg_len))?;

        // 将消息复制到 WASM 内存中
        let memory = &instance_ctx.memory;
        store_many(memory.data_mut(&mut store), msg_ptr, msg)?;

        // 发送消息
        func_host_message_handler.call(&mut store, (msg_ptr, msg_len))?;

        // 释放消息内存
        func_canonical_abi_free.call(&mut store, (msg_ptr, msg_len, 1))?;

        Ok(())
    }
}

/// 本函数是开源项目 bytecodealliance/wit-bindgen 的一部分，遵照 Apache License 协议引入
fn get_memory<T>(caller: &mut Caller<'_, T>, mem: &str) -> Result<Memory> {
    let mem = caller
        .get_export(mem)
        .ok_or_else(|| {
            let msg = format!("`{}` export not available", mem);
            Trap::new(msg)
        })?
        .into_memory()
        .ok_or_else(|| {
            let msg = format!("`{}` export not a memory", mem);
            Trap::new(msg)
        })?;
    Ok(mem)
}

/// 本函数是开源项目 bytecodealliance/wit-bindgen 的一部分，遵照 Apache License 协议引入
fn store_many(this: &mut [u8], offset: i32, val: &[u8]) -> Result<()> {
    let mem = this
        .get_mut(offset as usize..)
        .and_then(|m| {
            let len = mem::size_of::<u8>().checked_mul(val.len())?;
            m.get_mut(..len)
        })
        .ok_or_else(|| Trap::new("out of bounds write"))?;
    mem.copy_from_slice(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::*;

    #[test]
    fn test_send_message_to_wasm() {
        let Context { mut store, module, mut linker } = guest_prepare();

        // 创建 Ctx
        let ctx = LowLevelCtx::new();
        let ctx = Arc::new(ctx);

        // Linker
        ctx.clone().add_to_linker(&mut linker).unwrap();

        // 实例化
        let instance = linker.instantiate(&mut store, &module).unwrap();
        ctx.attach(&mut store, &instance).unwrap();

        // 发送消息
        let msg = "hello, wasm!".as_bytes();
        ctx.move_store(store);
        ctx.send_message_to_wasm(msg).unwrap();
        let mut store = ctx.take_store().unwrap();

        // 检查结果
        let check = instance
            .get_typed_func::<(), u32, _>(&mut store, "get_receive_check")
            .unwrap();

        assert_eq!(1, check.call(&mut store, ()).unwrap());
    }

    #[test]
    fn test_receive_message_from_wasm() {
        let Context { mut store, module, mut linker } = guest_prepare();

        // 创建 Ctx
        let mut ctx = LowLevelCtx::new();
        ctx.set_message_callback(|msg| {
            println!("接收到 WASM 消息：{:?}", msg);
            assert_eq!("hello, host!".as_bytes(), msg);
        });
        let ctx = Arc::new(ctx);

        // Linker
        ctx.clone().add_to_linker(&mut linker).unwrap();

        // 实例化
        let instance = linker.instantiate(&mut store, &module).unwrap();
        ctx.attach(&mut store, &instance).unwrap();

        // 触发消息发送
        let check = instance
            .get_typed_func::<(), (), _>(&mut store, "test_send_message")
            .unwrap();
        check.call(&mut store, ()).unwrap()
    }
}
