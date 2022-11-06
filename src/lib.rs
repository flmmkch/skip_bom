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
//! let mut reader = SkipEncodingBom::new(Cursor::new(BOM_BYTES));
//! assert_eq!(Some(BomType::UTF8), reader.read_bom().unwrap());
//! let mut string = Default::default();
//! let _ = reader.read_to_string(&mut string).unwrap();
//! assert_eq!("This stream starts with a UTF-8 BOM.", &string);
//! 
//! // Read a stream without a starting BOM
//! const NO_BOM_BYTES: &'static [u8] = b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.";
//! let mut reader = SkipEncodingBom::new(Cursor::new(NO_BOM_BYTES));
//! assert_eq!(None, reader.read_bom().unwrap());
//! let mut buf = Default::default();
//! let _ = reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(b"This stream does not start with the UTF-8 BOM: \xEF\xBB\xBF.", buf.as_slice());
//! 
//! // Read a stream and disregard the starting BOM completely
//! let mut reader = SkipEncodingBom::new(Cursor::new(BOM_BYTES));
//! let mut buf = Default::default();
//! let _ = reader.read_to_end(&mut buf).unwrap();
//! assert_eq!(b"This stream starts with a UTF-8 BOM.", buf.as_slice());
//! // Check the BOM after the read is over.
//! assert_eq!(Some(Some(BomType::UTF8)), reader.bom_found());
//! ```

use std::io::{Cursor, Read};

pub type Result<T> = std::io::Result<T>;

const MAX_BOM_LENGTH: u8 = utf8::BOM_LENGTH;

type BomBytesArray = [u8; MAX_BOM_LENGTH as usize];

/// Type of encoding BOM.
/// 
/// See [the questions about the BOM in the official Unicode FAQ](https://www.unicode.org/faq/utf_bom.html#bom1).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BomType {
    /// Unicode with the UTF-8 format.
    UTF8,
}

/// Read from I/O and skip the initial encoding BOM if present.
#[derive(Debug, Clone)]
pub struct SkipEncodingBom<R: Read> {
    reader: R,
    state: BomState,
}

