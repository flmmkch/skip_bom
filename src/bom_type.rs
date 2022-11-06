/// Type of encoding BOM.
/// 
/// See [the questions about the BOM in the official Unicode FAQ](https://www.unicode.org/faq/utf_bom.html#bom1).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BomType {
    /// Unicode with the UTF-8 format.
    UTF8,
    /// Unicode with the UTF-16 LE format.
    UTF16LE,
    /// Unicode with the UTF-16 BE format.
    UTF16BE,
    /// Unicode with the UTF-32 LE format.
    UTF32LE,
    /// Unicode with the UTF-32 BE format.
    UTF32BE,
    /// Unicode with the UTF-7 format.
    UTF7,
    /// Unicode with the UTF-1 format.
    UTF1,
    /// Unicode with the UTF-EBCDIC format.
    UTFEBDIC,
    /// Unicode with the SCSU format.
    SCSU,
    /// Unicode with the BOCU-1 format.
    BOCU1,
    /// GB1803 format: Information Technology — Chinese coded character set.
    GB1803,
}

impl BomType {
    pub const fn bom_bytes(&self) -> &'static [u8] {
        use BomType::*;

        match self {
            UTF8 => &[0xEF, 0xBB, 0xBF],
            UTF16LE => &[0xFF, 0xFE],
            UTF16BE => &[0xFE, 0xFF],
            UTF32LE => &[0x00, 0x00, 0xFF, 0xFE],
            UTF32BE => &[0xFF, 0xFE, 0x00, 0x00],
            UTF7 => &[0x2B, 0x2F, 0x76],
            UTF1 => &[0xF7, 0x64, 0x4C],
            UTFEBDIC => &[0xDD, 0x73, 0x66, 0x73],
            SCSU => &[0x0E, 0xFE, 0xFF],
            BOCU1 => &[0xFB, 0xEE, 0x28],
            GB1803 => &[0x84, 0x31, 0x95, 0x33],
        }
    }

    pub const fn bom_length(&self) -> usize {
        self.bom_bytes().len()
    }

    /// Returns:
    /// * `BomBytesTest::StartsWithBom` if `tested_bytes` is certain to start with the BOM.
    /// * `BomBytesTest::NotBom` if `tested_bytes` is certain not to be the BOM.
    /// * `BomBytesTest::Incomplete` otherwise.
    pub fn test_bytes(&self, tested_bytes: &[u8]) -> BomBytesTest {
        if tested_bytes.len() < self.bom_length() {
            if tested_bytes == &self.bom_bytes()[..tested_bytes.len()] {
                BomBytesTest::Incomplete
            }
            else {
                BomBytesTest::NotBom
            }
        } else {
            if &tested_bytes[..self.bom_length()] == self.bom_bytes() {
                BomBytesTest::StartsWithBom
            } else {
                BomBytesTest::NotBom
            }
        }
    }

    /// Returns:
    /// * `Ok((Some(bom_type), additional_bytes_slice)) if `tested_bytes` is certain to start with the `bom_type` BOM.
    /// * `Ok((None, bytes_slice))` if `tested_bytes` is certain not to be any BOM.
    /// * `Err(())` otherwise.
    pub fn try_find_bytes_bom<'a>(tested_bytes: &'a [u8]) -> AllBomsBytesTest<'a> {
        use BomType::*;

        let mut result = AllBomsBytesTest::Complete { bom_type: None, additional_bytes: tested_bytes };

        macro_rules! try_encoding {
            ($encoding:expr) => {
                match $encoding.test_bytes(tested_bytes) {
                    BomBytesTest::Incomplete => result = AllBomsBytesTest::Incomplete,
                    BomBytesTest::NotBom => (),
                    BomBytesTest::StartsWithBom => return AllBomsBytesTest::Complete { bom_type: Some($encoding), additional_bytes: &tested_bytes[$encoding.bom_length()..] },
                }
            };
        }

        try_encoding!(UTF8);
        try_encoding!(UTF32LE);
        try_encoding!(UTF32BE);
        try_encoding!(UTF16LE);
        try_encoding!(UTF16BE);
        try_encoding!(UTF7);
        try_encoding!(UTF1);
        try_encoding!(UTFEBDIC);
        try_encoding!(SCSU);
        try_encoding!(BOCU1);
        try_encoding!(GB1803);

        result
    }
}

/// Test result for the compatibility with a single BOM
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomBytesTest {
    /// Incomplete
    Incomplete,
    /// Incompatible with the BOM 
    NotBom,
    /// Starts with the complete BOM
    StartsWithBom,
}

/// Test result for the compatibility with any BOM
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllBomsBytesTest<'a> {
    /// Incomplete
    Incomplete,
    /// Complete
    Complete {
        /// the BOM type found or `None` if there is no compatible BOM
        bom_type: Option<BomType>,
        additional_bytes: &'a [u8],
    },
}

pub(crate) type BomSize = u8;

pub(crate) const MAX_BOM_LENGTH: BomSize = 4;
