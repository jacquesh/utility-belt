use std::mem::size_of;

pub fn to_bytes(input: &str) -> Option<Vec<u8>> {
    if input.len() == 0 {
        return Some(vec![]);
    }

    let input = match input.parse::<u64>() {
        Ok(i) => i,
        Err(_) => {
            return None;
        }
    };

    let mut result = Vec::with_capacity(size_of::<u64>());
    let mut current = input;
    while current != 0 {
        let temp = (current & 0xFF) as u8;
        current >>= 8;
        result.push(temp);
    }

    result.reverse();
    Some(result)
}

pub fn from_bytes(input: &[u8]) -> String {
    let mut value: u64 = 0;

    for (index, byte) in input.iter().rev().enumerate() {
        value |= (*byte as u64) << (index*8);
    }

    value.to_string()
}

#[cfg(test)]
mod binary_tests {
    use super::*;

    #[test]
    fn tobytes() {
        let input = "4358";
        let output = to_bytes(input).unwrap();
        assert_eq!(
            vec![0x11, 0x06],
            output
        );
    }

    #[test]
    fn frombytes() {
        let input = vec![0x11, 0x06];
        let output = from_bytes(&input);
        assert_eq!(
            "4358",
            output
        );
    }

    #[test]
    fn frombytes_leading_zeroes_excluded() {
        let input = vec![0x00, 0x01, 0x06];
        let output = from_bytes(&input);
        assert_eq!(
            "262",
            output
        );
    }
}