impl<R: Read> SkipEncodingBom<R> {
    /// Initialize an encoding BOM skip object with a reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            state: BomState::default(),
        }
    }
    /// Read the BOM from a reader if it is present and return the BOM found as an [`Option`] with a [`BomType`] or [`None`] if it was not found.
    /// 
    /// If the reader ends before a BOM if confirmed, [`None`] will be returned.  
    pub fn read_bom(&mut self) -> Result<Option<BomType>> {
        loop {
            match &self.state {
                BomState::Initial { start_bytes } => match Self::state_after_initial(start_bytes, &mut self.reader)? {
                    NextStateResult::NewState(new_state) => self.state = new_state,
                    NextStateResult::IncompleteRead(new_start_bytes) => {
                        self.state = BomState::Initial { start_bytes: new_start_bytes };
                        break Ok(None)
                    },
                },
                BomState::PostInitBuffer { bom_type, .. } | BomState::Final(bom_type) => break Ok(bom_type.clone()),
            }
        }
    }
    /// Return the BOM previously found as an inner [`Option`] with a [`BomType`] or [`None`] if it was not found, or [`None`] for the outer option if the presence of a BOM could not be determined yet.
    /// # Examples
    /// ```
    /// use skip_bom::SkipEncodingBom;
    /// use std::io::{Cursor, Read};
    /// 
    /// // No BOM was found.
    /// {
    ///     const BYTES: &'static [u8] = b"This stream does not have a BOM.";
    ///     let mut reader = SkipEncodingBom::new(Cursor::new(BYTES));
    ///     let mut buf = Default::default();
    ///     let _ = reader.read_to_end(&mut buf).unwrap();
    ///     assert_eq!(b"This stream does not have a BOM.", buf.as_slice());
    ///     assert_eq!(Some(None), reader.bom_found());
    /// }
    /// // Nothing was read yet.
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
    ///     let reader = SkipEncodingBom::new(Cursor::new(BYTES));
    ///     assert_eq!(None, reader.bom_found());
    /// }
    /// // The stream is to small.
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB";
    ///     let mut reader = SkipEncodingBom::new(Cursor::new(BYTES));
    ///     let mut buf = Default::default();
    ///     let _ = reader.read_to_end(&mut buf).unwrap();
    ///     assert_eq!(None, reader.bom_found());
    ///     assert_eq!(b"", buf.as_slice());
    /// }
    /// // The buffer provided by the client is too small: the BOM is still read successfully.
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
    ///     let mut reader = SkipEncodingBom::new(Cursor::new(BYTES));
    ///     let mut buf = [0u8; 2];
    ///     let _ = reader.read(&mut buf).unwrap();
    ///     assert_eq!(Some(Some(skip_bom::BomType::UTF8)), reader.bom_found());
    ///     assert_eq!(b"Th", buf.as_slice());
    /// }
    /// ```
    pub fn bom_found(&self) -> Option<Option<BomType>> {
        self.state.bom_found()
    }
    /// Unwraps this `SkipEncodingBom<R>`, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
    fn state_after_initial(start_bytes: &BomBytesPushBuffer, reader: &mut R) -> Result<NextStateResult> {
        use NextStateResult::*;
        match BomState::try_read_bom(start_bytes, reader)? {
            // no new bytes were read
            TryReadBomResult::Incomplete(new_start_bytes) if start_bytes.byte_count() == new_start_bytes.byte_count() => Ok(IncompleteRead(new_start_bytes)),
            // new bytes were read
            TryReadBomResult::Incomplete(new_start_bytes) => Ok(NewState(BomState::Initial { start_bytes: new_start_bytes })),
            // the BOM presence and type was determined
            TryReadBomResult::Complete { bom_type, bytes_after_bom } if bytes_after_bom.byte_count() == 0 => Ok(NewState(BomState::Final(bom_type))),
            TryReadBomResult::Complete { bom_type, bytes_after_bom } => Ok(NewState(BomState::PostInitBuffer { bytes_after_bom: Cursor::new(bytes_after_bom), bom_type })),
        }
    }
}

enum NextStateResult {
    IncompleteRead(BomBytesPushBuffer),
    NewState(BomState),
}

impl<R: Read> Read for SkipEncodingBom<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match &mut self.state {
                // initial state
                BomState::Initial { start_bytes } => match Self::state_after_initial(start_bytes, &mut self.reader)? {
                    NextStateResult::NewState(new_state) => self.state = new_state,
                    NextStateResult::IncompleteRead(new_bytes) => {
                        self.state = BomState::Initial { start_bytes: new_bytes };
                        break Ok(0)
                    },
                },
                BomState::PostInitBuffer { bytes_after_bom, bom_type } => {
                    let mut bytes_read = bytes_after_bom.read(buf)?;
                    if bytes_after_bom.position() == bytes_after_bom.get_ref().byte_count() as _ {
                        // if we are at the end of the post-init buffer, change state
                        self.state = BomState::Final(bom_type.take());
                        if bytes_read < buf.len() {
                            // if there is remaining space in the buffer
                            // then read from the underlying reader
                            bytes_read += self.reader.read(&mut buf[bytes_read..])?;
                        }
                    }
                    break Ok(bytes_read)
                },
                // read from the underlying reader
                BomState::Final(_) => break self.reader.read(buf),
            }
        }
    }
}

/// Reader BOM skipping state
#[derive(Debug, Clone)]
enum BomState {
    /// Reader initial state.
    Initial {
        /// Push buffer for the reader bytes that can be BOM bytes.
        start_bytes: BomBytesPushBuffer,
    },
    /// buffer state if the initialization is over but the client buffer could not hold everything
    PostInitBuffer {
        /// Buffer for the start bytes that can be BOM bytes.
        bytes_after_bom: Cursor<BomBytesPushBuffer>,
        /// The BOM type found if there was one.
        bom_type: Option<BomType>,
    },
    /// Reader state where the BOM has been determined to be present or not.
    Final(Option<BomType>),
}

