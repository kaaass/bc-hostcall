//! Future 单元测试用

#![cfg(target_arch = "wasm32")]

use low_level::set_message_callback;

static mut CNT: i32 = 0;

fn message_callback(msg: &[u8]) {
    println!("接收到 Host 消息：{:?}, cnt = {}", msg, unsafe { CNT });
    unsafe {
        CNT += 1;
    }
}

set_message_callback!(message_callback);

#[no_mangle]
pub extern "C" fn __bc_low_level_wasm_poll() {
    println!("__bc_low_level_wasm_poll()");
}

#[no_mangle]
pub extern "C" fn get_cnt() -> i32 {
    unsafe {
        CNT
    }
}

fn main() {
}
