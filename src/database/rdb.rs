use super::{Database, SETTINGS_DBFILENAME_ID, SETTINGS_DIR_ID};
use std::path::PathBuf;

const OP_CODE_EOF: u8 = 0xff;
const OP_CODE_SELECTDB: u8 = 0xfe;
const OP_CODE_EXPIRETIME_S: u8 = 0xfd;
const OP_CODE_EXPIRETIME_MS: u8 = 0xfc;
const OP_CODE_RESIZEDB: u8 = 0x04;
const OP_CODE_AUX: u8 = 0xfa;

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
        let pointer = rdb_bytes;
        let (pointer, version) = read_headers(pointer)?;
        println!("-> Version: {}", version); // TODO: Store version

        let mut pointer = pointer;
        loop {
            let op_code = RdbOpCode::try_from(pointer[0])?;
            pointer = &pointer[1..];

            match op_code {
                RdbOpCode::EOF => break,
                RdbOpCode::Aux => {
                    let (pointer, key) = read_string(&pointer)?;
                    let (pointer, value) = read_string(&pointer)?;
                    println!("-> Key: {}, Value: {}", key, value);
                }
                _ => anyhow::bail!("-> Unknown op code: {:?}", op_code),
            }
        }

        Ok(())
    }
}

/**
 * @return (pointer, version)
 */
fn read_headers(bytes: &[u8]) -> anyhow::Result<(&[u8], u32)> {
    let magic_string = &bytes[..5];
    anyhow::ensure!(magic_string == b"REDIS");
    let bytes = &bytes[5..];

    let version = String::from_utf8_lossy(&bytes[..4]).to_string();
    let version = version.parse::<u32>()?;
    let bytes = &bytes[4..];

    Ok((bytes, version))
}

/**
 * @return (pointer, length)
 */
fn read_length(bytes: &[u8]) -> anyhow::Result<(&[u8], usize)> {
    let mask = 3u8 << 6; // 1100 0000
    let (length, bytes_count) = match bytes[0] & mask {
        0 => (u32::from_be_bytes([0, 0, 0, bytes[0]]), 1),
        64u8 => (
            u32::from_be_bytes([0, 0, bytes[0] & (64u8 - 1), bytes[1]]),
            2,
        ),
        128u8 => (u32::from_be_bytes(bytes[1..5].try_into().unwrap()), 5),
        _ => anyhow::bail!("-> Length encoding not supported"),
    };

    let bytes = &bytes[bytes_count..];
    Ok((bytes, length as usize))
}

/**
 * @return (pointer, string)
 */
fn read_string(bytes: &[u8]) -> anyhow::Result<(&[u8], String)> {
    let (bytes, length) = read_length(bytes)?;
    let string = String::from_utf8_lossy(&bytes[..length]).to_string();

    let bytes = &bytes[length..];
    Ok((bytes, string))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_headers() {
        // Given
        let bytes: &[u8] = &[
            0x52, 0x45, 0x44, 0x49, 0x53, // magic
            0x30, 0x30, 0x30, 0x33, // version
        ];
        // When
        let (pointer, version) = read_headers(bytes).unwrap();
        // Then
        assert_eq!(pointer, &[]);
        assert_eq!(version, 3);
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
