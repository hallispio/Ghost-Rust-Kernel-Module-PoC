#![no_std]
#![no_main]
// 💡 비서의 잔소리(163개 경고)를 완전히 잠재웁니다.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::panic::PanicInfo;

// 1. 커널 보물지도 합체
pub mod kernel {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// 2. 고스트쉘 시동 함수
#[no_mangle]
pub extern "C" fn ghost_shell_init() -> i32 {
    unsafe {
        // 💡 수사 결과: 진짜 이름은 '_printk'였습니다! 
        kernel::_printk(
            b"\x016[Ghost Shell] Universal-i18n Layer: Ready!\n\0".as_ptr() as *const _
        );
    }
    0
}

// 3. 패닉 핸들러 (커널 필수 사양)
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}