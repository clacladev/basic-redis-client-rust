#[cfg(test)]
mod test {
    use crate::database::rdb::read_functions::{
        read_auxiliary, read_headers, read_length, read_number, read_resize_db, read_string,
        ReadLength,
    };

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
    const RESIZE_DB_START: usize = 82;

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
    fn test_read_number_correctly() {
        // Given
        let bytes = &TEST_BYTES[RESIZE_DB_START..];
        // When
        let (number, read_count) = read_number(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(number, 1);

        // Given
        let bytes = &bytes[read_count..];
        // When
        let (number, read_count) = read_number(bytes).unwrap();
        // Then
        assert_eq!(read_count, 1);
        assert_eq!(number, 0);
    }

    // ----------------------------

    #[test]
    fn test_read_headers() {
        // Given
        let bytes = &TEST_BYTES[HEADERS_START..AUX_1_START];
        // When
        let (version, read_count) = read_headers(bytes).unwrap();
        // Then
        assert_eq!(read_count, 9);
        assert_eq!(version, 11);
    }

    #[test]
    fn test_read_metadata() {
        // Given
        let bytes = &TEST_BYTES[AUX_1_START..];
        // When
        let ((key, value), read_count) = read_auxiliary(bytes).unwrap();
        // Then
        assert_eq!(read_count, 16);
        assert_eq!(key, "redis-ver");
        assert_eq!(value, "7.2.4");

        // Given
        let bytes = &TEST_BYTES[AUX_2_START..];
        // When
        let ((key, value), read_count) = read_auxiliary(bytes).unwrap();
        // Then
        assert_eq!(read_count, 13);
        assert_eq!(key, "redis-bits");
        assert_eq!(value, "64");
    }

    #[test]
    fn test_resize_db() {
        // Given
        let bytes = &TEST_BYTES[RESIZE_DB_START..];
        // When
        let ((size_hash_table, size_expiry_hash_table), read_count) =
            read_resize_db(bytes).unwrap();
        // Then
        assert_eq!(read_count, 2);
        assert_eq!(size_hash_table, 1);
        assert_eq!(size_expiry_hash_table, 0);
    }
}
