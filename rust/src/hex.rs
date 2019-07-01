fn hex_char_value(u: u8) -> Option<u8> {
    if (u >= b'0') && (u <= b'9') {
        return Some(u - 0x30);
    }
    if (u >= b'A') && (u <= b'F') {
        return Some(u - 0x41 + 10);
    }
    if (u >= b'a') && (u <= b'f') {
        return Some(u - 0x61 + 10);
    }

    None
}

pub fn hex_to_bytes(input: &str) -> Option<Vec<u8>> {
    if input.len() == 0 {
        return Some(vec![]);
    }

    if input.len() % 2 != 0 {
        return None
    }

    let bytes = input.as_bytes();
    let mut start_index: usize = 0;
    if (bytes[0] == b'0') && (bytes[1] == b'x') {
        start_index = 2;
    }

    let result_capacity = (bytes.len()-start_index)/2;
    let mut result = Vec::with_capacity(result_capacity);
    for i in 0..result_capacity {
        let v1 = hex_char_value(bytes[start_index + 2*i]);
        let v2 = hex_char_value(bytes[start_index + 2*i + 1]);

        let v1 = match v1 {
            Some(x) => x,
            None => return None
        };
        let v2 = match v2 {
            Some(x) => x,
            None => return None
        };

        let resultval = (v1 << 4) | v2;
        result.push(resultval);
    }

    Some(result)
}

pub fn bytes_to_hex(input: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut output = Vec::with_capacity(input.len() * 2);
    output.push(b'0');
    output.push(b'x');

    for b in input {
        let hi_nibble = ((b >> 4) & (0xF as u8)) as usize;
        let lo_nibble = (b & 0xF) as usize;
        output.push(HEX[hi_nibble]);
        output.push(HEX[lo_nibble]);
    }

    let output = String::from_utf8(output).unwrap();
    output
}

#[cfg(test)]
mod hex_tests {
    use super::*;

    #[test]
    fn hex2bytes_leading_0x_included() {
        let input = "0xDEADBEEF01";
        let output = hex_to_bytes(input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
            output
        );
    }

    #[test]
    fn hex2bytes_leading_0x_excluded() {
        let input = "DEADBEEF01";
        let output = hex_to_bytes(input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
            output
        );
    }

    #[test]
    fn bytes2hex() {
        let input = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x00];
        let output = bytes_to_hex(input);
        assert_eq!(
            "0xDEADBEEF0100",
            output
        );
    }
}
