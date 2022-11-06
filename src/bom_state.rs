use std::io::{Cursor, Read};

use super::{AllBomsBytesTest, BomType, BomBytesArray, BomBytesPushBuffer, Result};

/// Reader BOM skipping state
#[derive(Debug, Clone)]
pub enum BomState {
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

    pub fn try_read_bom<R: Read>(start_bytes: &BomBytesPushBuffer, reader: &mut R) -> Result<TryReadBomResult> {
        // read into the start_bytes buffer
        let mut new_start_bytes_buffer = BomBytesArray::default();
        let start_bytes_slice = start_bytes.bytes();
        if !start_bytes_slice.is_empty() {
            new_start_bytes_buffer[..start_bytes_slice.len()].copy_from_slice(start_bytes_slice);
        }
        let read_slice = &mut new_start_bytes_buffer[start_bytes_slice.len()..];
        let current_bytes_read = reader.read(read_slice)?;
        let total_bom_bytes_read = start_bytes_slice.len() + current_bytes_read;
        match BomType::try_find_bytes_bom(&new_start_bytes_buffer[..total_bom_bytes_read]) {
            // the BOM presence was determined
            AllBomsBytesTest::Complete { bom_type, additional_bytes } => {
                let bytes_after_bom = BomBytesPushBuffer::from_slice(additional_bytes);
                Ok(TryReadBomResult::Complete { bom_type, bytes_after_bom })
            },
            AllBomsBytesTest::Incomplete => {
                let bytes_after_bom = BomBytesPushBuffer::from_array(new_start_bytes_buffer, total_bom_bytes_read);
                Ok(TryReadBomResult::Incomplete(bytes_after_bom))
            }
        }
    }
}

pub enum TryReadBomResult {
    Incomplete(BomBytesPushBuffer),
    Complete { bom_type: Option<BomType>, bytes_after_bom: BomBytesPushBuffer },
}
