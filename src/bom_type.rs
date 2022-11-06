/// Type of encoding BOM.
/// 
/// See [the questions about the BOM in the official Unicode FAQ](https://www.unicode.org/faq/utf_bom.html#bom1).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BomType {
    /// Unicode with the UTF-8 format.
    UTF8,
}
