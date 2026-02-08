#[inline]
pub fn wrap_json(
    original: &str,
    translated: &str,
    output: &mut [u8]
) -> Result<usize, ()> {
    let mut pos = 0;
    
    // {"r":"
    let prefix = b"{\"r\":\"";
    if pos + prefix.len() > output.len() {
        return Err(());
    }
    output[pos..pos+prefix.len()].copy_from_slice(prefix);
    pos += prefix.len();
    
    // Original
    let orig = original.as_bytes();
    if pos + orig.len() > output.len() {
        return Err(());
    }
    output[pos..pos+orig.len()].copy_from_slice(orig);
    pos += orig.len();
    
    // ","t":"
    let middle = b"\",\"t\":\"";
    if pos + middle.len() > output.len() {
        return Err(());
    }
    output[pos..pos+middle.len()].copy_from_slice(middle);
    pos += middle.len();
    
    // Translated
    let trans = translated.as_bytes();
    if pos + trans.len() > output.len() {
        return Err(());
    }
    output[pos..pos+trans.len()].copy_from_slice(trans);
    pos += trans.len();
    
    // "}
    let suffix = b"\"}";
    if pos + suffix.len() > output.len() {
        return Err(());
    }
    output[pos..pos+suffix.len()].copy_from_slice(suffix);
    pos += suffix.len();
    
    Ok(pos)
}