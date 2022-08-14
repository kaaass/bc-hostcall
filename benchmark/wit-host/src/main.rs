use anyhow::{Result};
use std::io::Read;
use std::time::Instant;
use wasmtime::{AsContextMut, Config, Engine, Instance, Linker, Module, Store};

wit_bindgen_wasmtime::export!("../wit-wasm/imports.wit");
wit_bindgen_wasmtime::import!("../wit-wasm/exports.wit");

use imports::*;
use exports::*;

#[derive(Default)]
pub struct MyImports {
}

impl imports::Imports for MyImports {
    fn http_get(&mut self, url: &str) -> String {
        let mut res = reqwest::blocking::get(url).unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
        return body;
    }
}

fn default_config() -> Result<Config> {
    // Create an engine with caching enabled to assist with iteration in this
    // project.
    let mut config = Config::new();
    config.cache_config_load_default()?;
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    Ok(config)
}

fn default_wasi() -> wasmtime_wasi::WasiCtx {
    wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .build()
}

struct Context<I, E> {
    wasi: wasmtime_wasi::WasiCtx,
    imports: I,
    exports: E,
}

fn instantiate<I: Default, E: Default, T>(
    wasm: &str,
    add_imports: impl FnOnce(&mut Linker<Context<I, E>>) -> Result<()>,
    mk_exports: impl FnOnce(
        &mut Store<Context<I, E>>,
        &Module,
        &mut Linker<Context<I, E>>,
    ) -> Result<(T, Instance)>,
) -> Result<(T, Store<Context<I, E>>)> {
    let engine = Engine::new(&default_config()?)?;
    let module = Module::from_file(&engine, wasm)?;

    let mut linker = Linker::new(&engine);
    add_imports(&mut linker)?;
    wasmtime_wasi::add_to_linker(&mut linker, |cx| &mut cx.wasi)?;

    let mut store = Store::new(
        &engine,
        Context {
            wasi: default_wasi(),
            imports: I::default(),
            exports: E::default(),
        },
    );
    let (exports, _instance) = mk_exports(&mut store, &module, &mut linker)?;
    Ok((exports, store))
}

fn main() {
    let urls = vec![
        "http://127.0.0.1/~kaaass/512B.txt",
        "http://127.0.0.1/~kaaass/1K.txt",
        "http://127.0.0.1/~kaaass/5K.txt",
        "http://127.0.0.1/~kaaass/10K.txt",
        "http://127.0.0.1/~kaaass/100K.txt",
        "http://127.0.0.1/~kaaass/1M.txt",
        "http://127.0.0.1/~kaaass/10M.txt",
    ];
    let runs = 5;
    let total = 100;

    let (exports, mut store) = instantiate(
        "./benchmark/wit-wasm/wit-wasm.wasm",
        |linker| imports::add_to_linker(linker, |cx| -> &mut MyImports { &mut cx.imports }),
        |store, module, linker| {
            exports::Exports::instantiate(store, module, linker, |cx| &mut cx.exports)
        },
    ).unwrap();

    for url in urls {
        let mut total_time = 0;
        for _ in 0..runs {
            let start = Instant::now();
            for _ in 0..total {
                let result = exports.do_service(store.as_context_mut(), url).unwrap();
                assert!(result.len() >= 512);
            }
            total_time += start.elapsed().as_millis();
        }
        println!("Test {}: {}ms", url, total_time / runs);
    }
}
