use std::num::ParseIntError;

pub fn get_compact_size_bytes(data: &u64) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    if *data <= 0xFC {
        result.push(*data as u8);
    } else if *data <= 0xFFFF {
        result.push(0xFD);
        (*data as u16)
            .to_be_bytes()
            .iter()
            .rev()
            .for_each(|val| result.push(*val));
    } else if *data <= 0xFFFFFFFF {
        result.push(0xFE);
        (*data as u32)
            .to_be_bytes()
            .iter()
            .rev()
            .for_each(|val| result.push(*val));
    } else {
        result.push(0xFF);
        (*data as u64)
            .to_be_bytes()
            .iter()
            .rev()
            .for_each(|val| result.push(*val));
    }
    result
}

pub fn get_hex_bytes(num: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..num.len()).step_by(2).map(|i| u8::from_str_radix(&num[i..i+2], 16)).collect()
}
