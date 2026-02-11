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

// 🔥 [추가] 텔레메트리
pub struct Telemetry {
    pub size_tiny: AtomicU64,
    pub size_small: AtomicU64,
    pub size_medium: AtomicU64,
    pub size_large: AtomicU64,
    pub size_huge: AtomicU64,
    
    pub has_error: AtomicU64,
    pub has_warning: AtomicU64,
    pub has_json: AtomicU64,
    pub has_unknown: AtomicU64,
    
    pub danger_kernel_code: AtomicU64,
    pub danger_null_ptr: AtomicU64,
    pub danger_kernel_space: AtomicU64,
}

pub static TELEMETRY: Telemetry = Telemetry {
    size_tiny: AtomicU64::new(0),
    size_small: AtomicU64::new(0),
    size_medium: AtomicU64::new(0),
    size_large: AtomicU64::new(0),
    size_huge: AtomicU64::new(0),
    has_error: AtomicU64::new(0),
    has_warning: AtomicU64::new(0),
    has_json: AtomicU64::new(0),
    has_unknown: AtomicU64::new(0),
    danger_kernel_code: AtomicU64::new(0),
    danger_null_ptr: AtomicU64::new(0),
    danger_kernel_space: AtomicU64::new(0),
};

// 🔥 [추가] 위험 지역 자동 설정
static KERNEL_TEXT_START: AtomicU64 = AtomicU64::new(0);
static KERNEL_TEXT_END: AtomicU64 = AtomicU64::new(0);
static KERNEL_DATA_START: AtomicU64 = AtomicU64::new(0);
static KERNEL_DATA_END: AtomicU64 = AtomicU64::new(0);

// 🔥 [추가] current 포인터 (커널에서 제공)
extern "C" {
    fn get_current_task() -> *const task_struct;
    fn ghost_printk_comm(prefix: *const u8, comm: *const u8);
    fn ghost_printk_danger(danger_type: *const u8, comm: *const u8, ptr: u64);
}

// 🔥 [추가] task_struct (comm 위치)
#[repr(C)]
pub struct task_struct {
    _padding: [u8; 1216], // comm 오프셋 (x86_64 커널 6.x)
    pub comm: [u8; 16],
}

// 🔥 [추가] 위험 지역 설정
pub unsafe fn set_danger_zones(
    text_start: u64,
    text_end: u64,
    data_start: u64,
    data_end: u64
) {
    KERNEL_TEXT_START.store(text_start, Ordering::Relaxed);
    KERNEL_TEXT_END.store(text_end, Ordering::Relaxed);
    KERNEL_DATA_START.store(data_start, Ordering::Relaxed);
    KERNEL_DATA_END.store(data_end, Ordering::Relaxed);
}

// 🔥 [추가] 위험 지역 체크 with comm
#[inline]
unsafe fn check_danger_zone_with_comm(ptr: u64, comm: *const u8) -> bool {
    let text_start = KERNEL_TEXT_START.load(Ordering::Relaxed);
    let text_end = KERNEL_TEXT_END.load(Ordering::Relaxed);
    let data_start = KERNEL_DATA_START.load(Ordering::Relaxed);
    let data_end = KERNEL_DATA_END.load(Ordering::Relaxed);
    
    if ptr == 0 {
        TELEMETRY.danger_null_ptr.fetch_add(1, Ordering::Relaxed);
        ghost_printk_danger(b"NULL pointer\0".as_ptr(), comm, ptr);
        return false;
    }
    
    if text_start > 0 && ptr >= text_start && ptr <= text_end {
        TELEMETRY.danger_kernel_code.fetch_add(1, Ordering::Relaxed);
        ghost_printk_danger(b"Kernel code zone\0".as_ptr(), comm, ptr);
        return false;
    }
    
    if data_start > 0 && ptr >= data_start && ptr <= data_end {
        TELEMETRY.danger_kernel_code.fetch_add(1, Ordering::Relaxed);
        ghost_printk_danger(b"Kernel data zone\0".as_ptr(), comm, ptr);
        return false;
    }
    
    if ptr >= 0xffff800000000000 {
        TELEMETRY.danger_kernel_space.fetch_add(1, Ordering::Relaxed);
        ghost_printk_danger(b"Kernel space\0".as_ptr(), comm, ptr);
        return false;
    }
    
    true
}

