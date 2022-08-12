use std::io::Write;
use std::sync::Arc;
use tokio::task;
use bc_hostcall::module_api::manager::ModuleManager;
use bc_hostcall::module_api::module::WasmModule;
use bc_hostcall::rpc::abi;
use crate::exports::init_exports;
use crate::imports::{app};

mod imports;
mod exports;

fn usage() {
    println!("bc-hostcall CLI Demo");
    println!("load <*.wasm>            加载/重载 Bc Module");
    println!("list                     列出已加载模块");
    println!("call_app <name> <param>  调用模块导出函数 `app`");
    println!("unload <name>            卸载模块");
    println!("help                     显示此信息");
    println!("exit                     退出");
    println!();
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

struct CliContext {
    modules: Arc<ModuleManager>,
}

/// 加载/重载 Bc Module
async fn command_load(ctx: &mut CliContext, path: &str) -> Result<()> {
    let mut module = WasmModule::new();

    // 初始化模块
    module.init(path, init_exports())?;
    println!("[Host] 初始化模块：{}", module.get_name());

    // 启动模块
    module.start().await;
    println!("[Host] 成功启动模块：{}", module.get_name());

    // 添加到模块管理器
    let module = Arc::new(module);
    let swap_out =
        ctx.modules.register(module.get_hint(), module.clone());
    module.attach_to_manager(ctx.modules.clone());

    // 如果发现老模块，则卸载
    if let Some(old_module) = swap_out {
        println!("[Host] 卸载已存在的旧模块：{}", old_module.get_name());
        old_module.kill();
    }

    Ok(())
}

/// 列出已加载模块
async fn command_list(ctx: &mut CliContext) -> Result<()> {
    for hint in ctx.modules.list_modules() {
        println!("- {:?}", hint);
    }
    Ok(())
}

/// 调用模块导出函数 `app`
async fn command_call_app(ctx: &mut CliContext, name: &str, param: String) -> Result<()> {
    let hint = abi::LinkHint::BcModule(name.to_string());

    // 寻找模块
    let module = ctx.modules.resolve(&hint);
    let module = if let Some(module) = module {
        module
    } else {
        println!("[Host] 模块不存在：{}", name);
        return Ok(());
    };

    // 调用 `app` 函数
    let result = app(module.as_ref(), param).await?;
    println!("[Host] 调用结果：{}", result);

    Ok(())
}

/// 卸载模块
async fn command_unload(ctx: &mut CliContext, name: &str) -> Result<()> {
    // 寻找模块
    let hint = abi::LinkHint::BcModule(name.to_string());
    let module = ctx.modules.unregister(&hint);
    let module = if let Some(module) = module {
        module
    } else {
        println!("[Host] 模块不存在：{}", name);
        return Ok(());
    };

    // 卸载
    module.kill();
    println!("[Host] 成功卸载模块：{}", module.get_name());

    Ok(())
}

async fn handle_input(ctx: &mut CliContext, input: String) -> Result<()> {
    let parts = input.split_whitespace().collect::<Vec<&str>>();

    // 空行
    if parts.len() == 0 {
        return Ok(());
    }

    // 拆分命令和参数
    let cmd = parts[0];
    let rest = if parts.len() > 1 {
        parts[1..].join(" ")
    } else {
        "".to_string()
    };

    if cmd == "load" {
        command_load(ctx, &rest).await?;
    } else if cmd == "list" {
        command_list(ctx).await?;
    } else if cmd == "call_app" {
        if parts.len() < 3 {
            println!("[Host] 参数错误：call_app <name> <param>");
            return Ok(());
        }
        command_call_app(ctx, parts[1], parts[2].to_string()).await?;
    } else if cmd == "unload" {
        // 卸载模块
        command_unload(ctx, &rest).await?;
    } else if cmd == "help" {
        usage();
    } else if cmd == "exit" {
        println!("Bye!");
        std::process::exit(0);
    } else {
        println!("[Host] 未知指令: {}", cmd);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    usage();

    let mut ctx = CliContext {
        modules: Arc::new(ModuleManager::new()),
    };

    loop {
        let line = task::spawn_blocking(move || {
            let mut line = String::new();
            std::io::stdout().write(">> ".as_bytes()).unwrap();
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut line).unwrap();
            line
        }).await.unwrap();

        // println!("[Host] Input: {}", line.trim());
        let ret = handle_input(&mut ctx, line.trim().to_string()).await;
        if let Err(e) = ret {
            println!("[Host] 运行指令失败: {}", line.trim());
            println!("[Host] Error: {}", e);
        }

        println!();
    }
}
