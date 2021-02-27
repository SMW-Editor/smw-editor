use crate::error::DecompressionError;

use num_enum::TryFromPrimitive;

use std::convert::TryFrom;

#[repr(u8)]
#[derive(Copy, Clone, Debug, TryFromPrimitive)]
enum Command {
    DirectCopy     = 0b000, // Followed by (L+1) bytes of data

    ByteFill       = 0b001, // Followed by one byte to be repeated (L+1) times

    WordFill       = 0b010, // Followed by two bytes. Output first byte, then second, then first,
                            // then second, etc. until (L+1) bytes has been outputted

    IncreasingFill = 0b011, // Followed by one byte to be repeated (L+1) times, but the byte is
                            // increased by 1 after each write

    Repeat         = 0b100, // Followed by two bytes (ABCD byte order) containing address (in the
                            // output buffer) to copy (L+1) bytes from

    LongLength     = 0b111, // This command has got a two-byte header:
                            // 111CCCLL LLLLLLLL
                            // CCC:        Real command
                            // LLLLLLLLLL: Length
}

pub fn lc_lz2_decompress(input: &[u8]) -> Result<Vec<u8>, DecompressionError> {
    assert!(!input.is_empty());
    let mut output = Vec::with_capacity(input.len() * 2);
    let mut in_it = input;
    while let Some(chunk_header) = in_it.first().copied() {
        if chunk_header == 0xFF {
            break;
        }
        in_it = &in_it[1..];
        let command_bits = (chunk_header & 0b11100000) >> 5;
        let length = chunk_header & 0b00011111;

        let mut command = Command::try_from(command_bits).map_err(
            |_| DecompressionError("Reading command"))?;
        let mut length = length as usize + 1;

        if let Command::LongLength = command {
            let real_command_bits = (chunk_header & 0b00011100) >> 2;
            command = Command::try_from(real_command_bits).map_err(
                |_| DecompressionError("Reading long command"))?;
            let length_part_1 = chunk_header & 0b00000011;
            let length_part_2 = *in_it.first().ok_or(
                DecompressionError("Reading long length"))?;
            length = (((length_part_1 as usize) << 8) | (length_part_2 as usize)) + 1;
            in_it = &in_it[1..];
        }

        use Command::*;
        match command {
            DirectCopy => {
                let (bytes, rest) = in_it.split_at(length);
                output.extend_from_slice(bytes);
                in_it = rest;
            }
            ByteFill => {
                let byte = *in_it.first().ok_or(
                    DecompressionError("Reading byte to fill"))?;
                output.resize(output.len() + length, byte);
                in_it = &in_it[1..];
            }
            WordFill => {
                let (bytes, rest) = in_it.split_at(2);
                output.extend(bytes.iter().cycle().take(length));
                in_it = rest;
            }
            IncreasingFill => {
                let mut byte = *in_it.first().ok_or(
                    DecompressionError("Reading byte to increasingly fill"))?;
                output.extend(std::iter::repeat_with(|| {
                    let temp = byte;
                    byte = byte.wrapping_add(1);
                    temp
                }).take(length));
                in_it = &in_it[1..];
            }
            Repeat => {
                let (bytes, rest) = in_it.split_at(2);
                let read_start = ((bytes[0] as usize) << 8) | (bytes[1] as usize);
                let write_start = output.len();
                output.resize(output.len() + length, 0);
                output.copy_within(read_start..(read_start + length), write_start);
                in_it = rest;
            }
            LongLength => return Err(DecompressionError("Double long length command"))
        }
    }

    output.shrink_to_fit();
    Ok(output)
}