// 🔥 [추가] 텔레메트리 기록
#[inline]
unsafe fn record_telemetry(data: &[u8], len: usize) {
    match len {
        0..=15 => TELEMETRY.size_tiny.fetch_add(1, Ordering::Relaxed),
        16..=255 => TELEMETRY.size_small.fetch_add(1, Ordering::Relaxed),
        256..=4095 => TELEMETRY.size_medium.fetch_add(1, Ordering::Relaxed),
        4096..=32767 => TELEMETRY.size_large.fetch_add(1, Ordering::Relaxed),
        _ => TELEMETRY.size_huge.fetch_add(1, Ordering::Relaxed),
    };
    
    if data.windows(5).any(|w| w == b"Error" || w == b"error") {
        TELEMETRY.has_error.fetch_add(1, Ordering::Relaxed);
    } else if data.windows(7).any(|w| w == b"Warning" || w == b"warning") {
        TELEMETRY.has_warning.fetch_add(1, Ordering::Relaxed);
    } else if data.starts_with(b"{") {
        TELEMETRY.has_json.fetch_add(1, Ordering::Relaxed);
    } else {
        TELEMETRY.has_unknown.fetch_add(1, Ordering::Relaxed);
    }
}

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

// 🔥 [추가] VFS Kprobe
struct SafeKprobeVfs {
    kp: core::cell::UnsafeCell<kprobe>,
}
unsafe impl Sync for SafeKprobeVfs {}

static KP_VFS: SafeKprobeVfs = SafeKprobeVfs {
    kp: core::cell::UnsafeCell::new(unsafe {
        core::mem::MaybeUninit::zeroed().assume_init()
    }),
};

// Handler (핵심!)
unsafe extern "C" fn handler_pre(_p: *mut kprobe, regs: *mut pt_regs) -> c_int {
    STATS.total.fetch_add(1, Ordering::Relaxed);
    
    // 🔥 [추가] 누가 호출했는지 확인
    let task = &*get_current_task();
    let comm_ptr = task.comm.as_ptr();
    let user_buf_ptr = (*regs).si as u64;
    if user_buf_ptr > 0x00007fffffffffff || user_buf_ptr == 0 { return 0; }
    ghost_printk_comm(b"HANDLER CALLED\0".as_ptr(), comm_ptr);
    
    let user_buf_ptr = (*regs).si as u64;
    let len = (*regs).dx as usize;
    
    // 🔥 [추가] 위험 지역 체크
    if !check_danger_zone_with_comm(user_buf_ptr, comm_ptr) {
        return 0;
    }
    
    _printk(b"[GHOST] >>> SAW WRITE! Len: %llu\n\0".as_ptr() as *const i8, (*regs).dx);

    let user_buf = user_buf_ptr as *mut c_void;

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

    // 🔥 [추가] 텔레메트리 기록
    if result.is_some() {
        let input_buf = crate::ghost_core::get_input_buffer();
        let input_slice = &input_buf[0..len];
        record_telemetry(input_slice, len);
    }

    // 5. 결과 처리
    match result {
        Some(new_len) => {
            (*regs).dx = new_len as u64;
            STATS.translated.fetch_add(1, Ordering::Relaxed);
            
            // 🔥 [추가] 성공 로그
            ghost_printk_comm(b"SUCCESS Trapped\0".as_ptr(), comm_ptr);
        }
        None => {}
    }

    0
}

// 🔥 [추가] VFS 핸들러
unsafe extern "C" fn handler_vfs(_p: *mut kprobe, regs: *mut pt_regs) -> c_int {
    let task = &*get_current_task();
    let comm_ptr = task.comm.as_ptr();
    let user_buf_ptr = (*regs).si as u64;
    if user_buf_ptr > 0x00007fffffffffff || user_buf_ptr == 0 { return 0; }
    
    ghost_printk_comm(b"VFS WRITE\0".as_ptr(), comm_ptr);
    
    let user_buf_ptr = (*regs).si as u64;
    let len = (*regs).dx as usize;
    
    if !check_danger_zone_with_comm(user_buf_ptr, comm_ptr) {
        return 0;
    }
    
    if len < 4 || len > 32000 {
        return 0;
    }

    if !crate::ghost_core::try_lock() {
        return 0;
    }

    let user_buf = user_buf_ptr as *mut c_void;
    let result = translate_process(user_buf, len);
    crate::ghost_core::unlock();

    if result.is_some() {
        let input_buf = crate::ghost_core::get_input_buffer();
        let input_slice = &input_buf[0..len];
        record_telemetry(input_slice, len);
    }

    match result {
        Some(new_len) => {
            (*regs).dx = new_len as u64;
        }
        None => {}
    }

    0
}

