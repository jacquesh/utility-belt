pub fn to_bytes(input: &str) -> Option<Vec<u8>> {
    if input.len() == 0 {
        return Some(vec![]);
    }

    let bytes = input.as_bytes();
    let mut start_index: usize = 0;
    if (bytes.len() > 1) && (bytes[0] == b'0') && (bytes[1] == b'b') {
        start_index = 2;
    }

    let max_result_length = (bytes.len()-start_index)/8 + 1;
    let mut result = Vec::with_capacity(max_result_length);
    let mut current_byte = 0;
    let mut bit_count = 0;
    for i in (start_index..bytes.len()).rev() {
        if crate::is_char_ignorable(bytes[i]) {
            continue;
        }
        let bitval = match bytes[i] {
            b'0' => 0,
            b'1' => 1,
            _ => return None
        };

        current_byte |= bitval << bit_count;
        bit_count += 1;
        if bit_count == 8 {
            result.push(current_byte);
            bit_count = 0;
            current_byte = 0;
        }
    }

    if bit_count != 0 {
        result.push(current_byte);
    }

    result.reverse();
    Some(result)
}

pub fn from_bytes(input: &[u8], trim_leading_zeros: bool) -> String {
    let mut output = Vec::with_capacity(2 + (input.len() * 8));
    output.push(b'0');
    output.push(b'b');
    let mut nonzero_seen = false;

    for byte in input {
        for bit in (0..8).rev() {
            let bit_char = b'0' + ((byte >> bit) & (0x01 as u8));
            if trim_leading_zeros && !nonzero_seen {
                match bit_char {
                    b'0' => continue,
                    b'1' => nonzero_seen = true,
                    _ => {}
                }
            }
            output.push(bit_char);
        }
    }

    let output = String::from_utf8(output).unwrap();
    output
}

#[cfg(test)]
mod binary_tests {
    use super::*;

    #[test]
    fn tobytes_leading_0b_included() {
        let input = "0b11111111";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0xFF],
            output
        );
    }

    #[test]
    fn tobytes_leading_0b_excluded() {
        let input = "11111111";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0xFF],
            output
        );
    }

    #[test]
    fn tobytes_leading_zeroes_excluded() {
        let input = "0b11100000001";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0x07, 0x01],
            output
        );
    }

    #[test]
    fn frombytes_leading_zeroes_included() {
        let input = vec![0x07, 0x01];
        let output = from_bytes(&input, false);
        assert_eq!(
            "0b0000011100000001",
            output
        );
    }

    #[test]
    fn frombytes_leading_zeroes_excluded() {
        let input = vec![0x07, 0x01];
        let output = from_bytes(&input, true);
        assert_eq!(
            "0b11100000001",
            output
        );
    }
}
