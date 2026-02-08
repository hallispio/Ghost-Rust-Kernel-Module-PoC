use core::sync::atomic::{AtomicBool, Ordering};

// 64KB 대저택!
pub const BUFFER_SIZE: usize = 65536;
pub const HALF_SIZE: usize = 32768;
// 🔥 [핵심] 커널 데이터 섹션(.data)에 강제로 박아버림
#[link_section = ".data"]
static mut TRANS_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static BUFFER_LOCK: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn try_lock() -> bool {
    BUFFER_LOCK.compare_exchange(
        false, true,
        Ordering::Acquire,
        Ordering::Relaxed
    ).is_ok()
}

#[inline]
pub fn unlock() {
    BUFFER_LOCK.store(false, Ordering::Release);
}

// 입력 버퍼 (0..32KB)
#[inline]
pub unsafe fn get_input_buffer() -> &'static mut [u8] {
    &mut TRANS_BUFFER[0..HALF_SIZE]
}

// 출력 버퍼 (32KB..64KB)
#[inline]
pub unsafe fn get_output_buffer() -> &'static mut [u8] {
    &mut TRANS_BUFFER[HALF_SIZE..BUFFER_SIZE]
}