pub mod ctx;
pub mod future;

// FIXME: 此处的错误类型仅仅是最简单，可用于容纳任何错误的类型。而实际上好的错误类型
//        应该囊括更加细节的错误信息。此处仅为适应短时间的开发需求而临时设计。
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;


#[cfg(test)]
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
                                       "./tests/unittest-future/unittest-future.wasm").unwrap();

        Context { store, module, linker }
    }
}

