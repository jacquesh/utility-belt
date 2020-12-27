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

pub fn to_bytes(input: &str) -> Option<Vec<u8>> {
    if input.len() == 0 {
        return Some(vec![]);
    }

    let bytes = input.as_bytes();
    let mut start_index: usize = 0;
    if (bytes.len() > 1) && (bytes[0] == b'0') && (bytes[1] == b'x') {
        start_index = 2;
    }

    let max_result_length = (bytes.len()-start_index)/2 + 1;
    let mut result = Vec::with_capacity(max_result_length);
    if (bytes.len()-start_index) % 2 == 1 {
        let hexval = hex_char_value(bytes[start_index]);
        let hexval = match hexval {
            Some(x) => x,
            None => return None
        };
        result.push(hexval);
        start_index += 1;
    }

    let mut current_byte = 0;
    let mut nibble_count = 0;
    for i in start_index..bytes.len() {
        if bytes[i] == b'-' {
            continue;
        }

        let hexval = hex_char_value(bytes[i]);
        let hexval = match hexval {
            Some(x) => x,
            None => return None
        };

        current_byte = (current_byte << 4) | hexval;
        nibble_count += 1;
        if nibble_count == 2 {
            result.push(current_byte);
            nibble_count = 0;
            current_byte = 0;
        }
    }

    if nibble_count != 0 {
        result.push(current_byte);
    }

    Some(result)
}

pub fn from_bytes(input: &[u8]) -> String {
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
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
            output
        );
    }

    #[test]
    fn hex2bytes_leading_0x_excluded() {
        let input = "DEADBEEF01";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01],
            output
        );
    }

    #[test]
    fn hex2bytes_odd_number_of_chars() {
        let input = "100";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0x01, 0x00],
            output
        );
    }

    #[test]
    fn bytes2hex() {
        let input = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x00];
        let output = from_bytes(&input);
        assert_eq!(
            "0xDEADBEEF0100",
            output
        );
    }

    #[test]
    fn bytes2hex_odd_number_of_chars()
    {
        let input = vec![0x01, 0x00];
        let output = from_bytes(&input);
        assert_eq!(
            "0x0100",
            output
        );
    }
}
