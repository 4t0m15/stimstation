// Buffers for persistent region rendering
use crate::types::{WIDTH, HEIGHT};

/// Buffers for persistent region buffers
#[derive(Debug)]
pub struct Buffers {
    pub original: Vec<u8>,
    pub circular: Vec<u8>,
    pub full: Vec<u8>,
}

impl Buffers {
    /// Create new set of buffers
    pub fn new() -> Self {
        Self {
            original: vec![0; (WIDTH * HEIGHT * 4) as usize],
            circular: vec![0; (WIDTH * HEIGHT * 4) as usize],
            full: vec![0; (WIDTH * HEIGHT * 4) as usize],
        }
    }
    
    /// Clear all buffers
    pub fn clear_all(&mut self) {
        for pixel in self.original.chunks_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        }
        
        for pixel in self.circular.chunks_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        }
        
        for pixel in self.full.chunks_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        }
    }
    
    /// Copy a buffer to the main frame
    pub fn copy_to_frame(&self, buffer_type: BufferType, frame: &mut [u8]) {
        match buffer_type {
            BufferType::Original => frame.copy_from_slice(&self.original),
            BufferType::Circular => frame.copy_from_slice(&self.circular),
            BufferType::Full => frame.copy_from_slice(&self.full),
        }
    }
    
    /// Get a mutable reference to a specific buffer
    pub fn get_mut(&mut self, buffer_type: BufferType) -> &mut [u8] {
        match buffer_type {
            BufferType::Original => &mut self.original,
            BufferType::Circular => &mut self.circular,
            BufferType::Full => &mut self.full,
        }
    }
}

/// Buffer types for different visualization regions
pub enum BufferType {
    Original,
    Circular,
    Full,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffers() {
        let buffers = Buffers::new();
        
        // Check if buffers have correct size
        assert_eq!(buffers.original.len(), (WIDTH * HEIGHT * 4) as usize);
        assert_eq!(buffers.circular.len(), (WIDTH * HEIGHT * 4) as usize);
        assert_eq!(buffers.full.len(), (WIDTH * HEIGHT * 4) as usize);
    }
    
    #[test]
    fn test_clear_all() {
        let mut buffers = Buffers::new();
        
        // Set some data in buffers
        buffers.original[0] = 255;
        buffers.circular[0] = 255;
        buffers.full[0] = 255;
        
        // Clear all buffers
        buffers.clear_all();
        
        // Check if buffers were cleared
        assert_eq!(buffers.original[0], 0);
        assert_eq!(buffers.circular[0], 0);
        assert_eq!(buffers.full[0], 0);
    }
}
