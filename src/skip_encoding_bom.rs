use super::*;

use std::io::{Cursor, Read};

/// Read from I/O and skip the initial encoding BOM if present.
#[derive(Debug, Clone)]
pub struct SkipEncodingBom<'a, R: Read> {
    reader: R,
    state: BomState,
    bom_types: &'a [BomType],
}

impl<'a, R: Read> SkipEncodingBom<'a, R> {
    /// Initialize an encoding BOM skip struct given any stream reader.
    /// 
    /// # Arguments
    /// 
    /// * `bom_types` - a slice with the types of BOM to check for. To skip any of the supported BOMs, pass [`BomType::all`].
    /// * `reader` - the underlying input stream reader.
    pub fn new(bom_types: &'a [BomType], reader: R) -> Self {
        Self {
            reader,
            state: BomState::default(),
            bom_types,
        }
    }
    /// Read the BOM from a reader if it is present and return the BOM found as an [`Option`] with a [`BomType`] or [`None`] if it was not found.
    /// 
    /// If the reader ends before a BOM if confirmed, [`None`] will be returned.  
    pub fn read_bom(&mut self) -> Result<Option<BomType>> {
        loop {
            match &self.state {
                BomState::Initial { start_bytes } => match Self::state_after_initial(start_bytes, &mut self.reader, self.bom_types)? {
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
    /// use skip_bom::{BomType, SkipEncodingBom};
    /// use std::io::{Cursor, Read};
    /// 
    /// // No BOM was found.
    /// {
    ///     const BYTES: &'static [u8] = b"This stream does not have a BOM.";
    ///     let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BYTES));
    ///     let mut buf = Default::default();
    ///     let _ = reader.read_to_end(&mut buf).unwrap();
    ///     assert_eq!(b"This stream does not have a BOM.", buf.as_slice());
    ///     assert_eq!(Some(None), reader.bom_found());
    /// }
    /// // Nothing was read yet.
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
    ///     let reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BYTES));
    ///     assert_eq!(None, reader.bom_found());
    /// }
    /// // The stream is to small.
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB";
    ///     let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BYTES));
    ///     let mut buf = Default::default();
    ///     let _ = reader.read_to_end(&mut buf).unwrap();
    ///     assert_eq!(None, reader.bom_found());
    ///     assert_eq!(b"", buf.as_slice());
    /// }
    /// // The buffer provided by the client is too small: the BOM is still read successfully.
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
    ///     let mut reader = SkipEncodingBom::new(BomType::all(), Cursor::new(BYTES));
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

    /// Get a shared reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Get a mutable reference to the underlying reader. 
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    fn state_after_initial(start_bytes: &BomBytesPushBuffer, reader: &mut R, bom_types: &[BomType]) -> Result<NextStateResult> {
        use NextStateResult::*;
        match BomState::try_read_bom(start_bytes, reader, bom_types)? {
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

impl<'a, R: Read> Read for SkipEncodingBom<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match &mut self.state {
                // initial state
                BomState::Initial { start_bytes } => match Self::state_after_initial(start_bytes, &mut self.reader, self.bom_types)? {
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
