//! # skip_bom
//! `skip_bom` provides a minimalist utility type read a I/O stream and skip the initial DOM bytes if they are present.
//! 
//! # Examples
//! ```
//! use skip_bom::SkipUtf8Bom;
//! use std::io::{Cursor, Read};
//! 
//! // Read a stream starting with the DOM
//! {
//!     const BOM_BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
//!     let mut reader = SkipUtf8Bom::new(Cursor::new(BOM_BYTES));
//!     let mut string = Default::default();
//!     let _ = reader.read_to_string(&mut string).unwrap();
//!     assert_eq!(Some(true), reader.found_bom());
//!     assert_eq!("This stream starts with a UTF-8 BOM.", &string);
//! }
//! // Read a stream without a starting DOM
//! {
//!     const BOM_BYTES: &'static [u8] = b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.";
//!     let mut reader = SkipUtf8Bom::new(Cursor::new(BOM_BYTES));
//!     let mut buf = Default::default();
//!     let _ = reader.read_to_end(&mut buf).unwrap();
//!     assert_eq!(Some(false), reader.found_bom());
//!     assert_eq!(b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.", buf.as_slice());
//! }
//! ```

mod utf8;
pub use self::utf8::*;
