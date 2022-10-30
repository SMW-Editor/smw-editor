use crate::error::{DecompressionError, LcRle1Error};

const COMMAND_DIRECT_COPY: u8 = 0;
const COMMAND_BYTE_FILL: u8 = 1;

/// Returns decompressed data and the size of compressed data.
pub fn decompress(input: &[u8]) -> Result<(Vec<u8>, usize), DecompressionError> {
    assert!(!input.is_empty());
    assert!(!input.len() >= 2);
    let mut output = Vec::with_capacity(input.len() * 2);
    let mut in_it = input;
    while let Some(chunk_header) = in_it.first().copied() {
        if chunk_header == 0xFF && (in_it.len() == 1 || in_it[1] == 0xFF) {
            break;
        }
        in_it = &in_it[1..];
        let command = chunk_header >> 7;
        let length = (chunk_header & 0b01111111) as usize + 1;

        match command {
            COMMAND_DIRECT_COPY => {
                if length <= in_it.len() {
                    let (bytes, rest) = in_it.split_at(length);
                    output.extend_from_slice(bytes);
                    in_it = rest;
                } else {
                    return Err(LcRle1Error::DirectCopy(length).into());
                }
            }
            COMMAND_BYTE_FILL => {
                let byte = *in_it.first().ok_or(LcRle1Error::ByteFill)?;
                output.resize(output.len() + length, byte);
                in_it = &in_it[1..];
            }
            _ => unreachable!(),
        }
    }

    output.shrink_to_fit();
    let bytes_consumed = input.len() - in_it.len();
    Ok((output, bytes_consumed))
}
