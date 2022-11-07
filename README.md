# skip_bom

[![Build status](https://github.com/flmmkch/skip_bom/workflows/ci/badge.svg)](https://github.com/flmmkch/skip_bom/actions)
[![Rust](https://img.shields.io/badge/rust-1.57.0%2B-blue.svg?maxAge=3600)](https://github.com/flmmkch/skip_bom)
[![crates.io](https://img.shields.io/crates/v/skip_bom.svg)](https://crates.io/crates/skip_bom)
[![docs.rs](https://img.shields.io/docsrs/skip_bom?maxAge=3600)](https://docs.rs/skip_bom)

Skip the optional encoding BOM (Byte Order Mark) at the start of an I/O stream if it exists.

The `SkipEncodingBom` data structure does not make any dynamic allocations and supports progressive stream reads.

A list of supported BOMs can be found [in the crate documentation](https://docs.rs/skip_bom/*/skip_bom/enum.BomType.html).

## Examples

```rust
use skip_bom::{BomType, SkipEncodingBom};
use std::io::{Cursor, Read};

// Read a stream after checking that it starts with the BOM
const BOM_BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BOM_BYTES));
assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
let mut string = Default::default();
let _ = reader.read_to_string(&mut string).unwrap();
assert_eq!("This stream starts with a UTF-8 BOM.", &string);

// Read a stream without a starting BOM
const NO_BOM_BYTES: &'static [u8] = b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.";
let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(NO_BOM_BYTES));
assert_eq!(None, reader.read_bom().unwrap());
let mut buf = Default::default();
let _ = reader.read_to_end(&mut buf).unwrap();
assert_eq!(b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.", buf.as_slice());

// Read a stream and disregard the starting BOM completely
let mut reader = SkipEncodingBom::new(&[BomType::UTF8], Cursor::new(BOM_BYTES));
let mut buf = Default::default();
let _ = reader.read_to_end(&mut buf).unwrap();
assert_eq!(b"This stream starts with a UTF-8 BOM.", buf.as_slice());
// Check the BOM after the read is over.
assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
```

### Progressive reads

This crate supports I/O streams that are incomplete at first and receive data later, even for the initial BOM. Example:

```rust
use skip_bom::{BomType, SkipEncodingBom};
use std::io::{Cursor, Read};

let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(b"\xEF\xBB".to_vec()));
let mut buf = Default::default();
let _ = reader.read_to_end(&mut buf).unwrap();
// The stream is incomplete: there are only the first two bytes of the BOM yet
assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
assert_eq!(None, reader.bom_found());
// Add the next bytes and check that the UTF-8 BOM is accounted for
reader.get_mut().get_mut().extend_from_slice(b"\xBFThis stream has a BOM.");
let _ = reader.read_to_end(&mut buf).unwrap();
assert_eq!(b"This stream has a BOM.", buf.as_slice());
assert_eq!(Some(BomType::UTF8), reader.bom_found().unwrap());
```

## References

* [The official Unicode FAQ](https://www.unicode.org/faq/utf_bom.html)
* [Byte order mark on Wikipedia](https://en.wikipedia.org/wiki/Byte_order_mark#Byte_order_marks_by_encoding)

## Documentation

[Module documentation with examples](https://docs.rs/skip_bom)

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.
