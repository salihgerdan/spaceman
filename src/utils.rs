pub fn bytes_display(bytes: u64) -> String {
    let bytes_f = bytes as f64;
    if bytes_f > 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2}GB", bytes_f / (1024.0 * 1024.0 * 1024.0))
    } else if bytes_f > 1024.0 * 1024.0 {
        format!("{:.2}MB", bytes_f / (1024.0 * 1024.0))
    } else if bytes_f > 1024.0 {
        format!("{:.2}KB", bytes_f / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

pub fn abbreviate_string(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => s[..idx - 3].to_string() + "...",
    }
}
