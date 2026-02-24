
// Universal i18n Layer for Linux Kernel - Translation Core
// Author: Bureum Lee
// License: MIT

use core::slice;
use core::str;

// ═══════════════════════════════════════════════════════════════════════════
// Translation Entry Structure
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, Copy)]
pub struct TranslationEntry {
    pub original: &'static str,
    pub translated: &'static str,
    pub len_original: usize,
    pub len_translated: usize,
}

impl TranslationEntry {
    pub const fn new(original: &'static str, translated: &'static str) -> Self {
        Self {
            original,
            translated,
            len_original: original.len(),
            len_translated: translated.len(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Static Translation Table (Korean)
// ═══════════════════════════════════════════════════════════════════════════
pub const I18N_TABLE_KO: &[TranslationEntry] = &[
    // System Status
    TranslationEntry::new("Ready!", "준비 완료!"),
    TranslationEntry::new("Loading...", "불러오는 중..."),
    TranslationEntry::new("Done", "완료"),
    TranslationEntry::new("OK", "확인"),
    
    // Common Errors
    TranslationEntry::new("Error", "오류"),
    TranslationEntry::new("error", "오류"),
    TranslationEntry::new("ERROR", "오류"),
    TranslationEntry::new("Warning", "경고"),
    TranslationEntry::new("warning", "경고"),
    TranslationEntry::new("Failed", "실패"),
    TranslationEntry::new("failed", "실패"),
    
    // Permission & Access
    TranslationEntry::new("Permission denied", "권한이 거부되었습니다"),
    TranslationEntry::new("permission denied", "권한이 거부되었습니다"),
    TranslationEntry::new("Access denied", "접근이 거부되었습니다"),
    TranslationEntry::new("Operation not permitted", "허용되지 않은 작업입니다"),
    
    // File System
    TranslationEntry::new("No such file or directory", "파일 또는 디렉토리가 없습니다"),
    TranslationEntry::new("File not found", "파일을 찾을 수 없습니다"),
    TranslationEntry::new("file not found", "파일을 찾을 수 없습니다"),
    TranslationEntry::new("Directory not empty", "디렉토리가 비어있지 않습니다"),
    TranslationEntry::new("Is a directory", "디렉토리입니다"),
    TranslationEntry::new("Not a directory", "디렉토리가 아닙니다"),
    
    // Memory & Resources
    TranslationEntry::new("Out of memory", "메모리가 부족합니다"),
    TranslationEntry::new("Cannot allocate memory", "메모리를 할당할 수 없습니다"),
    TranslationEntry::new("Resource temporarily unavailable", "리소스를 일시적으로 사용할 수 없습니다"),
    TranslationEntry::new("Device or resource busy", "장치가 사용 중입니다"),
    
    // I/O Operations
    TranslationEntry::new("Input/output error", "입출력 오류"),
    TranslationEntry::new("I/O error", "입출력 오류"),
    TranslationEntry::new("Read error", "읽기 오류"),
    TranslationEntry::new("Write error", "쓰기 오류"),
    TranslationEntry::new("Bad file descriptor", "잘못된 파일 디스크립터"),
    
    // Network
    TranslationEntry::new("Connection refused", "연결이 거부되었습니다"),
    TranslationEntry::new("Connection reset", "연결이 재설정되었습니다"),
    TranslationEntry::new("Connection timed out", "연결 시간이 초과되었습니다"),
    TranslationEntry::new("Network is unreachable", "네트워크에 접근할 수 없습니다"),
    
    // Process & Signals
    TranslationEntry::new("Killed", "강제 종료됨"),
    TranslationEntry::new("Terminated", "종료됨"),
    TranslationEntry::new("Segmentation fault", "세그먼테이션 오류"),
    TranslationEntry::new("Illegal instruction", "잘못된 명령"),
    
    // Kernel Messages
    TranslationEntry::new("Kernel panic", "커널 패닉"),
    TranslationEntry::new("Oops", "커널 오류"),
    TranslationEntry::new("BUG", "버그"),
    TranslationEntry::new("Call Trace", "호출 추적"),
];

// ═══════════════════════════════════════════════════════════════════════════
// Translation Engine - Ultra-Fast Lookup
// ═══════════════════════════════════════════════════════════════════════════
/// Translates input string using static table
/// Returns Some(translated) if match found, None otherwise
#[inline]
pub fn translate(input: &str) -> Option<&'static str> { //변경: Option 반환
    // Early return for empty input
    if input.is_empty() {
        return None; // 변경: None 반환
    }
    
    // Trim trailing newlines, nulls, whitespace
    // Kernel messages often end with \n or \0
    let trimmed = input.trim_end_matches(|c: char| {
        c == '\n' || c == '\r' || c == '\0' || c.is_whitespace()
    });
    
    if trimmed.is_empty() {
        return None; //  변경: None 반환
    }
    
    // Linear search (sufficient for small table)
    // For larger tables, consider binary search or hash map
    for entry in I18N_TABLE_KO {
        // Use eq_ignore_ascii_case for case-insensitive comparison
        // This avoids heap allocation (no String created)
        if trimmed.eq_ignore_ascii_case(entry.original) {
            return Some(entry.translated); // 변경: Some으로 감싸서 반환
        }
    }
    
    // No match - return None (This lets the hook fallback to original)
    None //  변경: 못 찾으면 깔끔하게 None
}

/// Translates a slice of bytes (for kernel buffer)
/// Returns translated string if valid UTF-8 and match found
pub unsafe fn translate_bytes(buf: *const u8, len: usize) -> Option<&'static str> {
    // Safety check
    if buf.is_null() || len == 0 {
        return None;
    }
    
    // Create slice from raw pointer
    let slice = slice::from_raw_parts(buf, len);
    
    // Parse as UTF-8
    let text = match str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return None, // Invalid UTF-8
    };
    
    // Lookup translation
    // 변경: translate가 이제 Option을 반환하므로 로직이 단순해짐
    translate(text)
}

/// Checks if input should be translated
/// Ultra-fast filtering logic
#[inline]
pub fn should_translate_str(input: &str) -> bool {
    // Length check
    if input.is_empty() || input.len() > 256 {
        return false;
    }
    
    // Trim first
    let trimmed = input.trim_end_matches(|c: char| {
        c == '\n' || c == '\r' || c == '\0' || c.is_whitespace()
    });
    
    if trimmed.is_empty() {
        return false;
    }
    
    // Must contain ASCII alphabetic characters
    let has_alpha = trimmed.bytes().any(|b| b.is_ascii_alphabetic());
    if !has_alpha {
        return false;
    }
    
    // Check prefixes without heap allocation
    let prefixes = [
        b"error" as &[u8],
        b"warning",
        b"permission",
        b"no such",
        b"cannot",
        b"failed",
    ];
    
    let input_bytes = trimmed.as_bytes();
    for prefix in &prefixes {
        if input_bytes.len() >= prefix.len() {
            // Case-insensitive prefix check
            let input_prefix = &input_bytes[..prefix.len()];
            if input_prefix.eq_ignore_ascii_case(prefix) {
                return true;
            }
        }
    }
    
    // Check if input is in translation table (case-insensitive)
    for entry in I18N_TABLE_KO {
        if trimmed.eq_ignore_ascii_case(entry.original) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_translation() {
        // 변경: translate() 결과가 Option이 되었으므로 Some/None으로 비교
        assert_eq!(translate("Error"), Some("오류"));
        assert_eq!(translate("Permission denied"), Some("권한이 거부되었습니다"));
        assert_eq!(translate("unknown"), None); // 못 찾으면 None
    }
    
    #[test]
    fn test_translation_with_newline() {
        // 변경: Some 감싸기
        assert_eq!(translate("Error\n"), Some("오류"));
        assert_eq!(translate("Permission denied\r\n"), Some("권한이 거부되었습니다"));
        assert_eq!(translate("Ready!\0"), Some("준비 완료!"));
    }
    
    #[test]
    fn test_case_insensitive() {
        // 변경: Some 감싸기
        assert_eq!(translate("error"), Some("오류"));
        assert_eq!(translate("ERROR"), Some("오류"));
        assert_eq!(translate("ErRoR"), Some("오류"));
    }
    
    #[test]
    fn test_should_translate() {
        assert!(should_translate_str("Error"));
        assert!(should_translate_str("Permission denied"));
        assert!(should_translate_str("error\n"));  // With newline
        assert!(!should_translate_str("12345"));
        assert!(!should_translate_str(""));
    }
    
    #[test]
    fn test_translate_bytes() {
        let input = b"Error";
        let result = unsafe { translate_bytes(input.as_ptr(), input.len()) };
        assert_eq!(result, Some("오류"));
        
        let with_newline = b"Error\n";
        let result = unsafe { translate_bytes(with_newline.as_ptr(), with_newline.len()) };
        assert_eq!(result, Some("오류"));
        
        let invalid = &[0xFF, 0xFE];
        let result = unsafe { translate_bytes(invalid.as_ptr(), invalid.len()) };
        assert_eq!(result, None);
    }
}
