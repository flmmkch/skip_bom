use skip_bom::{BomType, SkipEncodingBom};
use std::io::{Cursor, Read};

fn skip_bom_reader_from_bytes<'a>(bytes: &'a [u8]) -> SkipEncodingBom<Cursor<&'a [u8]>> {
    SkipEncodingBom::new(Cursor::new(bytes))
}

#[test]
fn test_read_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBFThis stream has a BOM.");
    let mut string = Default::default();
    assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!("This stream has a BOM.", &string);
}

#[test]
fn test_read_bom_empty_file() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBF");
    let mut buf = Default::default();
    assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_read_no_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"This stream has no BOM.");
    let mut string = Default::default();
    assert_eq!(None, reader.read_bom().unwrap());
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!("This stream has no BOM.", &string);
}

#[test]
fn test_read_no_starting_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"This stream has no starting BOM\xEF\xBB\xBF.");
    let mut buf = Default::default();
    assert_eq!(None, reader.read_bom().unwrap());
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(b"This stream has no starting BOM\xEF\xBB\xBF.", buf.as_slice());
}

#[test]
fn test_read_small_buffer_1_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBFThis stream has a BOM.");
    assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
    let mut small_buf = [0u8; 1];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(1, bytes_read);
    assert_eq!(b"T", small_buf.as_slice());
}

#[test]
fn test_read_small_buffer_no_bom_after_start() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBBThis stream has no BOM.");
    assert_eq!(None, reader.read_bom().unwrap());
}

#[test]
fn test_read_no_bom_with_bom_length() {
    let mut reader = skip_bom_reader_from_bytes(b"Thi");
    assert_eq!(None, reader.read_bom().unwrap());
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!("Thi", &string);
}

#[test]
fn test_read_only_bom() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB\xBF");
    assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
    assert_eq!(b"", buf.as_slice());
}

#[test]
fn test_read_no_bom_short() {
    let mut reader = skip_bom_reader_from_bytes(b"Th");
    assert_eq!(None, reader.read_bom().unwrap());
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!("Th", &string);
}

#[test]
fn test_read_bom_short_with_same_start() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEFa");
    assert_eq!(None, reader.read_bom().unwrap());
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(b"\xEFa", buf.as_slice());
}

#[test]
fn test_read_bom_short() {
    let mut reader = skip_bom_reader_from_bytes(b"\xEF\xBB");
    assert_eq!(None, reader.read_bom().unwrap());
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_read_empty_stream() {
    let mut reader = skip_bom_reader_from_bytes(b"");
    assert_eq!(None, reader.read_bom().unwrap());
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}
