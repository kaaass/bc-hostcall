//! 低层通信模块，负责为上层模块提供一种复杂信息的传送方式

#[cfg(not(target_arch = "wasm32"))]
pub mod host;

pub mod wasm;

// FIXME: 此处的错误类型仅仅是最简单，可用于容纳任何错误的类型。而实际上好的错误类型
//        应该囊括更加细节的错误信息。此处仅为适应短时间的开发需求而临时设计。
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use wasmtime::*;
    use wasmtime_wasi::sync::WasiCtxBuilder;
    use wasmtime_wasi::WasiCtx;

    pub struct Context<T> {
        pub store: Store<T>,
        pub module: Module,
        pub linker: Linker<T>,
    }

    pub fn guest_prepare() -> Context<WasiCtx> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // 链接 WASI 函数
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

        // 创建 WASI 上下文
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        let store = Store::new(&engine, wasi);

        // 创建 Module 并进行实例化
        let module = Module::from_file(store.engine(),
                                       "tests/guest/guest.wasm").unwrap();

        Context { store, module, linker }
    }

    /// 对测试用的 WASM guest 模块的 `alloc_signal_buffer` 进行测试
    /// 若可通过测试，则说明 guest 模块可以正确的将信息通过内存块传递至 host，即相关测试模块有效
    #[test]
    fn check_test_guest() {
        let Context { mut store, module, mut linker } = guest_prepare();
        linker.func_wrap("__bc_low_level", "receive_message_from_wasm",
                         |_: Caller<'_, _>, _: i32, _: i32| {}).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let alloc_signal_buffer =
            instance.get_typed_func::<(), u32, _>(&mut store, "alloc_signal_buffer").unwrap();
        let buffer_off = alloc_signal_buffer.call(&mut store, ()).unwrap() as usize;
        let buffer_len = 8;

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mem_data = memory.data(&store);
        let actual = &mem_data[buffer_off..buffer_off + buffer_len];

        let expected = vec![0x12u8, 0x34, 0x56, 0x78, 0xde, 0xed, 0xbe, 0xef];

        println!("buffer_off: {}", buffer_off);
        assert_eq!(expected.as_slice(), actual);
    }
}
