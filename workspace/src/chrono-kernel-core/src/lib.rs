// src/lib.rs - The Real Rust Entry Point
#![no_std]
#![no_main]

// 비서의 잔소리 끄기
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// 족보 가져오기
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;

// 모듈 가져오기
pub mod i18n;
pub mod hook;

// ═══════════════════════════════════════════════════════════════════════════
// 🔥 [핵심 수정] 함수 이름을 shim.c가 찾는 이름이랑 똑같이 맞춤!
// ═══════════════════════════════════════════════════════════════════════════

/// Called from shim.c (my_module_init -> init_hook)
#[no_mangle]
pub unsafe extern "C" fn init_hook() -> i32 {
    // Banner Output
    _printk(c"\n".as_ptr());
    _printk(c"[GHOST] ══════════════════════════════════════════════════\n".as_ptr());
    _printk(c"[GHOST] Universal i18n Layer: Ready to Serve\n".as_ptr());
    _printk(c"[GHOST] Mode: Kprobe Injection (Safe Mode)\n".as_ptr());
    _printk(c"[GHOST] ══════════════════════════════════════════════════\n".as_ptr());
    
    // 내부 훅 로직 실행
    if let Err(_e) = hook::init_hook() {
        _printk(c"[GHOST] ❌ Hook installation failed.\n".as_ptr());
        return -1;
    }
    
    _printk(c"[GHOST] ✅ Gatekeeper DEPLOYED. System Secured.\n".as_ptr());
    0 
}

/// Called from shim.c (my_module_exit -> cleanup_hook)
#[no_mangle]
pub unsafe extern "C" fn cleanup_hook() {
    hook::cleanup_hook(); // 훅 제거
    _printk(c"[GHOST] Shutdown Complete. Bye! 👋\n".as_ptr());
}

/// Called from shim.c (my_module_exit -> print_stats)
#[no_mangle]
pub unsafe extern "C" fn print_stats() {
    hook::print_stats();
}