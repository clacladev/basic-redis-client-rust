const READ_LENGTH_TYPE_6BIT: u8 = 0b00;
const READ_LENGTH_TYPE_14BIT: u8 = 0b01;
const READ_LENGTH_TYPE_32BIT: u8 = 0b10;
const READ_LENGTH_TYPE_SPECIAL: u8 = 0b11;

#[derive(Debug, PartialEq)]
pub enum ReadLength {
    Number(usize),
    Special(usize),
}

type ReadResult<T> = anyhow::Result<(T, usize)>;

fn read_length(bytes: &[u8]) -> ReadResult<ReadLength> {
    let kind = bytes[0] >> 6;
    let b0 = bytes[0] & 0b0011_1111;

    match kind {
        READ_LENGTH_TYPE_6BIT => Ok((ReadLength::Number(b0 as usize), 1)),
        READ_LENGTH_TYPE_14BIT => Ok((
            ReadLength::Number(((b0 as usize) << 8) | (bytes[1] as usize)),
            2,
        )),
        READ_LENGTH_TYPE_32BIT => Ok((
            ReadLength::Number(
                u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize
            ),
            5,
        )),
        READ_LENGTH_TYPE_SPECIAL => match b0 {
            0 => Ok((ReadLength::Special(1 as usize), 1)),
            1 => Ok((ReadLength::Special(2 as usize), 1)),
            2 => Ok((ReadLength::Special(4 as usize), 1)),
            _ => anyhow::bail!("-> Length encoding not supported. Special kind id: {}", b0),
        },
        _ => anyhow::bail!("-> Length encoding not supported. Kind: {}", kind),
    }
}

fn read_string(bytes: &[u8]) -> ReadResult<String> {
    let (read_length, read_count_length) = read_length(bytes)?;
    let bytes = &bytes[read_count_length..];

    let (string, read_count_string) = match read_length {
        ReadLength::Number(length) => (
            String::from_utf8_lossy(&bytes[..length]).to_string(),
            length,
        ),
        ReadLength::Special(length) => match length {
            1 => (bytes[0].to_string(), 1),
            2 => (u16::from_le_bytes([bytes[0], bytes[1]]).to_string(), 2),
            4 => (
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_string(),
                4,
            ),
            _ => anyhow::bail!("-> Int length not supported. Length: {}", length),
        },
    };

    let read_count = read_count_length + read_count_string;
    Ok((string, read_count))
}

// fn read_number(bytes: &[u8]) -> ReadResult<u32> {
//     let (read_length, read_count_length) = read_length(bytes)?;
//     let bytes = &bytes[read_count_length..];

//     let (number, read_count_number) = match read_length {
//         ReadLength::Number(length) | ReadLength::Special(length) => match length {
//             1 => (u32::from_le_bytes([0u8, 0u8, 0u8, bytes[0]]), 1),
//             2 => (u32::from_le_bytes([0u8, 0u8, bytes[0], bytes[1]]), 2),
//             4 => (
//                 u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
//                 4,
//             ),
//             _ => anyhow::bail!("-> Int length not supported. Length: {}", length),
//         },
//     };

//     let read_count = read_count_length + read_count_number;
//     Ok((number, read_count))
// }

pub fn read_headers(bytes: &[u8]) -> ReadResult<u32> {
    const MAGIC_STRING_LENGTH: usize = 5;
    let magic_string = &bytes[..MAGIC_STRING_LENGTH];
    anyhow::ensure!(magic_string == b"REDIS");
    let bytes = &bytes[MAGIC_STRING_LENGTH..];

    const VERSION_LENGTH: usize = 4;
    let version = String::from_utf8_lossy(&bytes[..VERSION_LENGTH]).to_string();
    let version = version.parse::<u32>()?;

    let read_count = MAGIC_STRING_LENGTH + VERSION_LENGTH;
    Ok((version, read_count))
}

