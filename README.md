# rust-bitwriter

`rust-bitwriter` is a rust crate to write data into a bit stream.

![tests](https://github.com/Danaozhong/rust-bitwriter/actions/workflows/test.yml/badge.svg)

This example shows how to write a `bool`, an `u28` and an `i28` into a byte vector:

```rust
    let mut writer = BitWriter::new();

    writer.write_bool(true).expect("failed to write bool");
    writer.write_u32(178956970, 28).expect("failed to write u28");
    writer.write_i32(-22369622, 28).expect("failed to write i28");
    
    writer.close().expect("failed to close byte vector");
    let buffer = writer.data();
```

You can write signed and unsigned integers with 1-64 bit length.

It is intended to complement [irauta/bitreader](https://github.com/irauta/bitreader) with a writer component.


This is my first rust project, so there might be some issues. If you have some suggestions or improvements, please feel welcomed to raise a PR!

## License

Licensed under the Apache License, Version 2.0 or the MIT license, at your option.