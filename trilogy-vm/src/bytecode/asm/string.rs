// NOTE: Logic taken from scanner
fn unicode_escape_sequence(s: &str) -> Option<(char, &str)> {
    let s = s.strip_prefix('{')?;
    let repr: String = s.chars().take_while(|ch| ch.is_ascii_hexdigit()).collect();
    let s = s[repr.len()..].strip_prefix('}')?;
    let num = u32::from_str_radix(&repr, 16).ok()?;
    Some((char::from_u32(num)?, s))
}

fn ascii_escape_sequence(s: &str) -> Option<(char, &str)> {
    u32::from_str_radix(&s[0..2], 16)
        .ok()
        .and_then(char::from_u32)
        .map(|ch| (ch, &s[2..]))
}

pub fn escape_sequence(s: &str) -> Option<(char, &str)> {
    match s.strip_prefix('\\')? {
        s if s.starts_with('u') => unicode_escape_sequence(&s[1..]),
        s if s.starts_with('x') => ascii_escape_sequence(&s[1..]),
        s if s.starts_with(['"', '\'', '$', '\\']) => Some((s.chars().next()?, &s[1..])),
        s if s.starts_with('n') => Some(('\n', &s[1..])),
        s if s.starts_with('t') => Some(('\t', &s[1..])),
        s if s.starts_with('r') => Some(('\r', &s[1..])),
        s if s.starts_with('0') => Some(('\0', &s[1..])),
        _ => None,
    }
}

pub fn extract_string_prefix(s: &str) -> Option<(String, &str)> {
    let mut string = String::new();
    let mut s = &s[1..];
    loop {
        if s.is_empty() {
            return None;
        }
        if let Some(s) = s.strip_prefix('"') {
            return Some((string, s));
        }
        if s.starts_with('\\') {
            let (ch, rest) = escape_sequence(s)?;
            s = rest;
            string.push(ch);
            continue;
        }
        string.push(s.chars().next()?);
        s = &s[1..];
    }
}
