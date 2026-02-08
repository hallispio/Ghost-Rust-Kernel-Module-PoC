#![no_std]
#![no_main]
#![no_builtins]

use core::panic::PanicInfo;
use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};

// 🔥 [추가] memcpy 직접 구현 (GOT 차단 핵심!)
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

extern "C" {
    fn ghost_printk(fmt: *const u8);
    fn _copy_from_user(to: *mut c_void, from: *const c_void, n: u64) -> u64;
    fn _copy_to_user(to: *mut c_void, from: *const c_void, n: u64) -> u64;
    fn register_kprobe(kp: *mut kprobe) -> i32;
    fn unregister_kprobe(kp: *mut kprobe);
}

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

const BUFFER_SIZE: usize = 65536;
const HALF_SIZE: usize = 32768;
static mut TRANS_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static BUFFER_LOCK: AtomicBool = AtomicBool::new(false);

struct SafeKprobe { kp: core::cell::UnsafeCell<kprobe> }
unsafe impl Sync for SafeKprobe {}

static KP: SafeKprobe = SafeKprobe { 
    kp: core::cell::UnsafeCell::new(unsafe { core::mem::MaybeUninit::zeroed().assume_init() }) 
};

unsafe extern "C" fn handler_pre(_p: *mut kprobe, regs: *mut pt_regs) -> i32 {
    ghost_printk(b"[GHOST] !!! HANDLER CALLED !!!\n\0".as_ptr());
    let fd = (*regs).di as i32;
    let len = (*regs).dx as usize;
    
    // 🔥 [로그 1] 모든 write 감지
    ghost_printk(b"[GHOST] Write: fd=%d len=%d\n\0".as_ptr());
    
    // 🔥 [로그 2] 길이 체크
    if len < 5 {
        ghost_printk(b"[GHOST] Skip: too short\n\0".as_ptr());
        return 0;
    }
    if len > 100 {
        ghost_printk(b"[GHOST] Skip: too long\n\0".as_ptr());
        return 0;
    }

    let user_buf = (*regs).si as *mut c_void;
    
    if BUFFER_LOCK.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() { 
        ghost_printk(b"[GHOST] Skip: locked\n\0".as_ptr());
        return 0; 
    }

    let input_buf = &mut TRANS_BUFFER[0..HALF_SIZE];
    if _copy_from_user(input_buf.as_mut_ptr() as *mut c_void, user_buf, len as u64) != 0 {
        ghost_printk(b"[GHOST] ERROR: copy_from_user failed\n\0".as_ptr());
        BUFFER_LOCK.store(false, Ordering::Release);
        return 0;
    }

    // 🔥 [로그 3] 복사 성공
    ghost_printk(b"[GHOST] Data copied OK\n\0".as_ptr());

    let input_slice = &input_buf[0..len];
    
    let is_error = input_slice.windows(5).any(|w| w == b"Error");
    let is_warning = input_slice.windows(7).any(|w| w == b"Warning");

    if !is_error && !is_warning {
        ghost_printk(b"[GHOST] Skip: no keyword\n\0".as_ptr());
        BUFFER_LOCK.store(false, Ordering::Release);
        return 0;
    }

    // 🔥 [로그 4] 키워드 발견
    if is_error {
        ghost_printk(b"[GHOST] Found: Error\n\0".as_ptr());
    } else {
        ghost_printk(b"[GHOST] Found: Warning\n\0".as_ptr());
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
            ghost_printk(b"[GHOST] ERROR: json build failed\n\0".as_ptr());
            BUFFER_LOCK.store(false, Ordering::Release); 
            return 0; 
        }
    };
    
    if _copy_to_user(user_buf, output_buf.as_ptr() as *const c_void, json_len as u64) != 0 {
        ghost_printk(b"[GHOST] ERROR: copy_to_user failed\n\0".as_ptr());
        BUFFER_LOCK.store(false, Ordering::Release);
        return 0;
    }

    (*regs).dx = json_len as u64;
    BUFFER_LOCK.store(false, Ordering::Release);
    
    // 🔥 [로그 5] 최종 성공
    ghost_printk(b"[GHOST] SUCCESS: Trapped!\n\0".as_ptr());
    0
}

