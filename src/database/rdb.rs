use super::{Database, SETTINGS_DBFILENAME_ID, SETTINGS_DIR_ID};
use std::path::PathBuf;

const OP_CODE_EOF: u8 = 0xff;
const OP_CODE_SELECTDB: u8 = 0xfe;
const OP_CODE_EXPIRETIME_S: u8 = 0xfd;
const OP_CODE_EXPIRETIME_MS: u8 = 0xfc;
const OP_CODE_RESIZEDB: u8 = 0x04;
const OP_CODE_AUX: u8 = 0xfa;

const STRING_LENGTH_TYPE_6BIT: u8 = 0b00;
const STRING_LENGTH_TYPE_14BIT: u8 = 0b01;
const STRING_LENGTH_TYPE_32BIT: u8 = 0b10;
const STRING_LENGTH_TYPE_SPECIAL: u8 = 0b11;

#[derive(Debug, PartialEq)]
enum ReadType {
    String(usize),
    Int(usize),
}

#[derive(Debug)]
enum RdbOpCode {
    EOF,
    SelectDB,
    ExpireTimeS,
    ExpireTimeMS,
    ResizeDB,
    Aux,
}

impl TryFrom<u8> for RdbOpCode {
    type Error = anyhow::Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            OP_CODE_EOF => Ok(Self::EOF),
            OP_CODE_SELECTDB => Ok(Self::SelectDB),
            OP_CODE_EXPIRETIME_S => Ok(Self::ExpireTimeS),
            OP_CODE_EXPIRETIME_MS => Ok(Self::ExpireTimeMS),
            OP_CODE_RESIZEDB => Ok(Self::ResizeDB),
            OP_CODE_AUX => Ok(Self::Aux),
            _ => anyhow::bail!("Unknown op code: {}", val),
        }
    }
}

impl Database {
    pub fn load_from_disk(&mut self) -> anyhow::Result<()> {
        let Some(dir) = self.config_get(SETTINGS_DIR_ID) else {
            anyhow::bail!("Failed to get dir")
        };
        let Some(dbfilename) = self.config_get(SETTINGS_DBFILENAME_ID) else {
            anyhow::bail!("Failed to get dbfilename")
        };

        let mut dbpath = PathBuf::new();
        dbpath.push(dir);
        dbpath.push(dbfilename);

        let rdb_bytes = std::fs::read(&dbpath)?;
        self.parse_and_restore_rdb(&rdb_bytes)?;

        Ok(())
    }

    fn parse_and_restore_rdb(&self, rdb_bytes: &[u8]) -> anyhow::Result<()> {
        let mut pointer = rdb_bytes;
        println!("--> RDB size: {:?}", pointer.len()); // TODO: remove
        let (read_count, version) = read_headers(pointer)?;
        pointer = &pointer[read_count..];
        println!("-> Version: {}", version); // TODO: Store version

        loop {
            println!("--> Pointer size: {:?}", pointer.len()); // TODO: remove
            let op_code = RdbOpCode::try_from(pointer[0])?;
            pointer = &pointer[1..];

            match op_code {
                RdbOpCode::EOF => break,
                RdbOpCode::Aux => {
                    let (read_count, key) = read_string(&pointer)?;
                    pointer = &pointer[read_count..];
                    let (read_count, value) = read_string(&pointer)?;
                    pointer = &pointer[read_count..];
                    println!("-> Key: {}, Value: {}", key, value);
                }
                _ => anyhow::bail!("-> Unknown op code: {:?}", op_code),
            }
        }

        Ok(())
    }
}

/**
 * @return (read_count, version)
 */
fn read_headers(bytes: &[u8]) -> anyhow::Result<(usize, u32)> {
    const MAGIC_STRING_LENGTH: usize = 5;
    let magic_string = &bytes[..MAGIC_STRING_LENGTH];
    anyhow::ensure!(magic_string == b"REDIS");
    let bytes = &bytes[MAGIC_STRING_LENGTH..];

    const VERSION_LENGTH: usize = 4;
    let version = String::from_utf8_lossy(&bytes[..VERSION_LENGTH]).to_string();
    let version = version.parse::<u32>()?;

    let read_count = MAGIC_STRING_LENGTH + VERSION_LENGTH;
    Ok((read_count, version))
}

/**
 * @return (read_count, read_type)
 */