impl Default for BomState {
    fn default() -> Self {
        Self::Initial { start_bytes: Default::default() }
    }
}

impl BomState {
    pub fn bom_found(&self) -> Option<Option<BomType>> {
        match self {
            BomState::Initial { .. } => None,
            BomState::PostInitBuffer { bom_type, .. } => Some(bom_type.clone()),
            BomState::Final(bom_type) => Some(bom_type.clone()),
        }
    }

    fn try_read_bom<R: Read>(start_bytes: &BomBytesPushBuffer, reader: &mut R) -> Result<TryReadBomResult> {
        // read into the start_bytes buffer
        let mut new_start_bytes_buffer = BomBytesArray::default();
        let start_bytes_slice = start_bytes.bytes();
        if !start_bytes_slice.is_empty() {
            new_start_bytes_buffer[..start_bytes_slice.len()].copy_from_slice(start_bytes_slice);
        }
        let read_slice = &mut new_start_bytes_buffer[start_bytes_slice.len()..];
        let current_bytes_read = reader.read(read_slice)?;
        let total_bom_bytes_read = start_bytes_slice.len() + current_bytes_read;
        match Self::test_incomplete_bom_bytes(&new_start_bytes_buffer[..total_bom_bytes_read]) {
            // the BOM presence was determined
            Ok((bom_type, additional_bytes)) => {
                let bytes_after_bom = BomBytesPushBuffer::from_slice(additional_bytes);
                Ok(TryReadBomResult::Complete { bom_type, bytes_after_bom })
            },
            Err(()) => {
                let bytes_after_bom = BomBytesPushBuffer::from_array(new_start_bytes_buffer, total_bom_bytes_read);
                Ok(TryReadBomResult::Incomplete(bytes_after_bom))
            }
        }
    }
    // returns Ok((Some(bom_type), additional_bytes_slice)) if this is certain to be a BOM, Ok((None, bytes_slice)) if the bytes are certain not to be a BOM, and Err(()) otherwise
    fn test_incomplete_bom_bytes<'a>(tested_bytes: &'a [u8]) -> std::result::Result<(Option<BomType>, &'a [u8]), ()> {
        let slice_len = tested_bytes.len();
        if slice_len < utf8::BOM_LENGTH as _ {
            if tested_bytes == &utf8::BOM[..slice_len] {
                Err(())
            }
            else {
                Ok((None, tested_bytes))
            }
        } else {
            if tested_bytes[..utf8::BOM_LENGTH as usize] == utf8::BOM {
                Ok((Some(BomType::UTF8), &tested_bytes[utf8::BOM_LENGTH as usize..]))
            } else {
                Ok((None, tested_bytes))
            }
        }
    }
}

enum TryReadBomResult {
    Incomplete(BomBytesPushBuffer),
    Complete { bom_type: Option<BomType>, bytes_after_bom: BomBytesPushBuffer },
}

#[derive(Default, Debug, Clone, Copy)]
struct BomBytesPushBuffer {
    buffer: BomBytesArray,
    position: usize,
}

impl BomBytesPushBuffer {
    fn from_slice(slice: &[u8]) -> Self {
        let mut bom_bytes_push_buffer = Self::default();
        bom_bytes_push_buffer.push(slice);
        bom_bytes_push_buffer
    }
    fn from_array(array: BomBytesArray, byte_count: usize) -> Self {
        Self {
            buffer: array,
            position: byte_count,
        }
    }
    fn available_bytes(&self) -> usize {
        self.buffer.len() - self.position
    }
    fn push(&mut self, bytes: &[u8]) -> usize {
        let count = bytes.len().min(self.available_bytes());
        self.buffer[self.position..(self.position + count)].copy_from_slice(&bytes[..count]);
        self.position += count;
        count
    }
    fn bytes(&self) -> &[u8] {
        &self.buffer[..self.position]
    }
    fn byte_count(&self) -> usize {
        self.position
    }
}

impl AsRef<[u8]> for BomBytesPushBuffer {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}

pub mod utf8;
