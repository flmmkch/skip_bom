use skip_bom::*;
use std::io::{Cursor, Read};

fn test_read_with_bom_types(bom_type: BomType, bom_types: &[BomType], found: bool) {
    let mut bytes = Vec::new();
    bytes.extend(bom_type.bom_bytes());
    bytes.extend(b"This stream has a BOM.");
    let result_expected = if found { Some(bom_type) } else { None };
    // with a BOM read at the start
    {
        let mut reader = SkipEncodingBom::new(bom_types, Cursor::new(bytes.as_slice()));
        let mut buf = Default::default();
        assert_eq!(result_expected, reader.read_bom().unwrap());
        let _ = reader.read_to_end(&mut buf).unwrap();
        if found {
            assert_eq!(b"This stream has a BOM.", buf.as_slice());
        }
        else {
            assert_eq!(bytes.as_slice(), buf.as_slice());
        }
        assert_eq!(result_expected, reader.bom_found().unwrap());
    }
    // without a BOM read at the start
    {
        let mut reader = SkipEncodingBom::new(bom_types, Cursor::new(bytes.as_slice()));
        let mut buf = Default::default();
        let _ = reader.read_to_end(&mut buf).unwrap();
        if found {
            assert_eq!(b"This stream has a BOM.", buf.as_slice());
        }
        else {
            assert_eq!(bytes.as_slice(), buf.as_slice());
        }
        assert_eq!(result_expected, reader.bom_found().unwrap());
    }
}

macro_rules! test_read_bom_types_parameter {
    ($test_fn_name:ident, $bom_type:expr, $bom_types:expr, $found:expr) => {
        #[test]
        fn $test_fn_name() {
            test_read_with_bom_types($bom_type, $bom_types, $found);
        }
    };
}

test_read_bom_types_parameter!(test_read_utf8_bom, BomType::UTF8, &[BomType::UTF8], true);
test_read_bom_types_parameter!(test_read_utf16le_bom, BomType::UTF16LE, &[BomType::UTF16LE], true);
test_read_bom_types_parameter!(test_read_utf16be_bom, BomType::UTF16BE, &[BomType::UTF16BE], true);
test_read_bom_types_parameter!(test_read_utf32le_bom, BomType::UTF32LE, &[BomType::UTF32LE], true);
test_read_bom_types_parameter!(test_read_utf32be_bom, BomType::UTF32BE, &[BomType::UTF32BE], true);
test_read_bom_types_parameter!(test_read_utf7_bom, BomType::UTF7, &[BomType::UTF7], true);
test_read_bom_types_parameter!(test_read_utf1_bom, BomType::UTF1, &[BomType::UTF1], true);
test_read_bom_types_parameter!(test_read_utfebdic_bom, BomType::UTFEBDIC, &[BomType::UTFEBDIC], true);
test_read_bom_types_parameter!(test_read_scsu_bom, BomType::SCSU, &[BomType::SCSU], true);
test_read_bom_types_parameter!(test_read_bocu1_bom, BomType::BOCU1, &[BomType::BOCU1], true);
test_read_bom_types_parameter!(test_read_gb1803_bom, BomType::GB1803, &[BomType::GB1803], true);

const ONLY_SOME_BOMS: &'static [BomType] = &[BomType::UTF32LE, BomType::UTF16BE, BomType::UTFEBDIC];

test_read_bom_types_parameter!(test_read_utf8_bom_only_some, BomType::UTF8, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_utf16le_bom_only_some, BomType::UTF16LE, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_utf16be_bom_only_some, BomType::UTF16BE, ONLY_SOME_BOMS, true);
test_read_bom_types_parameter!(test_read_utf32le_bom_only_some, BomType::UTF32LE, ONLY_SOME_BOMS, true);
test_read_bom_types_parameter!(test_read_utf32be_bom_only_some, BomType::UTF32BE, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_utf7_bom_only_some, BomType::UTF7, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_utf1_bom_only_some, BomType::UTF1, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_utfebdic_bom_only_some, BomType::UTFEBDIC, ONLY_SOME_BOMS, true);
test_read_bom_types_parameter!(test_read_scsu_bom_only_some, BomType::SCSU, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_bocu1_bom_only_some, BomType::BOCU1, ONLY_SOME_BOMS, false);
test_read_bom_types_parameter!(test_read_gb1803_bom_only_some, BomType::GB1803, ONLY_SOME_BOMS, false);

// test that UTF-16 Little Endian and UTF-32 Big Endian are not confused
test_read_bom_types_parameter!(test_read_utf16le_utf32be, BomType::UTF32BE, &[BomType::UTF16LE, BomType::UTF32BE], true);
test_read_bom_types_parameter!(test_read_utf32be_utf16le, BomType::UTF32BE, &[BomType::UTF32BE, BomType::UTF16LE], true);
