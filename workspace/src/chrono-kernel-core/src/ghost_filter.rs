#[inline(always)]
pub fn should_translate_text(text: &str) -> bool {
    // 너무 짧으면 패스
    if text.len() < 2 {
        return false;
    }
    
    // 이미 JSON이면 패스 (무한 루프 방지!)
    if text.starts_with("{\"r\":") {
        return false;
    }
    
    // 첫 글자가 영어 대문자
    let first = text.as_bytes()[0];
    if !first.is_ascii_uppercase() {
        return false;
    }
    
    true
}