use std::io::{Cursor, Read};
use std::convert::TryInto;

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];
const UTF8_BOM_LENGTH: u8 = UTF8_BOM.len() as u8;

/// Reader BOM skipping state
#[derive(Debug, Clone)]
enum BomState {
    /// Reader initial state.
    InitialState {
        /// Buffer for the start bytes that can be DOM bytes
        start_bytes: [u8; UTF8_BOM_LENGTH as usize],
        /// Number of initial bytes that have been read
        bytes_read: u8,
    },
    /// buffer state if the bom was not found but the client could not read everything
    NoBomBuffer {
        /// Buffer for the start bytes that can be DOM bytes
        cursor: Cursor<[u8; UTF8_BOM_LENGTH as usize]>,
    },
    /// Reader state where no BOM has not been found.
    NoBom,
    /// Reader state where the DOM has been found.
    Found,
}

/// Skip initial UTF-8 BOM from an I/O reader.
#[derive(Debug, Clone)]
pub struct SkipUtf8Bom<T: Read> {
    reader: T,
    state: BomState,
}

impl<T: Read> SkipUtf8Bom<T> {
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            state: BomState::InitialState { start_bytes: [0u8; UTF8_BOM_LENGTH as usize], bytes_read: 0 },
        }
    }
    /// Returns `Some(true)` if the BOM was found, `Some(false)` if it was not, or `None` if the presence of a DOM could not be determined yet.
    /// * Examples for the cases that could not be determined
    /// ```
    /// use skip_bom::SkipUtf8Bom;
    /// use std::io::{Cursor, Read};
    /// 
    /// // Nothing was read yet
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
    ///     let reader = SkipUtf8Bom::new(Cursor::new(BYTES));
    ///     assert_eq!(None, reader.found_bom());
    /// }
    /// // The stream is to small
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB";
    ///     let mut reader = SkipUtf8Bom::new(Cursor::new(BYTES));
    ///     let mut buf = Default::default();
    ///     let _ = reader.read_to_end(&mut buf).unwrap();
    ///     assert_eq!(None, reader.found_bom());
    ///     assert_eq!(b"", buf.as_slice());
    /// }
    /// // The buffer provided by the client is too small
    /// {
    ///     const BYTES: &'static [u8] = b"\xEF\xBB\xBFThis stream starts with a UTF-8 BOM.";
    ///     let mut reader = SkipUtf8Bom::new(Cursor::new(BYTES));
    ///     let mut buf = [0u8; 2];
    ///     // The first read does not go far enough to confirm the presence of the BOM.
    ///     let bytes_read = reader.read(&mut buf).unwrap();
    ///     assert_eq!(None, reader.found_bom());
    ///     assert_eq!(0, bytes_read);
    ///     // The second read confirms the presence of the BOM and reads the next bytes.
    ///     let _ = reader.read(&mut buf).unwrap();
    ///     assert_eq!(Some(true), reader.found_bom());
    ///     assert_eq!(b"Th", buf.as_slice());
    /// }
    /// ```
    pub fn found_bom(&self) -> Option<bool> {
        match self.state {
            BomState::InitialState { .. } => None,
            BomState::NoBom | BomState::NoBomBuffer { .. } => Some(false),
            BomState::Found => Some(true),
        }
    }
    pub fn into_inner(self) -> T {
        self.reader
    }
}


impl<T: Read> Read for SkipUtf8Bom<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (result, new_state) = match core::mem::replace(&mut self.state, BomState::NoBom) {
            // try to read the initial BOM if it can be found
            BomState::InitialState { mut start_bytes, bytes_read: previous_bytes_read } if previous_bytes_read < UTF8_BOM_LENGTH => {
                // read into the start_bytes buffer
                let required_read_end = UTF8_BOM_LENGTH.min(previous_bytes_read.saturating_add(buf.len().try_into().unwrap_or(core::u8::MAX)));
                let bytes_read_slice = if required_read_end >= UTF8_BOM_LENGTH {
                    &mut start_bytes[previous_bytes_read as usize..]
                } else {
                    &mut start_bytes[previous_bytes_read as usize..required_read_end as usize]
                };
                let current_bytes_read = self.reader.read(bytes_read_slice)?;
                match previous_bytes_read + current_bytes_read as u8 {
                    // the BOM was found
                    UTF8_BOM_LENGTH if start_bytes == UTF8_BOM => (self.reader.read(buf), BomState::Found),
                    // the bom was not found
                    UTF8_BOM_LENGTH => {
                        use core::cmp::Ordering;
                        match buf.len().cmp(&(UTF8_BOM_LENGTH as usize)) {
                            Ordering::Less => {
                                // remain in the initial state but we know the next bytes
                                let mut cursor = Cursor::new(start_bytes);
                                let bytes_read = cursor.read(buf)?;
                                (Ok(bytes_read), BomState::NoBomBuffer { cursor })
                            },
                            Ordering::Equal => {
                                buf.copy_from_slice(start_bytes.as_slice());
                                (Ok(start_bytes.len()), BomState::NoBom)
                            },
                            Ordering::Greater => {
                                buf[..UTF8_BOM_LENGTH as usize].copy_from_slice(start_bytes.as_slice());
                                let additional_bytes_read = self.reader.read(&mut buf[UTF8_BOM_LENGTH as usize..])?;
                                (Ok(UTF8_BOM_LENGTH as usize + additional_bytes_read), BomState::NoBom)
                            },
                        }
                    },
                    // continue reading the BOM
                    n if n < UTF8_BOM_LENGTH => {
                        let bytes_read = previous_bytes_read + current_bytes_read as u8;
                        if start_bytes[..n as usize] == UTF8_BOM[..n as usize] {
                            (Ok(0), BomState::InitialState { start_bytes, bytes_read })
                        }
                        else {
                            buf[..bytes_read as usize].copy_from_slice(&start_bytes[..bytes_read as usize]);
                            (Ok(bytes_read as usize), BomState::NoBom)
                        }
                    },
                    _ => unreachable!(),
                }
            },
            BomState::NoBomBuffer { mut cursor } => {
                let cursor_bytes_read = cursor.read(buf)?;
                if cursor_bytes_read < buf.len() {
                    let reader_bytes_read = self.reader.read(&mut buf[cursor_bytes_read..])?;
                    (Ok(cursor_bytes_read + reader_bytes_read), BomState::NoBom)
                }
                else if cursor.position() == cursor.get_ref().len() as _ {
                    (Ok(cursor_bytes_read), BomState::NoBom)
                } else {
                    // if we continue through the buffer
                    (Ok(cursor_bytes_read), BomState::NoBomBuffer { cursor })
                }
            },
            // bytes_read must always be inferior to UTF8_BOM_LENGTH
            BomState::InitialState { .. } => unreachable!(),
            // after the initial state: inner reader
            BomState::Found => (self.reader.read(buf), BomState::Found),
            BomState::NoBom => (self.reader.read(buf), BomState::NoBom),
        };
        self.state = new_state;
        result
    }
}
