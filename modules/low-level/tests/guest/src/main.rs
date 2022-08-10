//! 用于作为 low-level 模块单元测试的 wasm 模块

use low_level::set_message_callback;
use low_level::wasm;

/// 申请包含特定内容的内存块，用于对 copy_from_wasm 函数的测试
#[no_mangle]
pub extern "C" fn alloc_signal_buffer() -> u32 {
    let mut buf = vec![0x12u8, 0x34, 0x56, 0x78, 0xde, 0xed, 0xbe, 0xef];
    let ptr = buf.as_mut_ptr() as *mut u8;
    std::mem::forget(buf);
    ptr as u32
}

/// 对指定内存地址的内容进行检查，用于对 copy_to_wasm 函数的测试
#[no_mangle]
pub extern "C" fn check_signal_at(ptr: *mut u8) -> bool {
    let actual = unsafe { Vec::from_raw_parts(ptr, 8, 8) };
    let expected = vec![0x43u8, 0x21, 0x67, 0x89, 0xbe, 0xef, 0xde, 0xed];
    actual == expected
}

/// 发送指定消息至 host，用于对 `low_level::wasm::send_message_to_host` 进行测试
#[no_mangle]
pub extern "C" fn test_send_message() {
    println!("发送消息至 Host");
    wasm::send_message_to_host("hello, host!".as_bytes()).unwrap();
}

static mut RECV_CHECK: u32 = 0;

/// 设置接收 host 消息的回调函数，用于对 `low_level::wasm::set_message_callback!` 进行测试
fn receive_message_from_host(msg: &[u8]) {
    println!("接收到 Host 消息：{:?}", msg);

    let expected = "hello, wasm!".as_bytes();
    unsafe {
        RECV_CHECK = if msg == expected { 1 } else { 0 };
    }
}

set_message_callback!(receive_message_from_host);

#[no_mangle]
pub extern "C" fn get_receive_check() -> u32 {
    unsafe { RECV_CHECK }
}

fn main() {}
