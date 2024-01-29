use super::{Database, SETTINGS_DBFILENAME_ID, SETTINGS_DIR_ID};
use crate::database::rdb::read_functions::{read_auxilliary, read_headers, read_resize_db};
use std::path::PathBuf;

mod read_functions;

const OP_CODE_EOF: u8 = 0xff;
const OP_CODE_SELECTDB: u8 = 0xfe;
const OP_CODE_EXPIRETIME_S: u8 = 0xfd;
const OP_CODE_EXPIRETIME_MS: u8 = 0xfc;
const OP_CODE_RESIZEDB: u8 = 0xfb;
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
            _ => anyhow::bail!("Unknown op code: {:#02X}", val),
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

    fn parse_and_restore_rdb(&mut self, rdb_bytes: &[u8]) -> anyhow::Result<()> {
        let mut bytes = rdb_bytes;
        println!("--> RDB size: {:?}", rdb_bytes.len()); // TODO: remove

        let (version, read_count) = read_headers(bytes)?;
        bytes = &bytes[read_count..];
        println!("-> Version: {}", version);
        self.metadata.insert("version".into(), version.to_string());

        loop {
            println!("--> Pointer position: {:?}", rdb_bytes.len() - bytes.len()); // TODO: remove
            let op_code = RdbOpCode::try_from(bytes[0])?;
            bytes = &bytes[1..];

            match op_code {
                RdbOpCode::EOF => break,
                RdbOpCode::Aux => {
                    let ((key, value), read_count) = read_auxilliary(&bytes)?;
                    bytes = &bytes[read_count..];
                    println!("-> Key: {}, Value: {}", key, value);
                    self.metadata.insert(key, value);
                }
                RdbOpCode::SelectDB => {
                    let db_number = bytes[0];
                    bytes = &bytes[1..];
                    println!("-> DB number: {}", db_number);
                }
                RdbOpCode::ResizeDB => {
                    let ((size_hash_table, size_expiry_hash_table), read_count) =
                        read_resize_db(&bytes)?;
                    bytes = &bytes[read_count..];
                    println!("-> Size hash table: {}", size_hash_table);
                    println!("-> Size expire hash table: {}", size_expiry_hash_table);
                }
                _ => {
                    eprintln!("-> Unmanaged op code: {:?}", op_code);
                    break;
                }
            }
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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

    #[test]
    fn test_parse_and_restore_rdb() {
        // Given
        let mut database = Database::new();
        // When
        database.parse_and_restore_rdb(TEST_BYTES).unwrap();
        // Then
        let mut expected_metadata = HashMap::new();
        expected_metadata.insert("version".into(), "11".into());
        expected_metadata.insert("redis-ver".into(), "7.2.4".into());
        expected_metadata.insert("aof-base".into(), "0".into());
        expected_metadata.insert("redis-bits".into(), "64".into());
        expected_metadata.insert("ctime".into(), "1706281767".into());
        expected_metadata.insert("used-mem".into(), "1148576".into());
        assert_eq!(database.metadata, expected_metadata);

        // TODO: check the rest of the data when parsed
    }
}
