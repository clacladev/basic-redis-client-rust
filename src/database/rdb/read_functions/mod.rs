use crate::database::Entry;

use self::value_type::ValueType;

#[cfg(test)]
mod tests;

mod value_type;

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

fn read_number(bytes: &[u8]) -> ReadResult<u32> {
    let kind = bytes[0] >> 6;
    let b0 = bytes[0] & 0b0011_1111;

    match kind {
        READ_LENGTH_TYPE_6BIT => Ok((b0 as u32, 1)),
        READ_LENGTH_TYPE_14BIT => Ok((((b0 as u32) << 8) | (bytes[1] as u32), 2)),
        READ_LENGTH_TYPE_32BIT => Ok((
            u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
            5,
        )),
        READ_LENGTH_TYPE_SPECIAL => match b0 {
            0 => Ok((bytes[1] as u32, 2)),
            1 => Ok((u16::from_le_bytes([bytes[1], bytes[2]]) as u32, 3)),
            2 => Ok((
                u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
                5,
            )),
            _ => anyhow::bail!("-> Length encoding not supported. Special kind id: {}", b0),
        },
        _ => anyhow::bail!("-> Length encoding not supported. Kind: {}", kind),
    }
}

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

pub fn read_db_number(bytes: &[u8]) -> ReadResult<u32> {
    Ok(read_number(&bytes)?)
}

pub fn read_auxiliary(bytes: &[u8]) -> ReadResult<(String, String)> {
    let (key, read_count_key) = read_string(&bytes)?;
    let bytes = &bytes[read_count_key..];
    let (value, read_count_value) = read_string(&bytes)?;
    let read_count = read_count_key + read_count_value;
    Ok(((key, value), read_count))
}

pub fn read_resize_db(bytes: &[u8]) -> ReadResult<(u32, u32)> {
    let (size_hash_table, read_count_hash_table) = read_number(&bytes)?;
    let bytes = &bytes[read_count_hash_table..];
    let (size_expiry_hash_table, read_count_expiry_hash_table) = read_number(&bytes)?;
    let read_count = read_count_hash_table + read_count_expiry_hash_table;
    Ok(((size_hash_table, size_expiry_hash_table), read_count))
}

pub fn read_key_value(bytes: &[u8]) -> ReadResult<(String, String)> {
    let value_type = ValueType::try_from(bytes[0])?;
    let bytes = &bytes[1..];

    match value_type {
        ValueType::String => {
            let (key, read_count_key) = read_string(&bytes)?;
            let bytes = &bytes[read_count_key..];
            let (value, read_count_value) = read_string(&bytes)?;
            let read_count = read_count_key + read_count_value + 1;
            Ok(((key, value), read_count))
        }
    }
}

pub fn read_key_value_with_ms_expiry(bytes: &[u8]) -> ReadResult<(String, Entry)> {
    let expiry_ms = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let bytes = &bytes[8..];

    let ((key, value), read_count) = read_key_value(&bytes)?;
    let read_count = read_count + 8;

    let entry = Entry {
        value,
        expires_at: Some(expiry_ms as u128),
    };

    Ok(((key, entry), read_count))
}
