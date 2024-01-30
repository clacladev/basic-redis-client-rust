use super::{Database, SETTINGS_DBFILENAME_ID, SETTINGS_DIR_ID};
use crate::database::rdb::{
    op_code::OpCode,
    read_functions::{
        read_auxiliary, read_db_number, read_headers, read_key_value, read_resize_db,
    },
};
use std::path::PathBuf;

mod op_code;
mod read_functions;

#[cfg(test)]
mod tests;

impl Database {
    pub fn can_load_from_disk(&mut self) -> bool {
        let dir = self.config_get(SETTINGS_DIR_ID);
        let dbfilename = self.config_get(SETTINGS_DBFILENAME_ID);
        dir.is_some() && dbfilename.is_some()
    }

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
            let Ok(op_code) = OpCode::try_from(bytes[0]) else {
                let ((key, value), read_count) = read_key_value(&bytes)?;
                bytes = &bytes[read_count..];
                println!("-> Key: {}, Value: {}", key, value);
                self.set(key, value, None)?;
                continue;
            };

            bytes = &bytes[1..];

            match op_code {
                OpCode::EOF => break,
                OpCode::Auxiliary => {
                    let ((key, value), read_count) = read_auxiliary(&bytes)?;
                    bytes = &bytes[read_count..];
                    println!("-> Key: {}, Value: {}", key, value);
                    self.metadata.insert(key, value);
                }
                OpCode::SelectDB => {
                    let (db_number, read_count) = read_db_number(&bytes)?;
                    bytes = &bytes[read_count..];
                    println!("-> DB number: {}", db_number);
                }
                OpCode::ResizeDB => {
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
