#![no_std]
#![no_main]
#![no_builtins]

use core::panic::PanicInfo;
use core::ffi::{c_int, c_void}; 
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// ============================================================
// 1. 메모리 함수
// ============================================================
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

// ============================================================
// 2. 외부 함수 (shim.c)
// ============================================================
extern "C" {
    fn ghost_printk(fmt: *const u8);
    fn ghost_copy_from_user(to: *mut c_void, from: *const c_void, n: u64) -> u64;
    fn ghost_copy_to_user(to: *mut c_void, from: *const c_void, n: u64) -> u64;
    fn ghost_register_kprobe(kp: *mut kprobe) -> i32;
    fn ghost_unregister_kprobe(kp: *mut kprobe);
    
    // 🔥 신원 조회 함수 (FD 인자 추가)
    fn ghost_inspect_task(fd: u64);
}

// ============================================================
// 3. 구조체
// ============================================================
#[repr(C)]
pub struct pt_regs {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub bp: u64, pub bx: u64, pub r11: u64, pub r10: u64,
    pub r9: u64, pub r8: u64, pub ax: u64, pub cx: u64,
    pub dx: u64, pub si: u64, pub di: u64, pub orig_ax: u64,
    pub ip: u64, pub cs: u64, pub flags: u64, pub sp: u64, pub ss: u64,
}

#[repr(C)]
pub struct kprobe {
    pub pre_handler: Option<unsafe extern "C" fn(*mut kprobe, *mut pt_regs) -> i32>,
    pub post_handler: Option<unsafe extern "C" fn(*mut kprobe, *mut pt_regs, u64)>,
    pub fault_handler: Option<unsafe extern "C" fn(*mut kprobe, *mut pt_regs, i32) -> i32>,
    pub symbol_name: *const i8,
    pub offset: u64,
    pub addr: *mut c_void,
    pub flags: u32,
    pub nmissed: u64,
}

// ============================================================
// 4. 전역 변수
// ============================================================
const BUFFER_SIZE: usize = 65536;
const HALF_SIZE: usize = 32768;
static mut TRANS_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static BUFFER_LOCK: AtomicBool = AtomicBool::new(false);

// 🔥 텔레메트리 (전수조사용)
pub struct Telemetry {
    pub total_calls: AtomicU64,
    pub size_tiny: AtomicU64,
    pub size_small: AtomicU64,
    pub size_medium: AtomicU64,
    pub size_large: AtomicU64,
    pub hit_error: AtomicU64,
    pub hit_warning: AtomicU64,
    pub failed_copy: AtomicU64,
}

pub static TELEMETRY: Telemetry = Telemetry {
    total_calls: AtomicU64::new(0),
    size_tiny: AtomicU64::new(0),
    size_small: AtomicU64::new(0),
    size_medium: AtomicU64::new(0),
    size_large: AtomicU64::new(0),
    hit_error: AtomicU64::new(0),
    hit_warning: AtomicU64::new(0),
    failed_copy: AtomicU64::new(0),
};

struct SafeKprobe { kp: core::cell::UnsafeCell<kprobe> }
unsafe impl Sync for SafeKprobe {}

static KP: SafeKprobe = SafeKprobe { 
    kp: core::cell::UnsafeCell::new(unsafe { core::mem::MaybeUninit::zeroed().assume_init() }) 
};

