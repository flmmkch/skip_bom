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
    /// * `Ok((Some(bom_type), additional_bytes_slice))` if `tested_bytes` is certain to start with the `bom_type` BOM.
    /// * `Ok((None, bytes_slice))` if `tested_bytes` is certain not to be any BOM.
    /// * `Err(())` otherwise.
    pub fn try_find_bytes_bom<'a>(tested_bytes: &'a [u8], bom_types_tested: &[BomType]) -> BomsBytesTest<'a> {
        use BomType::*;

        let mut result = BomsBytesTest::Complete { bom_type: None, additional_bytes: tested_bytes };

        macro_rules! try_encoding {
            ($encoding:expr) => {
                if bom_types_tested.contains(&$encoding) {
                    match $encoding.test_bytes(tested_bytes) {
                        BomBytesTest::Incomplete => result = BomsBytesTest::Incomplete,
                        BomBytesTest::NotBom => (),
                        BomBytesTest::StartsWithBom => return BomsBytesTest::Complete { bom_type: Some($encoding), additional_bytes: &tested_bytes[$encoding.bom_length()..] },
                    }
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

    /// Get a slice containing a list of all BOM types available.
    pub fn all() -> &'static [BomType] {
        use BomType::*;
        &[
            UTF8,
            UTF32LE,
            UTF32BE,
            UTF16LE,
            UTF16BE,
            UTF7,
            UTF1,
            UTFEBDIC,
            SCSU,
            BOCU1,
            GB1803,
        ]
    }
}

/// Test result for the compatibility with a single BOM.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomBytesTest {
    /// The byte array tested is not long enough to confirm whether the BOM is present or absent.
    Incomplete,
    /// The byte array tested is confirmed not to start with the BOM.
    NotBom,
    /// The byte array tested is confirmed to start with the BOM.
    StartsWithBom,
}

/// Test result for the compatibility with multiple BOMs.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomsBytesTest<'a> {
    /// The byte array tested is not long enough to confirm whether one of the BOMs is present.
    Incomplete,
    /// The byte array tested is confirmed to start with either one of the BOMs or none of them.
    Complete {
        /// The BOM type found or `None` if there is no compatible BOM.
        bom_type: Option<BomType>,
        /// Additional bytes found in the tested buffer after the BOM.
        additional_bytes: &'a [u8],
    },
}

pub(crate) type BomSize = u8;

pub(crate) const MAX_BOM_LENGTH: BomSize = 4;
