use skip_bom::BomType;
use std::io::{Cursor, Read};

mod utf8_bom_test_utils;
use utf8_bom_test_utils::*;

#[test]
fn test_skip_bom() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEF\xBB\xBFThis stream has a BOM.");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
    assert_eq!("This stream has a BOM.", &string);
}

#[test]
fn test_skip_bom_empty_file() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEF\xBB\xBF");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_skip_no_bom() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"This stream has no BOM.");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!("This stream has no BOM.", &string);
}

#[test]
fn test_skip_no_starting_bom() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"This stream has no starting BOM\xEF\xBB\xBF.");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(b"This stream has no starting BOM\xEF\xBB\xBF.", buf.as_slice());
}

#[test]
fn test_skip_small_buffer_1_bom() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEF\xBB\xBFThis stream has a BOM.");
    let mut small_buf = [0u8; 1];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
    assert_eq!(1, bytes_read);
    assert_eq!(b"T", small_buf.as_slice());
}

#[test]
fn test_skip_small_buffer_2_bom() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEF\xBB\xBFThis stream has a BOM.");
    let mut small_buf = [0u8; 2];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
    assert_eq!(2, bytes_read, "{:?}", small_buf);
    assert_eq!(b"Th", small_buf.as_slice());
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
    assert_eq!(2, bytes_read, "{:?}", small_buf);
    assert_eq!(b"is", small_buf.as_slice());
}

#[test]
fn test_skip_small_buffer_no_bom() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"This stream has no BOM.");
    let mut small_buf = [0u8; 2];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(small_buf.len(), bytes_read);
    assert_eq!(b"Th", small_buf.as_slice());
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(small_buf.len(), bytes_read);
    assert_eq!(b"is", small_buf.as_slice());
}

#[test]
fn test_skip_small_buffer_no_bom_after_start() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEF\xBBThis stream has no BOM.");
    let mut small_buf = [0u8; 2];
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(2, bytes_read);
    assert_eq!(b"\xEF\xBB", &small_buf);
    let bytes_read = reader.read(small_buf.as_mut_slice()).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(2, bytes_read);
    assert_eq!(b"Th", &small_buf);
    let mut end_buf = Vec::new();
    let _ = reader.read_to_end(&mut end_buf).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(b"is stream has no BOM.", end_buf.as_slice());
}

#[test]
fn test_skip_no_bom_with_bom_length() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"Thi");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!("Thi", &string);
}

#[test]
fn test_skip_no_bom_short() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"Th");
    let mut string = Default::default();
    let _ = reader.read_to_string(&mut string).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!("Th", &string);
}

#[test]
fn test_skip_bom_short_with_same_start() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEFa");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(Some(None), reader.bom_found());
    assert_eq!(b"\xEFa", buf.as_slice());
}

#[test]
fn test_skip_bom_short() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"\xEF\xBB");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(None, reader.bom_found());
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_skip_empty_stream() {
    let mut reader = skip_utf8_bom_reader_from_byte_slice(b"");
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(None, reader.bom_found());
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
}

#[test]
fn test_skip_bom_progressive() {
    let mut reader = skip_utf8_bom_reader(Cursor::new(b"\xEF\xBB".to_vec()));
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
    assert_eq!(None, reader.bom_found());
    reader.get_mut().get_mut().extend_from_slice(b"\xBFThis stream has a BOM.");
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(b"This stream has a BOM.", buf.as_slice());
    assert_eq!(Some(BomType::UTF8), reader.bom_found().unwrap());
}

#[test]
fn test_skip_no_bom_progressive() {
    let mut reader = skip_utf8_bom_reader(Cursor::new(b"\xEF\xBB".to_vec()));
    let mut buf = Default::default();
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(0, buf.len(), "{:?}", buf.as_slice());
    assert_eq!(None, reader.bom_found());
    reader.get_mut().get_mut().extend_from_slice(b"This stream has no BOM actually.");
    let _ = reader.read_to_end(&mut buf).unwrap();
    assert_eq!(b"\xEF\xBBThis stream has no BOM actually.", buf.as_slice());
    assert_eq!(None, reader.bom_found().unwrap());
}