fn read_length(bytes: &[u8]) -> anyhow::Result<(usize, ReadType)> {
    let kind = bytes[0] >> 6;
    let b0 = bytes[0] & 0b0011_1111;

    match kind {
        STRING_LENGTH_TYPE_6BIT => Ok((1, ReadType::String(b0 as usize))),
        STRING_LENGTH_TYPE_14BIT => Ok((
            2,
            ReadType::String(((b0 as usize) << 8) | (bytes[1] as usize)),
        )),
        STRING_LENGTH_TYPE_32BIT => Ok((
            5,
            ReadType::String(u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize),
        )),
        STRING_LENGTH_TYPE_SPECIAL => match b0 {
            0 => Ok((1, ReadType::Int(1 as usize))),
            1 => Ok((1, ReadType::Int(2 as usize))),
            2 => Ok((1, ReadType::Int(4 as usize))),
            _ => anyhow::bail!("-> Length encoding not supported. Special kind id: {}", b0),
        },
        _ => anyhow::bail!("-> Length encoding not supported. Kind: {}", kind),
    }
}

/**
 * @return (read_count, string)
 */
fn read_string(bytes: &[u8]) -> anyhow::Result<(usize, String)> {
    let (read_count_length, reat_type) = read_length(bytes)?;
    let bytes = &bytes[read_count_length..];

    let (read_count_string, string) = match reat_type {
        ReadType::String(length) => (
            length,
            String::from_utf8_lossy(&bytes[..length]).to_string(),
        ),
        ReadType::Int(length) => match length {
            1 => (1, bytes[0].to_string()),
            2 => (2, u16::from_le_bytes([bytes[0], bytes[1]]).to_string()),
            4 => (
                4,
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_string(),
            ),
            _ => anyhow::bail!("-> Int length not supported. Length: {}", length),
        },
    };

    let read_count = read_count_length + read_count_string;
    Ok((read_count, string))
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
    fn test_read_headers() {
        // Given
        let bytes = &TEST_BYTES[HEADERS_START..AUX_1_START];
        // When
        let (read_count, version) = read_headers(bytes).unwrap();
        // Then
        assert_eq!(read_count, 9);
        assert_eq!(version, 11);
    }

    #[test]
    fn test_read_length_returns_read_type_string() {
        // Given
        let bytes = &TEST_BYTES[AUX_1_START..];
        // When
        let (read_count, read_type) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadType::String(9));

        // Given
        let bytes = &bytes[read_count + 9..];
        // When
        let (read_count, read_type) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadType::String(5));
    }

    #[test]
    fn test_read_length_returns_read_type_int() {
        // Given
        let bytes = &TEST_BYTES[AUX_2_START..];
        // When
        let (read_count, read_type) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadType::String(10));

        // Given
        let bytes = &bytes[1 + 10..];
        // When
        let (read_count, read_type) = read_length(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(read_type, ReadType::Int(1));
    }

    #[test]
    fn test_read_string_reads_correctly_redis_ver() {
        // Given
        let bytes = &TEST_BYTES[AUX_1_START..];
        // When
        let (read_count, key) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 10);
        assert_eq!(key, "redis-ver");

        // Given
        let bytes = &bytes[read_count..];
        // When
        let (read_count, value) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 6);
        assert_eq!(value, "7.2.4");
    }

    #[test]
    fn test_read_string_reads_correctly_redis_bits() {
        // Given
        let bytes = &TEST_BYTES[AUX_2_START..];
        // When
        let (read_count, key) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 11);
        assert_eq!(key, "redis-bits");

        // Given
        let bytes = &bytes[read_count..];
        // When
        let (read_count, value) = read_string(bytes).unwrap();
        // Then
        assert_eq!(read_count, 2);
        assert_eq!(value, "64");
    }
}

/*
Example RDB file:

----------------------------#
52 45 44 49 53              # Magic String "REDIS"
30 30 30 33                 # RDB Version Number as ASCII string. "0003" = 3
----------------------------
FA                          # Auxiliary field
$string-encoded-key         # May contain arbitrary metadata
$string-encoded-value       # such as Redis version, creation time, used memory, ...
----------------------------
FE 00                       # Indicates database selector. db number = 00
FB                          # Indicates a resizedb field
$length-encoded-int         # Size of the corresponding hash table
$length-encoded-int         # Size of the corresponding expire hash table
----------------------------# Key-Value pair starts
FD $unsigned-int            # "expiry time in seconds", followed by 4 byte unsigned int
$value-type                 # 1 byte flag indicating the type of value
$string-encoded-key         # The key, encoded as a redis string
$encoded-value              # The value, encoding depends on $value-type
----------------------------
FC $unsigned long           # "expiry time in ms", followed by 8 byte unsigned long
$value-type                 # 1 byte flag indicating the type of value
$string-encoded-key         # The key, encoded as a redis string
$encoded-value              # The value, encoding depends on $value-type
----------------------------
$value-type                 # key-value pair without expiry
$string-encoded-key
$encoded-value
----------------------------
FE $length-encoding         # Previous db ends, next db starts.
----------------------------
...                         # Additional key-value pairs, databases, ...

FF                          ## End of RDB file indicator
8-byte-checksum             ## CRC64 checksum of the entire file.
*/