// 번역 로직 (안정성 최우선)
#[inline]
unsafe fn translate_process(user_ptr: *mut c_void, len: usize) -> Option<usize> {
    let input_buf = crate::ghost_core::get_input_buffer();

    let addr = user_ptr as u64;
    if addr > 0x00007fffffffffff || addr == 0 { return None; }
    
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
    
    (*kp_ptr).addr = vfs_addr as *mut c_void; 
    (*kp_ptr).pre_handler = Some(handler_pre);

    if register_kprobe(kp_ptr) < 0 {
        return Err("Kprobe registration failed");
    }
    
    Ok(())
}

// 🔥 [추가] VFS 훅 초기화
pub unsafe fn init_vfs_hook(vfs_addr: u64) -> Result<(), &'static str> {
    let kp_ptr = KP_VFS.kp.get();
    
    (*kp_ptr).addr = vfs_addr as *mut c_void;
    (*kp_ptr).pre_handler = Some(handler_vfs);
    
    if register_kprobe(kp_ptr) < 0 {
        return Err("VFS Kprobe registration failed");
    }
    
    Ok(())
}

// 정리
pub unsafe fn cleanup_hook() {
    let kp_ptr = KP.kp.get();
    unregister_kprobe(kp_ptr);
}

// 🔥 [추가] VFS 정리
pub unsafe fn cleanup_vfs_hook() {
    let kp_ptr = KP_VFS.kp.get();
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
        _printk(b"\n[GHOST] ========== Statistics ==========\n\0".as_ptr() as *const i8);
        _printk(b"[GHOST] Stats: All Systems Nominal.\n\0".as_ptr() as *const i8);
        _printk(
            b"[GHOST] Total: %llu | Filtered: %llu | Translated: %llu\n\0".as_ptr() as *const i8,
            total, filtered, translated
        );
        _printk(
            b"[GHOST] Bypassed: %llu | Failed Copy: %llu\n\0".as_ptr() as *const i8,
            bypassed, failed
        );
        
        // 🔥 [추가] 텔레메트리
        _printk(b"\n[GHOST] ===== Telemetry =====\n\0".as_ptr() as *const i8);
        _printk(
            b"[GHOST] Size: Tiny=%llu Small=%llu Medium=%llu Large=%llu Huge=%llu\n\0".as_ptr() as *const i8,
            TELEMETRY.size_tiny.load(Ordering::Relaxed),
            TELEMETRY.size_small.load(Ordering::Relaxed),
            TELEMETRY.size_medium.load(Ordering::Relaxed),
            TELEMETRY.size_large.load(Ordering::Relaxed),
            TELEMETRY.size_huge.load(Ordering::Relaxed),
        );
        _printk(
            b"[GHOST] Pattern: Error=%llu Warning=%llu JSON=%llu Unknown=%llu\n\0".as_ptr() as *const i8,
            TELEMETRY.has_error.load(Ordering::Relaxed),
            TELEMETRY.has_warning.load(Ordering::Relaxed),
            TELEMETRY.has_json.load(Ordering::Relaxed),
            TELEMETRY.has_unknown.load(Ordering::Relaxed),
        );
        
        // 🔥 [추가] 위험 지역
        _printk(b"\n[GHOST] ===== Danger Zones =====\n\0".as_ptr() as *const i8);
        _printk(
            b"[GHOST] KernelCode=%llu NullPtr=%llu KernelSpace=%llu\n\0".as_ptr() as *const i8,
            TELEMETRY.danger_kernel_code.load(Ordering::Relaxed),
            TELEMETRY.danger_null_ptr.load(Ordering::Relaxed),
            TELEMETRY.danger_kernel_space.load(Ordering::Relaxed),
        );
        
        _printk(b"[GHOST] ==================================\n\n\0".as_ptr() as *const i8);
    }
}