// 🔥 [수정] copy_from_slice 대신 수동 복사 (memcpy 호출 방지)
fn simple_replace_json_bytes(original: &[u8], translated: &[u8], output: &mut [u8]) -> Option<usize> {
    let mut pos = 0;
    let prefix = b"{\"r\":\"";
    let middle = b"\",\"t\":\"";
    let suffix = b"\"}\n";

    // 🔥 [수정] copy_from_slice → 수동 복사
    if pos + prefix.len() > output.len() { return None; }
    for i in 0..prefix.len() {
        output[pos + i] = prefix[i];
    }
    pos += prefix.len();

    if pos + original.len() > output.len() { return None; }
    for i in 0..original.len() {
        output[pos + i] = original[i];
    }
    pos += original.len();

    if pos + middle.len() > output.len() { return None; }
    for i in 0..middle.len() {
        output[pos + i] = middle[i];
    }
    pos += middle.len();

    let trans_len = translated.len() - 1;
    if pos + trans_len > output.len() { return None; }
    for i in 0..trans_len {
        output[pos + i] = translated[i];
    }
    pos += trans_len;

    if pos + suffix.len() > output.len() { return None; }
    for i in 0..suffix.len() {
        output[pos + i] = suffix[i];
    }
    pos += suffix.len();
    
    Some(pos)
}

#[no_mangle]
pub unsafe extern "C" fn init_hook(sys_write: u64) -> i32 {
    ghost_printk(b"\n\0".as_ptr());
    ghost_printk(b"[GHOST]       ______  __  __  ______  ______  ______\n\0".as_ptr());
    ghost_printk(b"[GHOST]      / ____/ / / / / / __  / / ____/ /_  __/\n\0".as_ptr());
    ghost_printk(b"[GHOST]     / / __  / /_/ / / / / / / \\__ \\   / /   \n\0".as_ptr());
    ghost_printk(b"[GHOST]    / /_/ / / __  / / /_/ / ___/ /  / /      \n\0".as_ptr());
    ghost_printk(b"[GHOST]    \\____/ /_/ /_/  \\____/ /____/   /_/       \n\0".as_ptr());
    ghost_printk(b"[GHOST] \n\0".as_ptr());
    
    ghost_printk(b"[GHOST] \xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\n\0".as_ptr());
    ghost_printk(b"[GHOST] \xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x92\xE2\x96\x91\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x91\xE2\x96\x91\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\n\0".as_ptr());
    ghost_printk(b"[GHOST] \xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x91\x20\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\x20\xE2\x96\x91\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\n\0".as_ptr());
    ghost_printk(b"[GHOST] \xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x91\x20\x20\x20\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x91\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\n\0".as_ptr());
    ghost_printk(b"[GHOST] \xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\n\0".as_ptr());
    ghost_printk(b"[GHOST] \xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\n\0".as_ptr());
    ghost_printk(b"[GHOST] \xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x92\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\xE2\x96\x93\n\0".as_ptr());
    ghost_printk(b"[GHOST] \n\0".as_ptr());
    ghost_printk(b"[GHOST]       ______  __  __  ______  ______  ______\n\0".as_ptr());
    ghost_printk(b"[GHOST] [SUCCESS] __x64_sys_write captured. System Online.\n\0".as_ptr());
    ghost_printk(b"[GHOST] [SYSTEM] Arch: x86_64 | Mem: 64KB (Zero-Alloc)\n\0".as_ptr());

    let kp_ptr = KP.kp.get();
    (*kp_ptr).addr = sys_write as *mut c_void;
    (*kp_ptr).pre_handler = Some(handler_pre);
    
    if register_kprobe(kp_ptr) < 0 {
        ghost_printk(b"[GHOST] FATAL: Hook Failed!\n\0".as_ptr());
        return -1;
    }
    ghost_printk(b"[GHOST] OK: Hooked __x64_sys_write.\n\0".as_ptr());
    0
}

#[no_mangle] 
pub unsafe extern "C" fn cleanup_hook() { 
    unregister_kprobe(KP.kp.get()); 
}

#[no_mangle] 
pub unsafe extern "C" fn print_stats() {}

#[panic_handler] 
fn panic(_info: &PanicInfo) -> ! { 
    loop {} 
}