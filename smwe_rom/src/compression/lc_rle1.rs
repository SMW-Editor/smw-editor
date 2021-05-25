use std::convert::TryFrom;

use num_enum::TryFromPrimitive;

use crate::error::DecompressionError;

#[repr(u8)]
#[derive(Copy, Clone, Debug, TryFromPrimitive)]
enum Command {
    DirectCopy = 0,
    ByteFill = 1,
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>, DecompressionError> {
    assert!(!input.is_empty());
    assert!(!input.len() >= 2);
    let mut output = Vec::with_capacity(input.len()* 2);
    let mut in_it = input;
    while let Some(chunk_header) = in_it.first().copied() {
        if chunk_header == 0xFF && (in_it.len() == 1 || in_it[1] == 0xFF) {
            break;
        }
        in_it = &in_it[1..];
        let command = (chunk_header >> 7) & 1;
        let command = Command::try_from(command).map_err(|_| DecompressionError("Reading command"))?;
        let length = (chunk_header & 0b01111111) as usize + 1;

        match command {
            Command::DirectCopy => {
                let (bytes, rest) = in_it.split_at(length);
                output.extend_from_slice(bytes);
                in_it = rest;
            }
            Command::ByteFill => {
                let byte = *in_it.first().ok_or(DecompressionError("Reading byte to fill"))?;
                output.resize(output.len() + length, byte);
                in_it = &in_it[1..];
            }
        }
    }

    output.shrink_to_fit();
    Ok(output)
}