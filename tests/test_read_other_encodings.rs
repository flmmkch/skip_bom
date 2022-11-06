use skip_bom::*;
use std::io::{Cursor, Read};

macro_rules! test_read_bom_type {
    ($test_fn_name:ident, $bom:expr) => {
        #[test]
        fn $test_fn_name() {
            let mut bytes = Vec::new();
            bytes.extend($bom.bom_bytes());
            bytes.extend(b"This stream has a BOM.");
            // with a BOM read at the start
            {
                let mut reader = SkipEncodingBom::new(Cursor::new(bytes.as_slice()));
                let mut buf = Default::default();
                assert_eq!(Some($bom), reader.read_bom().unwrap());
                let _ = reader.read_to_end(&mut buf).unwrap();
                assert_eq!(b"This stream has a BOM.", buf.as_slice());
                assert_eq!(Some($bom), reader.bom_found().unwrap());
            }
            // without a BOM read at the start
            {
                let mut reader = SkipEncodingBom::new(Cursor::new(bytes.as_slice()));
                let mut buf = Default::default();
                let _ = reader.read_to_end(&mut buf).unwrap();
                assert_eq!(b"This stream has a BOM.", buf.as_slice());
                assert_eq!(Some($bom), reader.bom_found().unwrap());
            }
        }
    };
}

test_read_bom_type!(test_read_utf8_bom, BomType::UTF8);
test_read_bom_type!(test_read_utf16le_bom, BomType::UTF16LE);
test_read_bom_type!(test_read_utf16be_bom, BomType::UTF16BE);
test_read_bom_type!(test_read_utf32le_bom, BomType::UTF32LE);
test_read_bom_type!(test_read_utf32be_bom, BomType::UTF32BE);
test_read_bom_type!(test_read_utf7_bom, BomType::UTF7);
test_read_bom_type!(test_read_utf1_bom, BomType::UTF1);
test_read_bom_type!(test_read_utfebdic_bom, BomType::UTFEBDIC);
test_read_bom_type!(test_read_scsu_bom, BomType::SCSU);
test_read_bom_type!(test_read_bocu1_bom, BomType::BOCU1);
test_read_bom_type!(test_read_gb1803_bom, BomType::GB1803);
