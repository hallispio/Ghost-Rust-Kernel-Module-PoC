// VFS Write Hook - The "Safe" Edition (SMAP bypass & Rust 2024 Compliant)
// Author: Bureum Lee
// License: MIT
#![no_std]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::cell::UnsafeCell;
use core::ffi::{c_int, c_char};
use core::ptr;

// bindings 모듈 가져오기
use crate::bindings::{kprobe, pt_regs, register_kprobe, unregister_kprobe, _printk};
use crate::i18n::translate_bytes;

// ═══════════════════════════════════════════════════════════════════════════
// Rust 2024 호환 안전한 전역 버퍼 (UnsafeCell 패턴)
// ═══════════════════════════════════════════════════════════════════════════
const MAX_TRANSLATE_LEN: usize = 512;

// 전역 변수를 안전하게 쓰기 위한 래퍼 구조체
struct GhostBuffer {
    data: UnsafeCell<[u8; MAX_TRANSLATE_LEN]>,
    in_use: AtomicBool,
}

// 커널 안에서 전역으로 써도 된다고 컴파일러에게 서약서 제출 (Sync 구현)
unsafe impl Sync for GhostBuffer {}

static GHOST_BUF: GhostBuffer = GhostBuffer {
    data: UnsafeCell::new([0; MAX_TRANSLATE_LEN]),
    in_use: AtomicBool::new(false),
};

pub struct HookStats {
    pub total_calls: AtomicU64,
    pub translated: AtomicU64,
}
pub static HOOK_STATS: HookStats = HookStats {
    total_calls: AtomicU64::new(0),
    translated: AtomicU64::new(0),
};

// Kprobe 객체도 안전하게 포장
struct SafeKprobe {
    kp: UnsafeCell<kprobe>,
}
unsafe impl Sync for SafeKprobe {}

static KP: SafeKprobe = SafeKprobe {
    kp: UnsafeCell::new(unsafe { core::mem::MaybeUninit::zeroed().assume_init() }),
};

// ═══════════════════════════════════════════════════════════════════════════
// 훅 핸들러
// ═══════════════════════════════════════════════════════════════════════════

unsafe extern "C" fn handler_pre(_p: *mut kprobe, regs: *mut pt_regs) -> c_int {
    // 1. 락 획득 (SpinLock 시뮬레이션: 이미 누가 쓰고 있으면 그냥 훅 포기)
    // compare_exchange가 실패하면 다른 CPU가 쓰고 있다는 뜻 -> return 0
    if GHOST_BUF.in_use.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        return 0;
    }

    let fd = (*regs).di;     // 1번째 인자: fd
    let user_buf_ptr = (*regs).si as *mut u8; // 2번째 인자: 사용자 버퍼 주소
    let len = (*regs).dx;    // 3번째 인자: 길이

    HOOK_STATS.total_calls.fetch_add(1, Ordering::Relaxed);

    // 필터링: stdout(1), stderr(2)만 타겟, 길이는 적당히, NULL 포인터 체크
    if fd > 2 || len == 0 || user_buf_ptr.is_null() {
        GHOST_BUF.in_use.store(false, Ordering::Release);
        return 0;
    }

    let len_usize = len as usize;
    // 번역할 때는 사용자 버퍼를 읽어야 함 (주의: 페이지 폴트 나면 안됨)
    // 여기서는 단순히 포인터로 읽음. (실제 커널에서는 copy_from_user_nofault 써야 안전함)
    
    // 2. 번역 시도
    if let Some(translated) = translate_bytes(user_buf_ptr, len_usize) {
        let trans_bytes = translated.as_bytes();
        let trans_len = trans_bytes.len();

        // **중요**: 번역된 길이가 원본 길이보다 작거나 같아야 덮어쓰기 가능 (안그러면 오버플로우)
        if trans_len <= MAX_TRANSLATE_LEN && trans_len <= len_usize {
            
            // [핵심 수정] 주소를 바꾸는 게 아니라, 사용자 버퍼에 '직접' 덮어쓴다.
            // 이렇게 하면 vfs_write는 여전히 사용자 주소(regs->si)를 보므로 SMAP 통과!
            ptr::copy_nonoverlapping(
                trans_bytes.as_ptr(),
                user_buf_ptr, // 타겟: 사용자 버퍼 원본 위치
                trans_len
            );

            // 길이 정보 업데이트 (더 짧아졌을 수 있으므로)
            (*regs).dx = trans_len as u64;
            
            HOOK_STATS.translated.fetch_add(1, Ordering::Relaxed);
        }
    }

    // 3. 락 해제
    GHOST_BUF.in_use.store(false, Ordering::Release);
    0
}

// ═══════════════════════════════════════════════════════════════════════════
// 초기화 및 정리
// ═══════════════════════════════════════════════════════════════════════════

pub unsafe fn init_hook() -> Result<(), &'static str> {
    let kp_ptr = KP.kp.get();
    
    // Kprobe 구조체 초기화
    (*kp_ptr).symbol_name = b"vfs_write\0".as_ptr() as *const c_char;
    (*kp_ptr).pre_handler = Some(handler_pre);

    if register_kprobe(kp_ptr) < 0 {
        return Err("Kprobe 등록 실패");
    }
    Ok(())
}

pub unsafe fn cleanup_hook() {
    let kp_ptr = KP.kp.get();
    unregister_kprobe(kp_ptr);
}

pub fn print_stats() {
    let total = HOOK_STATS.total_calls.load(Ordering::Relaxed);
    let trans = HOOK_STATS.translated.load(Ordering::Relaxed);
    unsafe {
        _printk(b"[GHOST Stats] Total: %llu | Translated: %llu\n\0".as_ptr() as *const c_char, total, trans);
    }
}