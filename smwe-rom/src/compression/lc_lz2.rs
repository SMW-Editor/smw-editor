use crate::error::{DecompressionError, LcLz2Error};

/// Followed by (L+1) bytes of data
const DIRECT_COPY: u8 = 0b000;

/// Followed by one byte to be repeated (L+1) times
const BYTE_FILL: u8 = 0b001;

/// Followed by two bytes. Output first byte, then second, then first,
/// then second, etc. until (L+1) bytes has been outputted
const WORD_FILL: u8 = 0b010;

/// Followed by one byte to be repeated (L+1) times, but the byte is
/// increased by 1 after each write
const INCREASING_FILL: u8 = 0b011;

/// Followed by two bytes (ABCD byte order) containing address (in the
/// output buffer) to copy (L+1) bytes from
const REPEAT: u8 = 0b100;

/// This command has got a two-byte header:
/// ```text
/// 111CCCLL LLLLLLLL
/// CCC:        Real command
/// LLLLLLLLLL: Length
/// ```
const LONG_LENGTH: u8 = 0b111;

pub fn decompress(input: &[u8], little_endian_in_repeat: bool) -> Result<Vec<u8>, DecompressionError> {
    assert!(!input.is_empty());

    let mut output = Vec::with_capacity(input.len() * 2);
    let mut in_it = input;
    while let Some(chunk_header) = in_it.first().copied() {
        if chunk_header == 0xFF {
            break;
        }
        in_it = &in_it[1..];

        let mut command = chunk_header >> 5;
        let length = match command {
            LONG_LENGTH => {
                command = (chunk_header >> 2) & 0b111;

                if !matches!(command, DIRECT_COPY..=REPEAT | LONG_LENGTH) {
                    return Err(LcLz2Error::LongLengthCommand(command).into());
                }

                let next_byte = *in_it.first().ok_or(LcLz2Error::LongLength)?;
                in_it = &in_it[1..];

                u16::from_le_bytes([next_byte, chunk_header & 3])
            }
            DIRECT_COPY..=REPEAT => u16::from(chunk_header & 0x1F),
            _ => return Err(LcLz2Error::Command(command).into()),
        };

        let length = usize::from(length) + 1;

        match command {
            DIRECT_COPY => {
                if length <= in_it.len() {
                    let (bytes, rest) = in_it.split_at(length);
                    output.extend_from_slice(bytes);
                    in_it = rest;
                } else {
                    return Err(LcLz2Error::DirectCopy(length).into());
                }
            }
            BYTE_FILL => {
                let byte = *in_it.first().ok_or(LcLz2Error::ByteFill)?;
                output.resize(output.len() + length, byte);
                in_it = &in_it[1..];
            }
            WORD_FILL => {
                if in_it.len() >= 2 {
                    let (bytes, rest) = in_it.split_at(2);
                    output.extend(bytes.iter().cycle().take(length));
                    in_it = rest;
                } else {
                    return Err(LcLz2Error::WordFill.into());
                }
            }
            INCREASING_FILL => {
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
            REPEAT => {
                if in_it.len() >= 2 {
                    let (bytes, rest) = in_it.split_at(2);
                    let from_bytes = if little_endian_in_repeat { u16::from_le_bytes } else { u16::from_be_bytes };
                    let read_start = usize::from(from_bytes([bytes[0], bytes[1]]));
                    let read_range = read_start..read_start + length;
                    if read_start >= output.len() {
                        return Err(LcLz2Error::RepeatRangeOutOfBounds(read_range, output.len()).into());
                    } else {
                        output.reserve(length);
                        for i in read_range {
                            output.push(output[i]);
                        }
                    }
                    in_it = rest;
                } else {
                    return Err(LcLz2Error::RepeatIncomplete.into());
                }
            }
            _ => unreachable!(),
        }
    }

    output.shrink_to_fit();
    Ok(output)
}

#[cfg(test)]
mod tests {
    fn assert_decompression(compressed: &[u8], decompressed: &[u8]) {
        let res = super::decompress(compressed, false);
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
                0,
                1,
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
                0,
                1,
            ];
            assert_decompression(&compressed, &EXPECTED)
        }
    }
}
