pub mod base64;
pub mod binary;
pub mod decimal;
pub mod hex;

pub fn is_char_ignorable(c: u8) -> bool {
    return match c {
        b' ' => true,
        b'-' => true,
        b'_' => true,
        b',' => true,
        _ => false
    }
}