// ============================================================
// 5. 핸들러 (전수조사 특화 + FD 추적)
// ============================================================
unsafe extern "C" fn handler_pre(_p: *mut kprobe, regs: *mut pt_regs) -> c_int {
    TELEMETRY.total_calls.fetch_add(1, Ordering::Relaxed);
    
    // 🔥 FD 추출
    let fd = (*regs).di;
    let user_buf_ptr = (*regs).si as u64;
    let len = (*regs).dx as usize;
    
    // 🔥 [수정 1] 주소 체크 완전 삭제! (ghost_copy_from_user가 알아서 검사함)
    
    // 🔥 [수정 2] 길이 체크 대폭 완화 (2~8192)
    if len < 2 || len > 8192 {
        return 0;
    }

    // 🔥 [수정 3] 텔레메트리 기록
    match len {
        0..=15 => TELEMETRY.size_tiny.fetch_add(1, Ordering::Relaxed),
        16..=255 => TELEMETRY.size_small.fetch_add(1, Ordering::Relaxed),
        256..=4095 => TELEMETRY.size_medium.fetch_add(1, Ordering::Relaxed),
        _ => TELEMETRY.size_large.fetch_add(1, Ordering::Relaxed),
    };

    let user_buf = user_buf_ptr as *mut c_void;
    
    // 🔥 [4단계] 락 획득
    if BUFFER_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() { 
        return 0; 
    }

    let input_buf = &mut TRANS_BUFFER[0..HALF_SIZE];
    
    // 🔥 [5단계] 복사 (여기서 안전하게 검사됨)
    if ghost_copy_from_user(input_buf.as_mut_ptr() as *mut c_void, user_buf, len as u64) != 0 {
        TELEMETRY.failed_copy.fetch_add(1, Ordering::Relaxed);
        BUFFER_LOCK.store(false, Ordering::Release);
        return 0;
    }

    let input_slice = &input_buf[0..len];
    
    // 🔥 [수정 4] 대소문자 무관 검색 (rror, arni)
    let is_error = input_slice.windows(4).any(|w| 
        w == b"rror" || w == b"RROR" || w == b"Rror"
    );
    let is_warning = input_slice.windows(4).any(|w| 
        w == b"arni" || w == b"ARNI" || w == b"Arni"
    );

    if !is_error && !is_warning {
        BUFFER_LOCK.store(false, Ordering::Release);
        return 0;
    }

    // 🔥 [6단계] 검거 성공! 로그 + 신원조회
    if is_error {
        TELEMETRY.hit_error.fetch_add(1, Ordering::Relaxed);
        ghost_printk(b"[GHOST] >>> ERROR CAPTURED! <<<\n\0".as_ptr());
        ghost_inspect_task(fd); // 🔥 FD 전달!
    } else {
        TELEMETRY.hit_warning.fetch_add(1, Ordering::Relaxed);
        ghost_printk(b"[GHOST] >>> WARNING CAPTURED! <<<\n\0".as_ptr());
        ghost_inspect_task(fd); // 🔥 FD 전달!
    }

    let translated = if is_error { 
        b"\xec\x98\xa4\xeb\xa5\x98\0"
    } else { 
        b"\xea\xb2\xbd\xea\xb3\xa0\0"
    };

    let output_buf = &mut TRANS_BUFFER[HALF_SIZE..BUFFER_SIZE];
    let json_len = match simple_replace_json_bytes(input_slice, translated, output_buf) {
        Some(len) => len,
        None => { 
            BUFFER_LOCK.store(false, Ordering::Release); 
            return 0; 
        }
    };
    
    if ghost_copy_to_user(user_buf, output_buf.as_ptr() as *const c_void, json_len as u64) != 0 {
        TELEMETRY.failed_copy.fetch_add(1, Ordering::Relaxed);
        BUFFER_LOCK.store(false, Ordering::Release);
        return 0;
    }

    (*regs).dx = json_len as u64;
    BUFFER_LOCK.store(false, Ordering::Release);
    0
}

fn simple_replace_json_bytes(original: &[u8], translated: &[u8], output: &mut [u8]) -> Option<usize> {
    let mut pos = 0;
    let prefix = b"{\"r\":\"";
    let middle = b"\",\"t\":\"";
    let suffix = b"\"}\n";

    if pos + prefix.len() > output.len() { return None; }
    for i in 0..prefix.len() { output[pos + i] = prefix[i]; }
    pos += prefix.len();

    if pos + original.len() > output.len() { return None; }
    for i in 0..original.len() { output[pos + i] = original[i]; }
    pos += original.len();

    if pos + middle.len() > output.len() { return None; }
    for i in 0..middle.len() { output[pos + i] = middle[i]; }
    pos += middle.len();

    let trans_len = translated.len() - 1;
    if pos + trans_len > output.len() { return None; }
    for i in 0..trans_len { output[pos + i] = translated[i]; }
    pos += trans_len;

    if pos + suffix.len() > output.len() { return None; }
    for i in 0..suffix.len() { output[pos + i] = suffix[i]; }
    pos += suffix.len();
    
    Some(pos)
}

