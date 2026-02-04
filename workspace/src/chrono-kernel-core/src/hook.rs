// VFS Write Hook - The "Pure" Edition (Using Auto-Bindings)
// Author: Bureum Lee
// License: MIT
#![no_std]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::mem::MaybeUninit;
use core::ffi::{c_int, c_char};

// ✅ [핵심] 수동 정의 다 갖다 버리고, 이제 자동 족보를 믿습니다!
use crate::bindings::{kprobe, pt_regs, register_kprobe, unregister_kprobe, _printk};

use crate::i18n::translate_bytes;

// ═══════════════════════════════════════════════════════════════════════════
// 설정 및 전역 변수
// ═══════════════════════════════════════════════════════════════════════════
const MAX_TRANSLATE_LEN: usize = 512;

static mut GHOST_BUF: [u8; MAX_TRANSLATE_LEN] = [0; MAX_TRANSLATE_LEN];
static IN_HOOK: AtomicBool = AtomicBool::new(false);

pub struct HookStats {
    pub total_calls: AtomicU64,
    pub translated: AtomicU64,
}
pub static HOOK_STATS: HookStats = HookStats {
    total_calls: AtomicU64::new(0),
    translated: AtomicU64::new(0),
};

static mut KP: MaybeUninit<kprobe> = MaybeUninit::uninit();

// ═══════════════════════════════════════════════════════════════════════════
// 훅 핸들러 (순정 족보 사용)
// ═══════════════════════════════════════════════════════════════════════════

unsafe extern "C" fn handler_pre(_p: *mut kprobe, regs: *mut pt_regs) -> c_int {
    if IN_HOOK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        return 0;
    }

    // ✅ 이제 bindings.rs가 정상적으로 만들어졌다면, 
    // (*regs).di, (*regs).si 처럼 바로 접근 가능합니다!
    let fd = (*regs).di;
    let buf_ptr = (*regs).si;
    let len = (*regs).dx;

    HOOK_STATS.total_calls.fetch_add(1, Ordering::Relaxed);

    if fd > 2 || len == 0 || buf_ptr == 0 {
        IN_HOOK.store(false, Ordering::Release);
        return 0;
    }

    let buf = buf_ptr as *const u8;
    let len_usize = len as usize;

    if let Some(translated) = translate_bytes(buf, len_usize) {
        let trans_bytes = translated.as_bytes();
        let trans_len = trans_bytes.len();

        if trans_len <= MAX_TRANSLATE_LEN {
            core::ptr::copy_nonoverlapping(
                trans_bytes.as_ptr(), 
                GHOST_BUF.as_mut_ptr(), 
                trans_len
            );

            // [Hijacking]
            (*regs).si = GHOST_BUF.as_ptr() as u64; 
            (*regs).dx = trans_len as u64;
            
            HOOK_STATS.translated.fetch_add(1, Ordering::Relaxed);
        }
    }

    IN_HOOK.store(false, Ordering::Release);
    0
}

// ═══════════════════════════════════════════════════════════════════════════
// 초기화 및 정리
// ═══════════════════════════════════════════════════════════════════════════

pub unsafe fn init_hook() -> Result<(), &'static str> {
    // 메모리 확보
    KP.write(core::mem::MaybeUninit::zeroed().assume_init());
    let kp_ptr = KP.as_mut_ptr();
    
    // 내용 채우기 (C 문자열)
    (*kp_ptr).symbol_name = b"vfs_write\0".as_ptr() as *const c_char;
    (*kp_ptr).pre_handler = Some(handler_pre);

    // ✅ bindings에 있는 register_kprobe 사용
    if register_kprobe(kp_ptr) < 0 {
        return Err("Kprobe 등록 실패");
    }
    Ok(())
}

pub unsafe fn cleanup_hook() {
    let kp_ptr = KP.as_mut_ptr();
    unregister_kprobe(kp_ptr);
}

pub fn print_stats() {
    let total = HOOK_STATS.total_calls.load(Ordering::Relaxed);
    let trans = HOOK_STATS.translated.load(Ordering::Relaxed);
    unsafe {
        _printk(b"[GHOST Stats] Total: %llu | Translated: %llu\n\0".as_ptr() as *const c_char, total, trans);
    }
}