use std::num::ParseIntError;

pub fn get_compact_size_bytes(data: u64) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    if data <= 0xFC {
        result.push(data as u8);
    } else if data <= 0xFFFF {
        result.push(0xFD);
        (data as u16)
            .to_be_bytes()
            .iter()
            .for_each(|val| result.push(*val));
    } else if data <= 0xFFFFFFFF {
        result.push(0xFE);
        (data as u32)
            .to_be_bytes()
            .iter()
            .for_each(|val| result.push(*val));
    } else {
        result.push(0xFF);
        (data as u64)
            .to_be_bytes()
            .iter()
            .for_each(|val| result.push(*val));
    }
    result
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::get_compact_size_bytes;

    #[test]
    pub fn test_compact_size() {
        let result = get_compact_size_bytes(131);
        let result = get_compact_size_bytes(965);
    }
}
