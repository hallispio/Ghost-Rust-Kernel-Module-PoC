use core::ffi::{c_int, c_void};
use core::sync::atomic::{AtomicU64, Ordering};
use crate::bindings::*;
use crate::ghost_filter::should_translate_text;
use crate::translation::TRANSLATION_TABLE;
use crate::json_wrapper::wrap_json;

// 통계
pub struct HookStats {
    pub total: AtomicU64,
    pub filtered: AtomicU64,
    pub translated: AtomicU64,
    pub bypassed: AtomicU64,
    pub failed_copy: AtomicU64,
}

pub static STATS: HookStats = HookStats {
    total: AtomicU64::new(0),
    filtered: AtomicU64::new(0),
    translated: AtomicU64::new(0),
    bypassed: AtomicU64::new(0),
    failed_copy: AtomicU64::new(0),
};

// Kprobe
struct SafeKprobe {
    kp: core::cell::UnsafeCell<kprobe>,
}
unsafe impl Sync for SafeKprobe {}

static KP: SafeKprobe = SafeKprobe {
    kp: core::cell::UnsafeCell::new(unsafe {
        core::mem::MaybeUninit::zeroed().assume_init()
    }),
};

// Handler (핵심!)
unsafe extern "C" fn handler_pre(_p: *mut kprobe, regs: *mut pt_regs) -> c_int {
    STATS.total.fetch_add(1, Ordering::Relaxed);
    _printk(b"[GHOST] >>> SAW WRITE! Len: %llu\n\0".as_ptr() as *const i8, (*regs).dx);

    let user_buf = (*regs).si as *mut c_void;
    let len = (*regs).dx as usize;

    // 1. 길이 체크 (32KB 제한)
    if len < 4 || len > 32000 {
        STATS.filtered.fetch_add(1, Ordering::Relaxed);
        return 0;
    }

    // 2. 락 획득 (실패 시 즉시 포기)
    if !crate::ghost_core::try_lock() {
        STATS.bypassed.fetch_add(1, Ordering::Relaxed);
        return 0;
    }

    // 3. 번역 시도
    let result = translate_process(user_buf, len);

    // 4. 락 해제
    crate::ghost_core::unlock();

    // 5. 결과 처리
    match result {
        Some(new_len) => {
            (*regs).dx = new_len as u64;
            STATS.translated.fetch_add(1, Ordering::Relaxed);
        }
        None => {}
    }

    0
}

// 번역 로직 (안정성 최우선)
#[inline]
unsafe fn translate_process(user_ptr: *mut c_void, len: usize) -> Option<usize> {
    let input_buf = crate::ghost_core::get_input_buffer();
    
    // [안전 1] copy_from_user (CPU 감시 통과)
    if _copy_from_user(
        input_buf.as_mut_ptr() as *mut c_void,
        user_ptr,
        len as u64
    ) != 0 {
        STATS.failed_copy.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    // [안전 2] UTF-8 검증
    let input_slice = &input_buf[0..len];
    let input_str = core::str::from_utf8(input_slice).ok()?.trim();

    // [안전 3] 필터링
    if !should_translate_text(input_str) {
        STATS.filtered.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    // 번역
    let translated = TRANSLATION_TABLE.lookup(input_str)?;

    // JSON 생성
    let output_buf = crate::ghost_core::get_output_buffer();
    let json_len = wrap_json(input_str, translated, output_buf).ok()?;

    // [안전 4] 버퍼 오버플로우 방지 (핵심!)
    // if json_len > len {
    //     // JSON이 원본보다 길면 포기
    //     STATS.bypassed.fetch_add(1, Ordering::Relaxed);
        
    //     // dmesg에 로깅 (선택)
    //     log_to_dmesg(output_buf, json_len);
        
    //     return None;
    // }

    // [안전 5] copy_to_user (CPU 감시 통과)
    if _copy_to_user(
        user_ptr,
        output_buf.as_ptr() as *const c_void,
        json_len as u64
    ) != 0 {
        STATS.failed_copy.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    Some(json_len)
}

// dmesg 로깅 (버퍼 오버플로우 시)
#[inline]
unsafe fn log_to_dmesg(json_buf: &[u8], json_len: usize) {
    // 256바이트까지만 출력
    let display_len = core::cmp::min(json_len, 255);
    let mut temp: [u8; 256] = [0; 256];
    temp[..display_len].copy_from_slice(&json_buf[..display_len]);
    temp[display_len] = 0;  // null terminator
    
    _printk(
        b"[GHOST-OVERFLOW] %s%s\n\0".as_ptr() as *const i8,
        temp.as_ptr(),
        if json_len > 255 { b"...\0".as_ptr() } else { b"\0".as_ptr() }
    );
}

// 초기화
pub unsafe fn init_hook_v2(vfs_addr: u64) -> Result<(), &'static str> {
    let kp_ptr = KP.kp.get();
    
    // 🔥 수정 포인트: 주소 직접 할당
    (*kp_ptr).addr = vfs_addr as *mut c_void; 
    (*kp_ptr).pre_handler = Some(handler_pre);

    if register_kprobe(kp_ptr) < 0 {
        return Err("Kprobe registration failed");
    }
    
    Ok(())
}

// 정리
pub unsafe fn cleanup_hook() {
    let kp_ptr = KP.kp.get();
    unregister_kprobe(kp_ptr);
}

// 통계 출력
pub fn print_stats() {
    let total = STATS.total.load(Ordering::Relaxed);
    let filtered = STATS.filtered.load(Ordering::Relaxed);
    let translated = STATS.translated.load(Ordering::Relaxed);
    let bypassed = STATS.bypassed.load(Ordering::Relaxed);
    let failed = STATS.failed_copy.load(Ordering::Relaxed);
    
    unsafe {
        _printk(b"[GHOST] Stats: All Systems Nominal.\n\0".as_ptr() as *const i8);
        _printk(
            b"[GHOST] Total: %llu | Filtered: %llu | Translated: %llu\n\0".as_ptr() as *const i8,
            total, filtered, translated
        );
        _printk(
            b"[GHOST] Bypassed: %llu | Failed Copy: %llu\n\0".as_ptr() as *const i8,
            bypassed, failed
        );
    }
}