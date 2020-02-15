enum Base64CharParseResult {
    Value(u8),
    Padding,
    Unknown
}

fn parse_base64_char(c: u8) -> Base64CharParseResult {
    return match c {
        b'A'..=b'Z' => Base64CharParseResult::Value(c - b'A'),
        b'a'..=b'z' => Base64CharParseResult::Value(c - b'a' + 26),
        b'0'..=b'9' => Base64CharParseResult::Value(c - b'0' + 52),
        b'+' => Base64CharParseResult::Value(62),
        b'/' => Base64CharParseResult::Value(63),
        b'=' => Base64CharParseResult::Padding,
        _ => Base64CharParseResult::Unknown
    }
}

pub fn to_bytes(input: &str) -> Option<Vec<u8>> {
    if input.len() == 0 {
        return Some(vec![]);
    }

    if input.len() % 4 != 0 {
        return None;
    }

    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(bytes.len()); // TODO: This could be *(3/4)
    let mut accumulated_bits = 0;
    let mut accumulated = 0;
    for i in 0..bytes.len() {
        let parse_result = parse_base64_char(bytes[i]);
        match parse_result {
            Base64CharParseResult::Value(val) => {
                accumulated = (accumulated << 6) | (val as u32);
                accumulated_bits += 6;
            },
            Base64CharParseResult::Padding => (),
            Base64CharParseResult::Unknown => panic!("Unrecognized base64 char")
        }

        while accumulated_bits >= 8 {
            let out = (accumulated >> (accumulated_bits-8) & 0xFF) as u8;
            result.push(out);
            accumulated_bits -= 8;
            accumulated &= !((0xFF as u32) << accumulated_bits);
        }
    }

    Some(result)
}

pub fn from_bytes(input: &[u8]) -> String {
    const HEX: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::with_capacity(input.len()); // TODO: This should be * 4/3
    let mut bits_available = 0;
    let mut current: u32 = 0;

    for b in input {
        current = (current << 8) | (*b as u32);
        bits_available += 8;
        if bits_available == 24 {
            while bits_available >= 6 {
                bits_available -= 6;
                let mask = 0x3F << bits_available;
                let new_out = ((current & mask) >> bits_available) as usize;
                current = current & !mask;
                output.push(HEX[new_out]);
            }
        }
    }

    if bits_available == 16 {
        current <<= 8;
        bits_available += 8;
        for _ in 0..3 {
            bits_available -= 6;
            let mask = 0x3F << bits_available;
            let new_out = ((current & mask) >> bits_available) as usize;
            current = current & !mask;
            output.push(HEX[new_out]);
        }
        output.push(b'=');
        bits_available = 0;

    } else if bits_available == 8 {
        current <<= 16;
        bits_available += 16;
        for _ in 0..2 {
            bits_available -= 6;
            let mask = 0x3F << bits_available;
            let new_out = ((current & mask) >> bits_available) as usize;
            current = current & !mask;
            output.push(HEX[new_out]);
        }
        output.push(b'=');
        output.push(b'=');
        bits_available = 0;
    }

    assert!(bits_available == 0, "Invalid number of bits: {}", bits_available);

    let output = String::from_utf8(output).unwrap();
    output
}

#[cfg(test)]
mod base64_tests {
    use super::*;

    #[test]
    fn base642bytes_two_padding_bytes() {
        let input = "3q2+7w==";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            output
        );
    }

    #[test]
    fn base642bytes_one_padding_bytes() {
        let input = "3q2+7wk=";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x09],
            output
        );
    }

    #[test]
    fn base642bytes_zero_padding_bytes() {
        let input = "3q2+7wkA";
        let output = to_bytes(&input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x09, 0x00],
            output
        );
    }

    #[test]
    fn bytes2base64_two_padding_bytes() {
        let input = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let output = from_bytes(&input);
        assert_eq!(
            "3q2+7w==",
            output
        );
    }

    #[test]
    fn bytes2base64_one_padding_byte() {
        let input = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x09];
        let output = from_bytes(&input);
        assert_eq!(
            "3q2+7wk=",
            output
        );
    }

    #[test]
    fn bytes2base64_zero_padding_bytes() {
        let input = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x09, 0x00];
        let output = from_bytes(&input);
        assert_eq!(
            "3q2+7wkA",
            output
        );
    }
}
