#![no_std] //禁用Rust标准库
#![no_main] //禁用所有 Rust 层级的入口点

mod vga_buffer;
use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {
    /*let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    // 因为链接器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
    // 默认命名为 `_start`
    */
    println!("Hello Joakim");
    loop {}
}

// 这个函数将在 panic 时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}