//! # skip_bom
//! `skip_bom` provides a minimalist utility type read a I/O stream and skip the initial BOM bytes if they are present.
//! 
//! ## Examples
//! ```
//! use skip_bom::{BomType, SkipEncodingBom};
//! use std::io::{Cursor, Read};
//! 
//! // Read a stream after checking that it starts with the BOM
//! const BOM_BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
//! let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BOM_BYTES));
//! assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
//! let mut string = Default::default();
//! let _ = reader.read_to_string(&mut string).unwrap();
//! assert_eq!("This stream starts with a UTF-8 BOM.", &string);
//! 
//! // Read a stream without a starting BOM
//! const NO_BOM_BYTES: &'static [u8] = b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.";
//! let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(NO_BOM_BYTES));
//! assert_eq!(None, reader.read_bom().unwrap());
//! let mut buf = Default::default();
//! let _ = reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.", buf.as_slice());
//! 
//! // Read a stream and disregard the starting BOM completely
//! let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BOM_BYTES));
//! let mut buf = Default::default();
//! let _ = reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(b"This stream starts with a UTF-8 BOM.", buf.as_slice());
//! // Check the BOM after the read is over.
//! assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
//! ```

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

/// Re-exported from [`std::io::Result`]
pub type Result<T> = std::io::Result<T>;

mod bom_type;
pub use bom_type::*;

mod skip_encoding_bom;
pub use skip_encoding_bom::*;

mod bom_state;
pub(crate) use bom_state::*;

mod byte_push_buffer;
pub(crate) use byte_push_buffer::*;
