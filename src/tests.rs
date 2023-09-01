use super::*;
use std::vec;

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

#[test]
fn test_byte_writing() {
    let mut writer = BitWriter::new();

    // First, test writing bytes aligned
    writer
        .write(&vec![0xFF, 0x22, 0x00, 0x12])
        .expect("failed to write bytes aligned");
    assert_eq!(writer.bit_count, 4 * 8);

    // Make the buffer unaligned
    writer.write_bool(true).expect("failed to write boolean");
    assert_eq!(writer.bit_count, 4 * 8 + 1);
    writer
        .write(&vec![0xFF, 0x22, 0x00, 0x12])
        .expect("failed to write bytes unaligned");
    assert_eq!(writer.bit_count, 8 * 8 + 1);

    // Align to byte boundary again
    writer.align(1).expect("failed to align bit stream");
    assert_eq!(writer.bit_count, 9 * 8);
    writer
        .write(&vec![0xFF])
        .expect("failed to write bytes aligned");
    assert_eq!(writer.bit_count, 10 * 8);

    writer.close().expect("failed to close byte vector");
}
