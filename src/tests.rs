use super::*;

#[test]
fn simple_writing() {
    let mut writer = BitWriter::new();

    writer.write_bool(true).expect("failed to write bool");
    writer
        .write_u32(178956970, 28)
        .expect("failed to write u28");
    writer
        .write_i32(-22369622, 28)
        .expect("failed to write i28");
    assert_eq!(writer.bit_count, 1 + 28 + 28);

    writer.close().expect("failed to close byte vector");
    assert_eq!(writer.bit_count, 64); // should be byte-aligned after closing

    let expected = Vec::<u8>::from([0xD5, 0x55, 0x55, 0x57, 0x55, 0x55, 0x55, 0x00]);
    assert_eq!(writer.data, expected);
}

#[test]
fn test_bitshift_overflow() {
    let mut writer = BitWriter::new();
    writer
        .write_u64(0xFFFFFFFFFFFFFFFF, 64)
        .expect("failed to u64");
    writer.write_u64(0x0, 64).expect("failed to write u64");
    writer.write_i64(0x0, 64).expect("failed to write i64");
    assert_eq!(writer.bit_count, 3 * 64);
    writer.close().expect("failed to close byte vector");
}
