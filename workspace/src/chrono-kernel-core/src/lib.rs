#![no_std]
#![no_main]

use core::panic::PanicInfo;

// 1. [수정] 커널 직접 호출 대신 shim.c의 wrapper를 가져옴
extern "C" {
    fn ghost_printk(fmt: *const u8);
}

// 2. 패닉 발생 시
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        ghost_printk(b"[GHOST] RUST PANIC! System Halting...\n\0".as_ptr());
    }
    loop {}
}

// 3. 초기화 함수 (C Wrapper가 호출)
#[no_mangle]
pub unsafe extern "C" fn init_hook() -> i32 {
    ghost_printk(b"\n\0".as_ptr());
    ghost_printk(b"[GHOST] ==========================================\n\0".as_ptr());
    ghost_printk(b"[GHOST] Universal i18n Layer: Ready to Serve\n\0".as_ptr());
    ghost_printk(b"[GHOST] Mode: Safe Mode (No-SSE)\n\0".as_ptr());
    ghost_printk(b"[GHOST] ==========================================\n\0".as_ptr());
    0
}

// 4. 종료 함수
#[no_mangle]
pub unsafe extern "C" fn cleanup_hook() {
    unsafe {
        ghost_printk(b"[GHOST] Shutdown Complete. Bye!\n\0".as_ptr());
    }
}

// 5. 상태 출력 함수
#[no_mangle]
pub unsafe extern "C" fn print_stats() {
    unsafe {
        ghost_printk(b"[GHOST] Stats: All Systems Nominal.\n\0".as_ptr());
    }
}