// ============================================================
// 6. 초기화
// ============================================================
#[no_mangle]
pub unsafe extern "C" fn init_hook(sys_write: u64) -> i32 {
    ghost_printk(b"\n\0".as_ptr());
    ghost_printk(b"[GHOST] ========================================\n\0".as_ptr());
    ghost_printk(b"[GHOST]    FULL INSPECTION MODE ACTIVATED\n\0".as_ptr());
    ghost_printk(b"[GHOST] ========================================\n\0".as_ptr());
    ghost_printk(b"[GHOST] Target: __x64_sys_write\n\0".as_ptr());
    ghost_printk(b"[GHOST] Mission: Total Traffic Analysis + FD Tracking\n\0".as_ptr());
    ghost_printk(b"[GHOST] ========================================\n\0".as_ptr());

    let kp_ptr = KP.kp.get();
    (*kp_ptr).addr = sys_write as *mut c_void;
    (*kp_ptr).pre_handler = Some(handler_pre);
    
    if ghost_register_kprobe(kp_ptr) < 0 {
        ghost_printk(b"[GHOST] FATAL: Hook Failed!\n\0".as_ptr());
        return -1;
    }
    
    ghost_printk(b"[GHOST] Inspector Online. Watching...\n\0".as_ptr());
    0
}

#[no_mangle] 
pub unsafe extern "C" fn cleanup_hook() { 
    ghost_unregister_kprobe(KP.kp.get()); 
}

#[no_mangle] 
pub unsafe extern "C" fn print_stats() {
    let total = TELEMETRY.total_calls.load(Ordering::Relaxed);
    let tiny = TELEMETRY.size_tiny.load(Ordering::Relaxed);
    let small = TELEMETRY.size_small.load(Ordering::Relaxed);
    let medium = TELEMETRY.size_medium.load(Ordering::Relaxed);
    let large = TELEMETRY.size_large.load(Ordering::Relaxed);
    let errors = TELEMETRY.hit_error.load(Ordering::Relaxed);
    let warnings = TELEMETRY.hit_warning.load(Ordering::Relaxed);
    let failed = TELEMETRY.failed_copy.load(Ordering::Relaxed);
    
    ghost_printk(b"\n[GHOST] ========== FINAL REPORT ==========\n\0".as_ptr());
    
    if total > 0 {
        ghost_printk(b"[GHOST] Total sys_write calls observed\n\0".as_ptr());
    }
    
    if tiny > 0 {
        ghost_printk(b"[GHOST] Tiny packets detected\n\0".as_ptr());
    }
    
    if small > 0 {
        ghost_printk(b"[GHOST] Small packets detected\n\0".as_ptr());
    }
    
    if medium > 0 {
        ghost_printk(b"[GHOST] Medium packets detected\n\0".as_ptr());
    }
    
    if large > 0 {
        ghost_printk(b"[GHOST] Large packets detected\n\0".as_ptr());
    }
    
    if errors > 0 {
        ghost_printk(b"[GHOST] ERROR keywords intercepted!\n\0".as_ptr());
    }
    
    if warnings > 0 {
        ghost_printk(b"[GHOST] WARNING keywords intercepted!\n\0".as_ptr());
    }
    
    if failed > 0 {
        ghost_printk(b"[GHOST] Failed copy operations recorded\n\0".as_ptr());
    }
    
    ghost_printk(b"[GHOST] ====================================\n\n\0".as_ptr());
}

// ============================================================
// 7. 더미 (빌드 에러 방지)
// ============================================================
#[no_mangle]
pub unsafe extern "C" fn set_danger_zones(_: u64, _: u64, _: u64, _: u64) {}

#[panic_handler] 
fn panic(_info: &PanicInfo) -> ! { 
    loop {} 
}