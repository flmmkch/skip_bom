use skip_bom::SkipUtf8Bom;
use std::io::{Cursor, Read};

fn skip_bom_reader_from_bytes<'a>(bytes: &'a [u8]) -> SkipUtf8Bom<Cursor<&'a [u8]>> {
    SkipUtf8Bom::new(Cursor::new(bytes))
}

#[test]
fn test_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBFThis stream has a BOM.");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(true), reader.found_bom());
    assert_eq!("This stream has a BOM.", &string);
}

#[test]
fn test_bom_empty_file() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBF");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(true), reader.found_bom());
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_no_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"This stream has no BOM.");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!("This stream has no BOM.", &string);
}

#[test]
fn test_no_starting_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"This stream has no starting BOM\xEF\xBB\xBF.");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(b"This stream has no starting BOM\xEF\xBB\xBF.", buf.as_slice());
}

#[test]
fn test_small_buffer_1_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBFThis stream has no BOM.");
    let mut small_buf = [0u8; 1];
    // check that we cannot read the BOM bytes
    for i in 0..2 {
        let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
        assert_eq!(None, reader.found_bom(), "byte number {i}");
        assert_eq!(0, bytes_read, "byte number {i}");
    }
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(true), reader.found_bom());
    assert_eq!(1, bytes_read);
    assert_eq!(b"T", small_buf.as_slice());
}

#[test]
fn test_small_buffer_2_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBFThis stream has no BOM.");
    let mut small_buf = [0u8; 2];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(None, reader.found_bom());
    assert_eq!(0, bytes_read);
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(true), reader.found_bom());
    assert_eq!(2, bytes_read, "{:?}", small_buf);
    assert_eq!(b"Th", small_buf.as_slice());
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(true), reader.found_bom());
    assert_eq!(2, bytes_read, "{:?}", small_buf);
    assert_eq!(b"is", small_buf.as_slice());
}

#[test]
fn test_small_buffer_no_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"This stream has no BOM.");
    let mut small_buf = [0u8; 2];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(small_buf.len(), bytes_read);
    assert_eq!(b"Th", small_buf.as_slice());
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(small_buf.len(), bytes_read);
    assert_eq!(b"is", small_buf.as_slice());
}

#[test]
fn test_small_buffer_no_bom_after_start() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBBThis stream has no BOM.");
    let mut small_buf = [0u8; 2];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(None, reader.found_bom());
    assert_eq!(0, bytes_read);
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(2, bytes_read);
    assert_eq!(b"\xEF\xBB", &small_buf);
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(2, bytes_read);
    assert_eq!(b"Th", &small_buf);
    let mut end_buf = Vec::new();
    let _ = reader.read_to_end(&mut end_buf).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(b"is stream has no BOM.", end_buf.as_slice());
}

#[test]
fn test_no_bom_with_bom_length() {
    let mut reader = skip_bom_reader_from_bytes(b"Thi");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!("Thi", &string);
}

#[test]
fn test_no_bom_short() {
    let mut reader = skip_bom_reader_from_bytes(b"Th");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!("Th", &string);
}

#[test]
fn test_bom_short_with_same_start() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEFa");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(false), reader.found_bom());
    assert_eq!(b"\xEFa", buf.as_slice());
}

#[test]
fn test_bom_short() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(None, reader.found_bom());
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_empty_stream() {
    let mut reader = skip_bom_reader_from_bytes(b"");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(None, reader.found_bom());
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}
