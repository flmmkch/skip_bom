use std::io::{Cursor, Read};

use skip_bom::{BomType, SkipEncodingBom};

pub fn skip_utf8_bom_reader_from_byte_slice<'a>(bytes: &'a [u8]) -> SkipEncodingBom<'static, Cursor<&'a [u8]>> {
    skip_utf8_bom_reader(Cursor::new(bytes))
}

pub fn skip_utf8_bom_reader<R: Read>(reader: R) -> SkipEncodingBom<'static, R> {
    SkipEncodingBom::new(&[BomType::UTF8], reader)
}

