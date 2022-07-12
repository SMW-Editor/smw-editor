use std::convert::TryFrom;

use num_enum::TryFromPrimitive;

use crate::error::{DecompressionError, LcLz2Error};

#[repr(u8)]
#[derive(Copy, Clone, Debug, TryFromPrimitive)]
enum Command {
    /// Followed by (L+1) bytes of data
    DirectCopy     = 0b000,

    /// Followed by one byte to be repeated (L+1) times
    ByteFill       = 0b001,

    /// Followed by two bytes. Output first byte, then second, then first,
    /// then second, etc. until (L+1) bytes has been outputted
    WordFill       = 0b010,

    /// Followed by one byte to be repeated (L+1) times, but the byte is
    /// increased by 1 after each write
    IncreasingFill = 0b011,

    /// Followed by two bytes (ABCD byte order) containing address (in the
    /// output buffer) to copy (L+1) bytes from
    Repeat         = 0b100,

    /// This command has got a two-byte header:
    /// ```text
    /// 111CCCLL LLLLLLLL
    /// CCC:        Real command
    /// LLLLLLLLLL: Length
    /// ```
    LongLength     = 0b111,
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>, DecompressionError> {
    assert!(!input.is_empty());
    let mut output = Vec::with_capacity(input.len() * 2);
    let mut in_it = input;
    while let Some(chunk_header) = in_it.first().copied() {
        if chunk_header == 0xFF {
            break;
        }
        in_it = &in_it[1..];
        let command = chunk_header >> 5;
        let length = chunk_header & 0b11111;

        let mut command = Command::try_from(command).map_err(|_| LcLz2Error::Command(command))?;
        let mut length = length as usize + 1;

        if let Command::LongLength = command {
            let real_command_bits = (chunk_header >> 2) & 0b111;
            command =
                Command::try_from(real_command_bits).map_err(|_| LcLz2Error::LongLengthCommand(real_command_bits))?;
            let length_part_1 = chunk_header & 0b11;
            let length_part_2 = *in_it.first().ok_or(LcLz2Error::LongLength)?;
            length = (((length_part_1 as usize) << 8) | (length_part_2 as usize)) + 1;
            in_it = &in_it[1..];
        }

        match command {
            Command::DirectCopy => {
                if length <= in_it.len() {
                    let (bytes, rest) = in_it.split_at(length);
                    output.extend_from_slice(bytes);
                    in_it = rest;
                } else {
                    return Err(LcLz2Error::DirectCopy(length).into());
                }
            }
            Command::ByteFill => {
                let byte = *in_it.first().ok_or(LcLz2Error::ByteFill)?;
                output.resize(output.len() + length, byte);
                in_it = &in_it[1..];
            }
            Command::WordFill => {
                if in_it.len() >= 2 {
                    let (bytes, rest) = in_it.split_at(2);
                    output.extend(bytes.iter().cycle().take(length));
                    in_it = rest;
                } else {
                    return Err(LcLz2Error::WordFill.into());
                }
            }
            Command::IncreasingFill => {
                let mut byte = *in_it.first().ok_or(LcLz2Error::IncreasingFill)?;
                output.extend(
                    std::iter::repeat_with(|| {
                        let temp = byte;
                        byte = byte.wrapping_add(1);
                        temp
                    })
                    .take(length),
                );
                in_it = &in_it[1..];
            }
            Command::Repeat => {
                if in_it.len() >= 2 {
                    let bytes;
                    (bytes, in_it) = in_it.split_at(2);
                    let read_start = usize::from(u16::from_be_bytes([bytes[0], bytes[1]]));
                    if read_start >= output.len() {
                        return Err(
                            LcLz2Error::RepeatRangeOutOfBounds(read_start..read_start + length, output.len()).into()
                        );
                    }
                    output.reserve(length);
                    let mut n = length;
                    while n > 0 {
                        let range = read_start..(read_start + n).min(output.len());
                        n -= range.len();
                        output.extend_from_within(range);
                    }
                } else {
                    return Err(LcLz2Error::RepeatIncomplete.into());
                }
            }
            Command::LongLength => return Err(LcLz2Error::DoubleLongLength.into()),
        }
    }

    output.shrink_to_fit();
    Ok(output)
}

#[cfg(test)]
mod tests {
    fn assert_decompression(compressed: &[u8], decompressed: &[u8]) {
        let res = super::decompress(compressed);
        let res = res.unwrap_or_else(|err| panic!("decompression failed unexpectedly ({err})"));
        if res.as_slice() != decompressed {
            panic!("decompression gave wrong results (got: {res:?}, expected: {decompressed:?})")
        }
    }

    #[test]
    fn test_slice_repeat() {
        let compressed = [
            // Insert [1, 2, 3, 4]
            (0b011 << 5) | (4 - 1),
            1,
            // Repeat 7 bytes from address 1
            (0b100 << 5) | (7 - 1),
            0,
            1,
        ];
        assert_decompression(&compressed, &[1, 2, 3, 4, 2, 3, 4, 2, 3, 4, 2]);
    }

    #[test]
    fn test_multiple_repeat_commands_short() {
        const EXPECTED: [u8; 6] = [1, 2, 3, 4, 2, 3];
        for command in [0b100, 0b101, 0b110] {
            let compressed = [
                // Insert [1, 2, 3, 4]
                (0b011 << 5) | (4 - 1),
                1,
                // Repeat 2 bytes from address 1
                (command << 5) | (2 - 1),
            ];
            assert_decompression(&compressed, &EXPECTED)
        }
    }

    #[test]
    fn test_multiple_repeat_commands_long() {
        const EXPECTED: [u8; 6] = [1, 2, 3, 4, 2, 3];
        for command in [0b100, 0b101, 0b110, 0b111] {
            let compressed = [
                // Insert [1, 2, 3, 4]
                (0b011 << 5) | (4 - 1),
                1,
                // Repeat 2 bytes from address 1
                (0b111 << 5) | (command << 2),
                2 - 1,
            ];
            assert_decompression(&compressed, &EXPECTED)
        }
    }
}
