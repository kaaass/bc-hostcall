
mod exports;
mod imports;

use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::time::Instant;
use bc_hostcall::module_api::module::WasmModule;
use exports::*;
use imports::*;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

async fn prepare_module(wasm: &str) -> WasmModule {
    // 加载模块
    let mut module = WasmModule::new();
    module.init(wasm, init_exports()).unwrap();

    // 启动
    module.start().await;

    module
}

async fn benchmark_once(module: Arc<WasmModule>, url: &str, total: i32) -> u128 {
    let cnt = Arc::new(AtomicI32::new(0));

    let start = Instant::now();
    for _ in 0..total {
        let module = Arc::clone(&module);
        let cnt = Arc::clone(&cnt);
        let url = url.to_string();
        tokio::spawn(async move {
            let result = do_service(module.as_ref(), url).await.unwrap();
            assert!(result.len() >= 512);
            cnt.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        });
    }
    while cnt.load(std::sync::atomic::Ordering::Relaxed) < total {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }

    return start.elapsed().as_millis();
}

#[tokio::main]
async fn main() {
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

    let module = prepare_module("./benchmark/wasm/wasm.wasm").await;
    let module = Arc::new(module);

    for url in urls {
        let mut total_time = 0;
        for _ in 0..runs {
            let time = benchmark_once(module.clone(), url, total).await;
            total_time += time;
        }
        println!("Test {}: {}ms", url, total_time / runs);
    }
}
