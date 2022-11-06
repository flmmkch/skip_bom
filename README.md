# skip_bom

Skip the optional encoding BOM at the start of a file if it exists.
As of now, only the UTF-8 BOM is supported.

## Examples

```rust
use skip_bom::{BomType, SkipEncodingBom};
use std::io::{Cursor, Read};

// Read a stream after checking that it starts with the BOM
const BOM_BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
let mut reader = SkipEncodingBom::new(Cursor::new(BOM_BYTES));
assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
let mut string = Default::default();
let _ = reader.read_to_string(&mut string).unwrap();
assert_eq!("This stream starts with a UTF-8 BOM.", &string);

// Read a stream without a starting BOM
const BOM_BYTES: &'static [u8] = b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.";
let mut reader = SkipEncodingBom::new(Cursor::new(BOM_BYTES));
assert_eq!(None, reader.read_bom().unwrap());
let mut buf = Default::default();
let _ = reader.read_to_end(&mut buf).unwrap();
assert_eq!(b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.", buf.as_slice());

// Read a stream and disregard the starting BOM completely
const BOM_BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
let mut reader = SkipEncodingBom::new(Cursor::new(BOM_BYTES));
let mut buf = Default::default();
let _ = reader.read_to_end(&mut buf).unwrap();
assert_eq!(b"This stream starts with a UTF-8 BOM.", buf.as_slice());
// Check the BOM after the read is over.
assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
```

## References

* [The official Unicode FAQ](https://www.unicode.org/faq/utf_bom.html).

## Documentation

[Module documentation with examples](https://docs.rs/skip_bom).

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.
