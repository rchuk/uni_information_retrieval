use anyhow::Result;

const CONT_MASK: u8 = 0b10000000;

pub fn vb_encode(value: usize) -> Vec<u8> {
    if value == 0 {
        return vec![CONT_MASK];
    }

    let mut result = Vec::new();

    let mut acc = value;
    while acc != 0 {
        result.push((acc % 128) as u8);
        acc /= 128;
    }

    result.reverse();
    if let Some(last) = result.last_mut() {
        *last |= CONT_MASK;
    }

    result
}

pub fn vb_decode(data: &mut impl Iterator<Item = Result<u8, std::io::Error>>) -> Result<usize> {
    let mut result = 0;
    while let Some(byte) = data.next() {
        let byte = byte?;
        result = (result << 7) | ((byte & 127) as usize);
        if byte & CONT_MASK == CONT_MASK {
            break;
        }
    }

    Ok(result)
}
