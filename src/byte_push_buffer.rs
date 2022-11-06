
#[derive(Default, Debug, Clone, Copy)]
pub struct BomBytesPushBuffer {
    buffer: BomBytesArray,
    position: usize,
}

impl BomBytesPushBuffer {
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bom_bytes_push_buffer = Self::default();
        bom_bytes_push_buffer.push(slice);
        bom_bytes_push_buffer
    }
    pub fn from_array(array: BomBytesArray, byte_count: usize) -> Self {
        Self {
            buffer: array,
            position: byte_count,
        }
    }
    pub fn available_bytes(&self) -> usize {
        self.buffer.len() - self.position
    }
    pub fn push(&mut self, bytes: &[u8]) -> usize {
        let count = bytes.len().min(self.available_bytes());
        self.buffer[self.position..(self.position + count)].copy_from_slice(&bytes[..count]);
        self.position += count;
        count
    }
    pub fn bytes(&self) -> &[u8] {
        &self.buffer[..self.position]
    }
    pub fn byte_count(&self) -> usize {
        self.position
    }
}

impl AsRef<[u8]> for BomBytesPushBuffer {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}

pub type BomBytesArray = [u8; crate::MAX_BOM_LENGTH as usize];
