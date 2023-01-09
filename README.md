# rust-bitwriter

rust-bitwriter is a Rust library to write bits into a byte vector.

It is intended to complement https://github.com/irauta/bitreader with a writer part. It supports standard signed/unsigned integer types, such as u32, i64, as well as integers of any bit length (up to 64), such as i28.


This example shows how to write a bool, an u28 and an i28 into a byte vector:

    let mut writer = BitWriter::new();

    writer.write_bool(true).expect("failed to write bool");
    writer.write_u32(178956970, 28).expect("failed to write u28");
    writer.write_i32(-22369622, 28).expect("failed to write i28");
    
    writer.close().expect("failed to close byte vector");
    let buffer = writer.data();


This is my first Rust project, so there might be some obvious issues. If you have some suggestions or improvements, please create a PR!