pub fn read_auxilliary(bytes: &[u8]) -> ReadResult<(String, String)> {
    let (key, read_count_key) = read_string(&bytes)?;
    let bytes = &bytes[read_count_key..];
    let (value, read_count_value) = read_string(&bytes)?;
    let read_count = read_count_key + read_count_value;
    Ok(((key, value), read_count))
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_BYTES: &[u8] = &[
        0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31, 0xfa, 0x09, 0x72, 0x65, 0x64, 0x69,
        0x73, 0x2d, 0x76, 0x65, 0x72, 0x05, 0x37, 0x2e, 0x32, 0x2e, 0x34, 0xfa, 0x0a, 0x72, 0x65,
        0x64, 0x69, 0x73, 0x2d, 0x62, 0x69, 0x74, 0x73, 0xc0, 0x40, 0xfa, 0x05, 0x63, 0x74, 0x69,
        0x6d, 0x65, 0xc2, 0x27, 0xcb, 0xb3, 0x65, 0xfa, 0x08, 0x75, 0x73, 0x65, 0x64, 0x2d, 0x6d,
        0x65, 0x6d, 0xc2, 0xa0, 0x86, 0x11, 0x00, 0xfa, 0x08, 0x61, 0x6f, 0x66, 0x2d, 0x62, 0x61,
        0x73, 0x65, 0xc0, 0x00, 0xfe, 0x00, 0xfb, 0x01, 0x00, 0x00, 0x05, 0x6d, 0x79, 0x6b, 0x65,
        0x79, 0x05, 0x6d, 0x79, 0x76, 0x61, 0x6c, 0xff, 0x3d, 0x30, 0xa8, 0x7a, 0xcf, 0x3e, 0x03,
        0x9a,
    ];

    // const TEST_BYTES: &[u8] = &[
    //     0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31, // header
    //     0xfa, 0x09, 0x72, 0x65, 0x64, 0x69, // aux
    //     0x73, 0x2d, 0x76, 0x65, 0x72, 0x05, 0x37, 0x2e, 0x32, 0x2e, 0x34, 0xfa, 0x0a, 0x72, 0x65,
    //     0x64, 0x69, 0x73, 0x2d, 0x62, 0x69, 0x74, 0x73, 0xc0, 0x40, 0xfa, 0x05, 0x63, 0x74, 0x69,
    //     0x6d, 0x65, 0xc2, 0x27, 0xcb, 0xb3, 0x65, 0xfa, 0x08, 0x75, 0x73, 0x65, 0x64, 0x2d, 0x6d,
    //     0x65, 0x6d, 0xc2, 0xa0, 0x86, 0x11, 0x00, 0xfa, 0x08, 0x61, 0x6f, 0x66, 0x2d, 0x62, 0x61,
    //     0x73, 0x65, 0xc0,
    //     0x00, // select db
    //     0xfe, 0x00, 0xfb, 0x01, 0x00, 0x00, 0x05, 0x6d, 0x79, 0x6b, 0x65,
    //     0x79, 0x05, 0x6d, 0x79, 0x76, 0x61, 0x6c, 0xff, 0x3d, 0x30, 0xa8, 0x7a, 0xcf, 0x3e, 0x03,
    //     0x9a,
    // ];

    const HEADERS_START: usize = 0;
    const AUX_1_START: usize = 10;
    const AUX_2_START: usize = 27;

    #[test]
    fn test_read_length_returns_read_length_number() {
        // Given
        let bytes = &TEST_BYTES[AUX_1_START..];
        // When
        let (read_type, read_count) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadLength::Number(9));

        // Given
        let bytes = &bytes[read_count + 9..];
        // When
        let (read_type, read_count) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadLength::Number(5));
    }

    #[test]
    fn test_read_length_returns_read_type_special() {
        // Given
        let bytes = &TEST_BYTES[AUX_2_START..];
        // When
        let (read_type, read_count) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadLength::Number(10));

        // Given
        let bytes = &bytes[1 + 10..];
        // When
        let (read_type, read_count) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadLength::Special(1));
    }

    #[test]
    fn test_read_string_reads_correctly_redis_ver() {
        // Given
        let bytes = &TEST_BYTES[AUX_1_START..];
        // When
        let (key, read_count) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 10);
        assert_eq!(key, "redis-ver");

        // Given
        let bytes = &bytes[read_count..];
        // When
        let (value, read_count) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 6);
        assert_eq!(value, "7.2.4");
    }

    #[test]
    fn test_read_string_reads_correctly_redis_bits() {
        // Given
        let bytes = &TEST_BYTES[AUX_2_START..];
        // When
        let (key, read_count) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 11);
        assert_eq!(key, "redis-bits");

        // Given
        let bytes = &bytes[read_count..];
        // When
        let (value, read_count) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 2);
        assert_eq!(value, "64");
    }

    #[test]
    fn test_read_headers() {
        // Given
        let bytes = &TEST_BYTES[HEADERS_START..AUX_1_START];
        // When
        let (version, read_count) = read_headers(bytes).unwrap();
        // Then
        assert_eq!(read_count, 11);
        assert_eq!(version, 9);
    }

    #[test]
    fn test_read_metadata() {
        // Given
        let bytes = &TEST_BYTES[AUX_1_START..];
        // When
        let ((key, value), read_count) = read_auxilliary(bytes).unwrap();
        // Then
        assert_eq!(read_count, 16);
        assert_eq!(key, "redis-ver");
        assert_eq!(value, "7.2.4");

        // Given
        let bytes = &TEST_BYTES[AUX_2_START..];
        // When
        let ((key, value), read_count) = read_auxilliary(bytes).unwrap();
        // Then
        assert_eq!(read_count, 13);
        assert_eq!(key, "redis-bits");
        assert_eq!(value, "64");
    }
}
